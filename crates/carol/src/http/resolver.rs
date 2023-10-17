use std::{net::SocketAddr, str::FromStr};

use carol_core::MachineId;
use hickory_resolver::{
    error::ResolveErrorKind,
    proto::rr::{rdata::CNAME, RecordType},
    Name, TokioAsyncResolver, TryParseIp,
};
use hyper::http::HeaderValue;

#[derive(Clone)]
pub struct Resolver {
    inner: TokioAsyncResolver,
    api_host: Option<Name>,
    base_domain: Option<Name>,
}

pub enum Resolution {
    Api,
    Machine(MachineId),
    Unknown,
}

impl Resolver {
    pub fn new(config: crate::config::dns::Config) -> Self {
        Self {
            inner: TokioAsyncResolver::tokio(config.hickory_conf, config.hickory_opts),
            api_host: config.api_host,
            base_domain: config.base_domain,
        }
    }

    pub async fn resolve_host(&self, host_header: &HeaderValue) -> anyhow::Result<Resolution> {
        if self.api_host.is_none() {
            return Ok(Resolution::Api);
        }

        let host = match host_header.to_str() {
            Ok(host) => {
                if let Ok(_) = SocketAddr::from_str(host) {
                    return Ok(Resolution::Api);
                }
                if let Some(_) = host.try_parse_ip() {
                    return Ok(Resolution::Api);
                }
                match Name::from_str(host).ok() {
                    Some(host) => host,
                    None => return Ok(Resolution::Unknown),
                }
            }
            Err(_) => return Ok(Resolution::Unknown),
        };

        if Some(&host) == self.api_host.as_ref() {
            return Ok(Resolution::Api);
        }

        if let Some(machine_id) = self.matches_machine(&host) {
            return Ok(Resolution::Machine(machine_id));
        }

        let lookup = match self.inner.lookup(host, RecordType::CNAME).await {
            Ok(lookup) => lookup,
            Err(e) => match e.kind() {
                ResolveErrorKind::NoRecordsFound { .. } => return Ok(Resolution::Unknown),
                _ => Err(e)?,
            },
        };

        for record in lookup.into_iter() {
            if let Ok(CNAME(cname)) = record.into_cname() {
                if let Some(machine_id) = self.matches_machine(&cname) {
                    return Ok(Resolution::Machine(machine_id));
                }
            }
        }

        Ok(Resolution::Unknown)
    }

    fn matches_machine(&self, name: &Name) -> Option<MachineId> {
        let mut labels = name.iter();
        let first = labels.next().unwrap_or(&[]);
        let first = String::from_utf8(first.to_vec()).ok()?;
        let machine_id = MachineId::from_str(&first).ok()?;
        let base_domain = Name::from_labels(labels).ok()?;

        if self.base_domain == Some(base_domain) {
            Some(machine_id)
        } else {
            None
        }
    }
}

use crate::{ExecutorState, State};
use anyhow::anyhow;
use async_trait::async_trait;
use carol_bls as bls;
use carol_core::MachineId;
use hyper::StatusCode;
use tracing::{event, Level};
use wasmtime::component::bindgen;

bindgen!({
    world: "machine",
    path: "../../wit/",
    tracing: true,
    async: true,
});

pub use exports::carol::machine::guest;

pub struct Host {
    pub env: Environment,
    pub panic_message: Option<String>,
}

pub enum Environment {
    Activation {
        machine_id: MachineId,
        http_client: reqwest::Client,
        state: State,
    },
    Http {
        machine_id: MachineId,
        state: State,
    },
    BinaryApi,
}

impl Environment {
    pub fn machine_id(&self) -> anyhow::Result<MachineId> {
        match self {
            Environment::Activation { machine_id, .. } | Environment::Http { machine_id, .. } => {
                Ok(*machine_id)
            }
            Environment::BinaryApi => Err(anyhow!("No machine in this environment")),
        }
    }
    pub fn http_client(&self) -> anyhow::Result<&reqwest::Client> {
        match self {
            Environment::Activation { http_client, .. } => Ok(http_client),
            _ => Err(anyhow!(
                "cannot use http client in http handler environment"
            )),
        }
    }

    pub fn bls_keypair(&self) -> anyhow::Result<bls::KeyPair> {
        match self {
            Environment::Activation { state, .. } => Ok(state.bls_keypair),
            _ => Err(anyhow!("cannot access BLS key in http handler environment")),
        }
    }

    pub fn executor_state(&self) -> anyhow::Result<ExecutorState> {
        match self {
            Environment::Activation { state, .. } | Environment::Http { state, .. } => {
                Ok(state.exec.clone())
            }
            _ => Err(anyhow!("cannot activate machines in this environment")),
        }
    }

    pub fn global_state(&self) -> anyhow::Result<State> {
        match self {
            Environment::Activation { state, .. } | Environment::Http { state, .. } => {
                Ok(state.clone())
            }
            _ => Err(anyhow!("cannot activate machines in this environment")),
        }
    }
}

pub use carol::machine::*;

#[async_trait]
impl http::Host for Host {
    async fn execute(
        &mut self,
        request: http::Request,
    ) -> anyhow::Result<Result<http::Response, http::Error>> {
        let client = self.env.http_client()?;
        let inner_result = (|| async {
            let request: reqwest::Request = request.try_into()?;
            let res = client.execute(request).await?;
            let headers = res
                .headers()
                .into_iter()
                .map(|(key, value)| Ok((key.to_string(), value.as_bytes().to_vec())))
                .collect::<Result<_, http::Error>>()?;
            let response = http::Response {
                status: res.status().as_u16(),
                body: res.bytes().await?.to_vec(),
                headers,
            };
            Ok(response)
        })()
        .await;
        Ok(inner_result)
    }
}

#[async_trait]
impl global::Host for Host {
    async fn bls_static_pubkey(&mut self) -> anyhow::Result<Vec<u8>> {
        Ok(self.env.bls_keypair()?.public_key().to_bytes().to_vec())
    }

    async fn bls_static_sign(&mut self, message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        Ok(
            carol_bls::sign(self.env.bls_keypair()?, self.env.machine_id()?, &message)
                .0
                .to_uncompressed()
                .to_vec(),
        )
    }
}

#[async_trait]
impl log::Host for Host {
    async fn info(&mut self, message: String) -> anyhow::Result<()> {
        event!(Level::DEBUG, message = message);
        Ok(())
    }

    async fn set_panic_message(&mut self, message: String) -> anyhow::Result<()> {
        event!(Level::DEBUG, message = message, "panic_message_set");
        self.panic_message = Some(message);
        Ok(())
    }
}

#[async_trait]
impl machines::Host for Host {
    async fn self_activate(
        &mut self,
        method_name: String,
        input: Vec<u8>,
    ) -> anyhow::Result<Result<Vec<u8>, machines::Error>> {
        let exec_state = self.env.executor_state()?;
        let machine_id = self.env.machine_id()?;
        let (binary_id, params) = exec_state
            .get_machine(self.env.machine_id()?)
            .expect("must exist since we are running it!");
        let compiled_binary = exec_state
            .get_binary(binary_id)
            .expect("must exist since we are running it!");
        match exec_state
            .executor()
            .activate_machine(
                self.env.global_state()?,
                compiled_binary.as_ref(),
                params.as_ref(),
                &method_name,
                &input,
            )
            .await
        {
            Ok(Ok(output)) => Ok(Ok(output)),
            Ok(Err(e)) => {
                event!(
                    Level::ERROR,
                    input = carol_core::hex::encode(&input),
                    "self_activate'd guest failed"
                );
                Ok(Err(machines::Error::Panic(machines::PanicInfo {
                    reason: e.to_string(),
                    machine: machine_id.to_bytes().to_vec(),
                })))
            }
            Err(e) => {
                event!(
                    Level::ERROR,
                    input = carol_core::hex::encode(&input),
                    "self_activate failed due to host error"
                );
                Err(e)
            }
        }
    }
}

impl TryFrom<http::Request> for http_crate::Request<hyper::Body> {
    type Error = http::Error;

    fn try_from(guest_request: http::Request) -> Result<Self, Self::Error> {
        let mut builder = http_crate::Request::builder()
            .method(guest_request.method)
            .uri(guest_request.uri);

        for (key, value) in guest_request.headers {
            builder = builder.header(key, value);
        }

        Ok(builder.body(hyper::Body::from(guest_request.body))?)
    }
}

impl TryFrom<http::Request> for reqwest::Request {
    type Error = http::Error;

    fn try_from(value: http::Request) -> Result<Self, Self::Error> {
        let http_req: http_crate::Request<hyper::Body> = value.try_into()?;
        // XXX: From looking at source it looks like this error can only happen because the uri does
        // not fit the stricter requirements of reqwest.
        http_req
            .try_into()
            .map_err(|e: reqwest::Error| http::Error::InvalidUrl(e.to_string()))
    }
}

impl From<http_crate::Error> for http::Error {
    fn from(e: http_crate::Error) -> Self {
        use http_crate::{header, uri};
        if e.is::<uri::InvalidUri>() || e.is::<uri::InvalidUriParts>() {
            http::Error::InvalidUrl(e.to_string())
        } else if e.is::<header::InvalidHeaderName>() || e.is::<header::InvalidHeaderValue>() {
            http::Error::InvalidHeader(e.to_string())
        } else {
            http::Error::Unexpected(e.to_string())
        }
    }
}

impl From<reqwest::Error> for http::Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            http::Error::Timeout
        } else if e.is_connect() {
            http::Error::Connection(e.to_string())
        } else {
            http::Error::Unexpected(e.to_string())
        }
    }
}

impl From<http::Method> for http_crate::Method {
    fn from(value: http::Method) -> Self {
        use http::Method::*;
        match value {
            Get => http_crate::Method::GET,
            Post => http_crate::Method::POST,
            Put => http_crate::Method::PUT,
            Delete => http_crate::Method::DELETE,
            Patch => http_crate::Method::PATCH,
        }
    }
}

impl TryFrom<http::Response> for http_crate::Response<hyper::Body> {
    type Error = anyhow::Error;
    fn try_from(res: http::Response) -> Result<Self, Self::Error> {
        let mut builder = http_crate::Response::builder().status(StatusCode::from_u16(res.status)?);

        for (key, value) in res.headers {
            builder = builder.header(key, value);
        }

        Ok(builder.body(hyper::Body::from(res.body))?)
    }
}

impl TryFrom<http_crate::Method> for http::Method {
    type Error = anyhow::Error;

    fn try_from(method: http_crate::Method) -> Result<http::Method, Self::Error> {
        Ok(match method.as_str() {
            "GET" => http::Method::Get,
            "POST" => http::Method::Post,
            "PUT" => http::Method::Put,
            "DELETE" => http::Method::Delete,
            "PATCH" => http::Method::Patch,
            method => {
                return Err(anyhow!(
                    "carol doesn't support ‘{}’ as a http method",
                    method
                ))
            }
        })
    }
}

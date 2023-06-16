use crate::State;
use anyhow::anyhow;
use async_trait::async_trait;
use carol_bls as bls;
use carol_core::MachineId;
use hyper::StatusCode;
use tracing::{event, Level};
use wasmtime::component::bindgen;

bindgen!({
    world: "carol.machine",
    path: "../../wit/v0.1.0",
    tracing: true,
    async: true,
});

pub struct Host {
    pub machine_id: MachineId,
    pub state: State,
    pub env: Environment,
    pub panic_message: Option<String>,
}
pub enum Environment {
    Activation {
        bls_keypair: bls::KeyPair,
        http_client: reqwest::Client,
    },
    Http,
}

impl Environment {
    pub fn http_client(&self) -> &reqwest::Client {
        match self {
            Environment::Activation { http_client, .. } => http_client,
            Environment::Http { .. } => {
                panic!("cannot use http client in http handler environment")
            }
        }
    }

    pub fn bls_keypair(&self) -> &bls::KeyPair {
        match self {
            Environment::Activation { bls_keypair, .. } => bls_keypair,
            Environment::Http { .. } => panic!("cannot access BLS key in http handler environment"),
        }
    }
}

#[async_trait]
impl http::Host for Host {
    async fn execute(&mut self, request: http::RequestResult) -> anyhow::Result<http::Response> {
        let client = self.env.http_client();
        let request = request.try_into()?;
        let res = client.execute(request).await?;
        let headers = res
            .headers()
            .into_iter()
            .map(|(key, value)| Ok((key.to_string(), value.to_str()?.to_string())))
            .collect::<Result<_, anyhow::Error>>()?;
        let response = http::Response {
            status: res.status().as_u16(),
            body: res.bytes().await?.to_vec(),
            headers,
        };
        Ok(response)
    }
}

#[async_trait]
impl global::Host for Host {
    async fn bls_static_pubkey(&mut self) -> anyhow::Result<Vec<u8>> {
        Ok(self
            .env
            .bls_keypair()
            .public_key()
            .to_uncompressed()
            .to_vec())
    }

    async fn bls_static_sign(&mut self, message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        Ok(
            carol_bls::sign(self.state.bls_keypair(), self.machine_id, &message)
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
    async fn self_activate(&mut self, input: Vec<u8>) -> anyhow::Result<Result<Vec<u8>, String>> {
        let (binary_id, params) = self
            .state
            .get_machine(self.machine_id)
            .expect("must exist since we are running it!");
        let compiled_binary = self
            .state
            .get_binary(binary_id)
            .expect("must exist since we are running it!");
        match self
            .state
            .executor()
            .activate_machine(
                self.state.clone(),
                compiled_binary.as_ref(),
                params.as_ref(),
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
                Ok(Err(e.to_string()))
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

impl<'a> TryFrom<http::RequestParam<'a>> for http_crate::Request<hyper::Body> {
    type Error = anyhow::Error;

    fn try_from(guest_request: http::RequestParam) -> Result<Self, Self::Error> {
        let mut builder = http_crate::Request::builder()
            .method(guest_request.method)
            .uri(guest_request.uri);

        for (key, value) in guest_request.headers {
            builder = builder.header(*key, *value);
        }

        Ok(builder.body(hyper::Body::from(guest_request.body.to_vec()))?)
    }
}

impl TryFrom<http::RequestResult> for http_crate::Request<hyper::Body> {
    type Error = anyhow::Error;

    fn try_from(guest_request: http::RequestResult) -> Result<Self, Self::Error> {
        let mut builder = http_crate::Request::builder()
            .method(guest_request.method)
            .uri(guest_request.uri);

        for (key, value) in guest_request.headers {
            builder = builder.header(key, value);
        }

        Ok(builder.body(hyper::Body::from(guest_request.body))?)
    }
}

impl<'a> TryFrom<http::RequestParam<'a>> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(value: http::RequestParam) -> Result<Self, Self::Error> {
        let http_req: http_crate::Request<hyper::Body> = value.try_into()?;
        Ok(http_req.try_into()?)
    }
}

impl TryFrom<http::RequestResult> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(value: http::RequestResult) -> Result<Self, Self::Error> {
        let http_req: http_crate::Request<hyper::Body> = value.try_into()?;
        Ok(http_req.try_into()?)
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

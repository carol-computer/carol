use async_trait::async_trait;
use rand::RngCore;
use std::str::FromStr;
use wasmtime::component::bindgen;

bindgen!({
    world: "contract",
    path: "../../wit",
    async: true,
});

pub struct Host {
    pub bls_keypair: BlsKeyPair,
    pub contract_id: [u8; 32],
    pub http_client: reqwest::Client,
}

impl TryFrom<http::Request> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(guest_request: http::Request) -> Result<Self, Self::Error> {
        let uri = reqwest::Url::parse(&guest_request.uri)?;
        let mut req = reqwest::Request::new(guest_request.method.into(), uri);
        let headers = req.headers_mut();
        // TODO: stop panic: HeaderMap can store a maximum of 32,768 headers (header name / value pairs). Attempting to insert more will result in a panic.
        for (key, value) in guest_request.headers {
            let header_name = reqwest::header::HeaderName::from_str(&key)?;
            let header_value = reqwest::header::HeaderValue::from_str(&value)?;
            headers.append(header_name, header_value);
        }

        *req.body_mut() = Some(reqwest::Body::from(guest_request.body));

        Ok(req)
    }
}

impl From<http::Method> for reqwest::Method {
    fn from(value: http::Method) -> Self {
        use http::Method::*;
        match value {
            Get => reqwest::Method::GET,
            Post => reqwest::Method::POST,
            Put => reqwest::Method::PUT,
            Delete => reqwest::Method::DELETE,
        }
    }
}

#[async_trait]
impl http::Host for Host {
    async fn execute(&mut self, request: http::Request) -> anyhow::Result<http::Response> {
        let request = request.try_into()?;
        let res = self.http_client.execute(request).await?;
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

#[derive(Clone, Debug)]
pub struct BlsKeyPair {
    pk: bls12_381::G1Affine,
    sk: bls12_381::Scalar,
}

impl BlsKeyPair {
    pub fn new(sk: bls12_381::Scalar) -> Self {
        let pk = bls12_381::G1Affine::generator() * &sk;
        Self { pk: pk.into(), sk }
    }

    pub fn random(rng: &mut impl RngCore) -> Self {
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        let sk = bls12_381::Scalar::from_bytes_wide(&bytes);
        Self::new(sk)
    }
}

#[async_trait]
impl global::Host for Host {
    async fn bls_static_pubkey(&mut self) -> anyhow::Result<Vec<u8>> {
        Ok(self.bls_keypair.pk.to_uncompressed().to_vec())
    }

    async fn bls_static_sign(&mut self, message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        use bls12_381::{
            hash_to_curve::{ExpandMsgXmd, HashToCurve},
            G2Affine, G2Projective,
        };
        let point = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
            message,
            self.contract_id.as_ref(),
        );

        Ok(G2Affine::from(point * self.bls_keypair.sk)
            .to_uncompressed()
            .to_vec())
    }
}

#[async_trait]
impl log::Host for Host {
    async fn info(&mut self, message: String) -> anyhow::Result<()> {
        println!("{}", message);
        Ok(())
    }
}

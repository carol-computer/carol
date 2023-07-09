use super::*;
use carol_core::{BinaryId, MachineId};
pub struct ActivateCap;

impl http::Cap for ActivateCap {
    fn http_execute(&self, _request: http::Request) -> Result<http::Response, http::Error> {
        panic!("cannot call activate outside of WASM guest environment")
    }
}

impl bls::Cap for ActivateCap {
    fn bls_static_public_key(&self) -> bls::PublicKey {
        panic!("cannot call activate outside of WASM guest environment")
    }

    fn bls_static_sign(&self, _message: &[u8]) -> bls::Signature {
        panic!("cannot call activate outside of WASM guest environment")
    }
}

impl log::Cap for ActivateCap {
    fn log_info(&self, _message: &str) {
        panic!("cannot call activate outside of WASM guest environment")
    }
}

impl machines::Cap for ActivateCap {
    fn self_activate(&self, _input: &[u8]) -> Result<Vec<u8>, machines::Error> {
        panic!("cannot call activate outside of WASM guest environment")
    }
}

pub struct TestCap {
    http_client: reqwest::blocking::Client,
    bls_keypair: carol_bls::KeyPair,
}

impl Default for TestCap {
    fn default() -> Self {
        Self {
            bls_keypair: carol_bls::KeyPair::new(carol_bls::bls12_381::Scalar::from_bytes_wide(
                &[42u8; 64],
            )),
            http_client: Default::default(),
        }
    }
}

impl TestCap {
    pub fn new(bls_keypair: carol_bls::KeyPair) -> Self {
        Self {
            bls_keypair,
            http_client: reqwest::blocking::Client::default(),
        }
    }
}

impl http::Cap for TestCap {
    fn http_execute(&self, request: http::Request) -> Result<http::Response, http::Error> {
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
        let request = http_crate::Request::try_from(request)?;
        let response = self.http_client.execute(request.try_into()?)?;
        Ok(http::Response {
            headers: response
                .headers()
                .iter()
                .map(|(key, value)| (key.as_str().into(), value.as_bytes().to_vec()))
                .collect(),
            status: response.status().as_u16(),
            body: response.bytes()?.to_vec(),
        })
    }
}

impl bls::Cap for TestCap {
    fn bls_static_public_key(&self) -> bls::PublicKey {
        self.bls_keypair.public_key()
    }

    fn bls_static_sign(&self, message: &[u8]) -> bls::Signature {
        let machine_id = MachineId::new(BinaryId::new(b"test"), &[]);
        carol_bls::sign(self.bls_keypair, machine_id, message)
    }
}

impl log::Cap for TestCap {
    fn log_info(&self, message: &str) {
        println!("LOG: {}", message);
    }
}

pub struct HttpHandlerCap;

impl machines::Cap for HttpHandlerCap {
    fn self_activate(&self, _input: &[u8]) -> Result<Vec<u8>, machines::Error> {
        todo!("we can't do activations outside of carol guest environments yeth")
    }
}

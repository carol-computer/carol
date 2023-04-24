#[allow(unused)]
#[macro_use]
extern crate alloc;

use ::http as http_crate;
pub use bincode;
pub use carol_bls as bls;
pub use carol_guest_derive::{activate, carol};
pub use serde;
pub use serde_json;

pub mod bind {
    wit_bindgen::generate!({
        world: "carol.machine",
        path: "../../wit/v0.1.0",
        macro_export,
        export_macro_name: "set_machine",
        macro_call_prefix: "carol_guest::bind::",
    });
}

pub mod http {
    use super::*;
    use alloc::{string::ToString, vec::Vec};
    pub use bind::http::{Method, Request, Response};

    impl From<Method> for http_crate::Method {
        fn from(value: Method) -> Self {
            use Method::*;
            match value {
                Get => http_crate::Method::GET,
                Post => http_crate::Method::POST,
                Put => http_crate::Method::PUT,
                Delete => http_crate::Method::DELETE,
                Patch => http_crate::Method::PATCH,
            }
        }
    }

    impl TryFrom<Request> for http_crate::Request<Vec<u8>> {
        type Error = http_crate::Error;

        fn try_from(req: Request) -> Result<Self, Self::Error> {
            let mut builder = http_crate::Request::builder()
                .method(req.method)
                .uri(req.uri);

            for (key, value) in req.headers {
                builder = builder.header(key, value);
            }

            builder.body(req.body)
        }
    }

    impl From<http_crate::Response<Vec<u8>>> for Response {
        fn from(res: http_crate::Response<Vec<u8>>) -> Self {
            Response {
                status: res.status().as_u16(),
                headers: res
                    .headers()
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.as_str().to_string(),
                            value
                                .to_str()
                                .expect("TODO: support binary headers")
                                .to_string(),
                        )
                    })
                    .collect(),
                body: res.into_body(),
            }
        }
    }

    impl Request {
        pub fn uri(&self) -> http_crate::Uri {
            use core::str::FromStr;
            http_crate::Uri::from_str(&self.uri).unwrap()
        }
    }
}

/// Capabilties passed into the guest WASM in different contexts
pub mod cap {
    use super::*;

    pub trait Log {
        fn log_info(&self, message: &str);
    }

    pub trait Bls {
        fn bls_static_public_key(&self) -> carol_bls::bls12_381::G1Affine;
        fn bls_static_sign(&self, message: &[u8]) -> carol_bls::Signature;
    }

    pub trait Machines {
        fn self_activate(&self, input: &[u8]) -> Result<Vec<u8>, String>;
    }

    pub trait Http {
        fn http_execute(&self, request: http::Request) -> http::Response;
        fn http_get(&self, uri: &str) -> http::Response {
            self.http_execute(http::Request {
                headers: vec![],
                body: vec![],
                method: http::Method::Get,
                uri: uri.into(),
            })
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    pub struct ActivateCap;
    pub struct HttpHandlerCap;
    pub use bind::__link_section;

    impl cap::Machines for ActivateCap {
        fn self_activate(&self, input: &[u8]) -> Result<Vec<u8>, String> {
            bind::machines::self_activate(input)
        }
    }

    impl cap::Http for ActivateCap {
        fn http_execute(&self, request: http::Request) -> http::Response {
            bind::http::execute(&request)
        }
    }

    impl cap::Bls for ActivateCap {
        fn bls_static_public_key(&self) -> bls::bls12_381::G1Affine {
            let mut bytes = [0u8; 96];
            bytes.copy_from_slice(bind::global::bls_static_pubkey().as_ref());
            carol_bls::bls12_381::G1Affine::from_uncompressed_unchecked(&bytes).unwrap()
        }

        fn bls_static_sign(&self, message: &[u8]) -> carol_bls::Signature {
            let mut bytes = [0u8; 192];
            let sig = bind::global::bls_static_sign(message);
            bytes.copy_from_slice(&sig);
            carol_bls::Signature(
                carol_bls::bls12_381::G2Affine::from_uncompressed_unchecked(&bytes).unwrap(),
            )
        }
    }

    impl cap::Log for ActivateCap {
        fn log_info(&self, message: &str) {
            bind::log::info(message)
        }
    }

    impl cap::Log for HttpHandlerCap {
        fn log_info(&self, message: &str) {
            bind::log::info(message)
        }
    }

    impl cap::Machines for HttpHandlerCap {
        fn self_activate(&self, input: &[u8]) -> Result<Vec<u8>, String> {
            bind::machines::self_activate(input)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod other {
    use super::*;
    use carol_core::{BinaryId, MachineId};

    pub struct ActivateCap;

    impl cap::Http for ActivateCap {
        fn http_execute(&self, _request: http::Request) -> http::Response {
            panic!("cannot call activate outside of WASM guest environment")
        }
    }

    impl cap::Bls for ActivateCap {
        fn bls_static_public_key(&self) -> bls::bls12_381::G1Affine {
            panic!("cannot call activate outside of WASM guest environment")
        }

        fn bls_static_sign(&self, _message: &[u8]) -> bls::Signature {
            panic!()
        }
    }

    impl cap::Log for ActivateCap {
        fn log_info(&self, _message: &str) {
            panic!("cannot call activate outside of WASM guest environment")
        }
    }

    impl cap::Machines for ActivateCap {
        fn self_activate(&self, _input: &[u8]) -> Result<Vec<u8>, String> {
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
                bls_keypair: carol_bls::KeyPair::new(
                    carol_bls::bls12_381::Scalar::from_bytes_wide(&[42u8; 64]),
                ),
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

    impl cap::Http for TestCap {
        fn http_execute(&self, request: http::Request) -> http::Response {
            let request = http_crate::Request::try_from(request).expect("TODO handle errors");
            let response = self
                .http_client
                .execute(request.try_into().expect("todo handle errors"))
                .expect("TODO handle errors");
            http::Response {
                headers: response
                    .headers()
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.as_str().into(),
                            value.to_str().expect("handle binary headers").into(),
                        )
                    })
                    .collect(),
                status: response.status().as_u16(),
                body: response.bytes().expect("handle errors").into(),
            }
        }
    }

    impl cap::Bls for TestCap {
        fn bls_static_public_key(&self) -> bls::bls12_381::G1Affine {
            self.bls_keypair.public_key()
        }

        fn bls_static_sign(&self, message: &[u8]) -> carol_bls::Signature {
            let machine_id = MachineId::new(BinaryId::new(b"test"), &[]);
            carol_bls::sign(&self.bls_keypair, machine_id, message)
        }
    }

    impl cap::Log for TestCap {
        fn log_info(&self, message: &str) {
            println!("LOG: {}", message);
        }
    }

    pub struct HttpHandlerCap;

    impl cap::Machines for HttpHandlerCap {
        fn self_activate(&self, _input: &[u8]) -> Result<Vec<u8>, String> {
            todo!("we can't do activations outside of carol guest environments yeth")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use other::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

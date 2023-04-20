#![no_std]

#[allow(unused)]
#[macro_use]
extern crate alloc;

pub use bincode;
pub use carol_guest_derive::{activate, carol};
pub use serde;
pub use serde_json;

mod bind {
    wit_bindgen::generate!({
        world: "carol.machine",
        path: "../../wit/v0.1.0",
        macro_export,
        export_macro_name: "set_machine",
        macro_call_prefix: "carol_guest::",
    });
}

pub mod bls {
    use super::*;
    pub use carol_bls::*;
    pub fn static_pubkey() -> bls12_381::G1Affine {
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(bind::global::bls_static_pubkey().as_ref());
        bls12_381::G1Affine::from_uncompressed_unchecked(&bytes).unwrap()
    }

    pub fn static_sign(message: &[u8]) -> Signature {
        let mut bytes = [0u8; 192];
        let sig = bind::global::bls_static_sign(message);
        bytes.copy_from_slice(&sig);
        Signature(bls12_381::G2Affine::from_uncompressed_unchecked(&bytes).unwrap())
    }
}

pub mod http {
    pub use super::bind::http as wit_http;
    use http as http_crate;
    pub use wit_http::{Method, Request, Response};

    pub fn http_get(uri: &str) -> Response {
        wit_http::execute(&Request {
            body: vec![],
            headers: vec![],
            method: Method::Get,
            uri: uri.into(),
        })
    }

    impl Request {
        pub fn uri(&self) -> http_crate::Uri {
            use core::str::FromStr;
            http_crate::Uri::from_str(&self.uri).unwrap()
        }
    }
}

// pub trait Ctx {
//     fn http(&self) -> impl Http;
// }

// pub type HttpRequest<'a> = bind::http::RequestParam<'a>;
// pub type HttpResponse

// pub trait Http {
//     fn execute<'a>(&self, HttpRequest<'a>) -> bind
// }

#[cfg(target_arch = "wasm32")]
pub use bind::__link_section;
pub use bind::{log, machine, machines};

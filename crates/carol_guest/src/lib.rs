#![no_std]

#[allow(unused)]
#[macro_use]
extern crate alloc;

pub use bincode;
pub use carol_guest_derive::{activate, carol};
pub use serde;
pub use serde_json;

mod raw {
    wit_bindgen::generate!({
        world: "carol.machine",
        path: "../../wit/v0.1.0",
        macro_export,
        export_macro_name: "set_machine"
    });
}

pub mod bls {
    use super::*;
    pub use carol_bls::*;
    pub fn static_pubkey() -> bls12_381::G1Affine {
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(raw::global::bls_static_pubkey().as_ref());
        bls12_381::G1Affine::from_uncompressed_unchecked(&bytes).unwrap()
    }

    pub fn static_sign(message: &[u8]) -> Signature {
        let mut bytes = [0u8; 192];
        let sig = raw::global::bls_static_sign(message);
        bytes.copy_from_slice(&sig);
        Signature(bls12_381::G2Affine::from_uncompressed_unchecked(&bytes).unwrap())
    }
}

pub mod http {
    pub use super::raw::http as wit_http;
    use http as http_crate;

    pub fn http_get(url: &str) -> wit_http::Response {
        wit_http::execute(wit_http::RequestParam {
            body: &[],
            headers: &[],
            method: wit_http::Method::Get,
            uri: url,
        })
    }

    impl wit_http::RequestResult {
        pub fn uri(&self) -> http::Uri {
            use core::str::FromStr;
            http_crate::Uri::from_str(&self.uri).unwrap()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use raw::__link_section;
pub use raw::{log, machine, machines};

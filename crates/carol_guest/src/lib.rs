pub use bincode;
use bincode::{de::read::Reader, enc::write::Writer, impl_borrow_decode};
pub use bls12_381;
pub use carol_guest_derive::carol_contract;

mod raw {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "contract",
        macro_export,
        export_macro_name: "set_contract"
    });
}

#[derive(Debug, Clone, Copy)]
pub struct BlsSignature(pub bls12_381::G2Affine);

impl bincode::Encode for BlsSignature {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        encoder.writer().write(&self.0.to_compressed())?;
        Ok(())
    }
}

impl bincode::Decode for BlsSignature {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let mut bytes = [0u8; 96];
        decoder.reader().read(&mut bytes)?;
        Ok(BlsSignature(
            bls12_381::G2Affine::from_compressed(&bytes).unwrap(),
        ))
    }
}

impl_borrow_decode!(BlsSignature);

pub mod global {
    use super::*;
    pub fn bls_static_pubkey() -> bls12_381::G1Affine {
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(raw::global::bls_static_pubkey().as_ref());
        bls12_381::G1Affine::from_uncompressed_unchecked(&bytes).unwrap()
    }

    pub fn bls_static_sign(message: &[u8]) -> BlsSignature {
        let mut bytes = [0u8; 192];
        let sig = raw::global::bls_static_sign(message);
        bytes.copy_from_slice(&sig);
        BlsSignature(bls12_381::G2Affine::from_uncompressed_unchecked(&bytes).unwrap())
    }
}

pub mod http {
    use super::raw;
    pub fn http_get(url: &str) -> raw::http::Response {
        raw::http::http_get(url)
    }
}

#[cfg(target_arch = "wasm32")]
pub use raw::__link_section;
pub use raw::{contract, log};

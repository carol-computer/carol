use super::*;
pub struct ActivateCap;
pub struct HttpHandlerCap;
pub use bind::__link_section;
use bind::carol::machine;

impl machines::Cap for ActivateCap {
    fn self_activate(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, machines::Error> {
        Ok(machine::machines::self_activate(method, input)?)
    }
}

impl http::Cap for ActivateCap {
    fn http_execute(&self, request: http::Request) -> Result<http::Response, http::Error> {
        Ok(machine::http::execute(&request)?)
    }
}

impl bls::Cap for ActivateCap {
    fn bls_static_public_key(&self) -> bls::PublicKey {
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(machine::global::bls_static_pubkey().as_ref());
        carol_bls::PublicKey(
            carol_bls::bls12_381::G1Affine::from_uncompressed_unchecked(&bytes).unwrap(),
        )
    }

    fn bls_static_sign(&self, message: &[u8]) -> carol_bls::Signature {
        let mut bytes = [0u8; 192];
        let sig = machine::global::bls_static_sign(message);
        bytes.copy_from_slice(&sig);
        carol_bls::Signature(
            carol_bls::bls12_381::G2Affine::from_uncompressed_unchecked(&bytes).unwrap(),
        )
    }
}

impl log::Cap for ActivateCap {
    fn log_info(&self, message: &str) {
        machine::log::info(message)
    }
}

impl log::Cap for HttpHandlerCap {
    fn log_info(&self, message: &str) {
        machine::log::info(message)
    }
}

impl machines::Cap for HttpHandlerCap {
    fn self_activate(&self, method_name: &str, input: &[u8]) -> Result<Vec<u8>, machines::Error> {
        Ok(machine::machines::self_activate(method_name, input)?)
    }
}

pub use carol_bls::*;

pub trait Cap {
    fn bls_static_public_key(&self) -> carol_bls::PublicKey;
    fn bls_static_sign(&self, message: &[u8]) -> carol_bls::Signature;
}

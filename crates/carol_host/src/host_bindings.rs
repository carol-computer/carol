use rand::RngCore;
use wasmtime::component::bindgen;

bindgen!("contract" in "../../wit");

pub struct Host {
    pub bls_keypair: BlsKeyPair,
    pub contract_id: [u8; 32],
}

impl http::Host for Host {
    fn http_get(&mut self, url: String) -> anyhow::Result<http::Response> {
        let reqwest_response = reqwest::blocking::get(url)?;
        let response = http::Response {
            status: reqwest_response.status().as_u16(),
            body: reqwest_response.bytes()?.to_vec(),
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

impl global::Host for Host {
    fn bls_static_pubkey(&mut self) -> anyhow::Result<Vec<u8>> {
        Ok(self.bls_keypair.pk.to_uncompressed().to_vec())
    }

    fn bls_static_sign(&mut self, message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
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

impl log::Host for Host {
    fn info(&mut self, message: String) -> anyhow::Result<()> {
        println!("{}", message);
        Ok(())
    }
}

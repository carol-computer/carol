pub use bls12_381;
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    G2Affine, G2Projective,
};
use carol_core::{impl_display_debug_serialize, impl_fromstr_deserialize, MachineId};

#[derive(Clone, Copy)]
pub struct KeyPair {
    pk: bls12_381::G1Affine,
    sk: bls12_381::Scalar,
}

impl KeyPair {
    pub fn new(sk: bls12_381::Scalar) -> Self {
        let pk = bls12_381::G1Affine::generator() * &sk;
        Self { pk: pk.into(), sk }
    }

    pub fn random(rng: &mut impl rand_core::RngCore) -> Self {
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        let sk = bls12_381::Scalar::from_bytes_wide(&bytes);
        Self::new(sk)
    }

    pub fn public_key(&self) -> bls12_381::G1Affine {
        self.pk
    }

    pub fn secret_key(&self) -> bls12_381::Scalar {
        self.sk
    }
}

#[derive(Clone, Copy)]
pub struct Signature(pub G2Affine);

impl_display_debug_serialize! {
    fn to_bytes(bls: &Signature) -> [u8;96] {
        bls.0.to_compressed()
    }
}

impl_fromstr_deserialize! {
    name => "BLS signature",
    fn from_bytes(bytes: [u8;96]) -> Option<Signature> {
        Some(Signature(G2Affine::from_compressed(&bytes).unwrap(),
        ))
    }
}

pub fn sign(keypair: &KeyPair, machine_id: MachineId, message: &[u8]) -> Signature {
    let point = <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
        message,
        machine_id.as_ref(),
    );
    Signature(bls12_381::G2Affine::from(point * keypair.secret_key()))
}

impl_fromstr_deserialize! {
    name => "BLS12-381 scalar",
    fn from_bytes(bytes: [u8;32]) -> Option<KeyPair> {
        Option::<KeyPair>::from(bls12_381::Scalar::from_bytes(&bytes).map(KeyPair::new))
    }
}

impl_display_debug_serialize! {
    fn to_bytes(kp: &KeyPair) -> [u8;32] {
        kp.sk.to_bytes()
    }
}

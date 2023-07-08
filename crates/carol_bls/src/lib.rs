//! BLS signatures using the short public key variant.
//! Hopefully compatible with: https://datatracker.ietf.org/doc/draft-irtf-cfrg-bls-signature/05/ (but haven't checked).
//!
//! The reason it's the long signture/short public key is for DLC performance but we probably want
//! short signatures as the standard.
pub use bls12_381;
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    G1Affine, G2Affine, G2Projective, Scalar,
};
use carol_core::{impl_display_debug_serialize, impl_fromstr_deserialize, MachineId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyPair {
    pk: bls12_381::G1Affine,
    sk: bls12_381::Scalar,
}

impl KeyPair {
    pub fn new(sk: bls12_381::Scalar) -> Self {
        let pk = bls12_381::G1Affine::generator() * sk;
        Self { pk: pk.into(), sk }
    }

    pub fn random(rng: &mut impl rand_core::RngCore) -> Self {
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        let sk = bls12_381::Scalar::from_bytes_wide(&bytes);
        Self::new(sk)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.pk)
    }

    pub fn secret_key(&self) -> bls12_381::Scalar {
        self.sk
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PublicKey(pub G1Affine);

impl_display_debug_serialize! {
    fn to_bytes(public_key: &PublicKey) -> [u8;48] {
        public_key.0.to_compressed()
    }
}

impl_fromstr_deserialize! {
    name => "BLS public key",
    fn from_bytes(bytes: [u8;48]) -> Option<PublicKey> {
        Some(PublicKey(Option::from(G1Affine::from_compressed(&bytes))?))
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

pub fn sign(keypair: KeyPair, machine_id: MachineId, message: &[u8]) -> Signature {
    let message_point = hash_to_curve(machine_id, message);
    Signature(G2Affine::from(message_point * keypair.secret_key()))
}

fn hash_to_curve(machine_id: MachineId, message: &[u8]) -> G2Projective {
    <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(
        message,
        machine_id.as_ref(),
    )
}

#[must_use]
pub fn verify(
    carol_public_key: PublicKey,
    machine_id: MachineId,
    signature: Signature,
    message: &[u8],
) -> bool {
    let message_point = G2Affine::from(hash_to_curve(machine_id, message));
    bls12_381::pairing(&carol_public_key.0, &message_point)
        == bls12_381::pairing(&G1Affine::generator(), &signature.0)
}

impl_fromstr_deserialize! {
    name => "BLS12-381 scalar",
    fn from_bytes(bytes: [u8;32]) -> Option<KeyPair> {
        Option::<KeyPair>::from(Scalar::from_bytes(&bytes).map(KeyPair::new))
    }
}

impl_display_debug_serialize! {
    fn to_bytes(kp: &KeyPair) -> [u8;32] {
        kp.sk.to_bytes()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn sign_verify(sk in any::<[u8;64]>(), message in any::<[u8;32]>(), machine_id in any::<[u8;32]>()) {
            let sk = Scalar::from_bytes_wide(&sk);
            let kp = KeyPair::new(sk);
            let machine_id = MachineId::from_bytes(machine_id);

            let signature = sign(kp, machine_id, &message[..]);
            prop_assert!(verify(kp.public_key(), machine_id, signature, &message))

        }
    }
}

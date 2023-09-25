#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

/// Re-export `serde`
pub use serde;

/// Re-export `bincode`
pub use bincode;

pub mod hex;
mod macros;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct MachineId([u8; 32]);

impl MachineId {
    pub fn new(binary_id: BinaryId, params: &[u8]) -> Self {
        let mut hash = Sha256::default();
        hash.update(binary_id.as_ref());
        hash.update(params);
        Self(hash.finalize().into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct BinaryId([u8; 32]);

impl BinaryId {
    pub fn new(binary: &[u8]) -> Self {
        let mut hash = Sha256::default();
        hash.update(binary);
        Self(hash.finalize().into())
    }
}

impl AsRef<[u8; 32]> for MachineId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsRef<[u8; 32]> for BinaryId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

crate::impl_fromstr_deserialize! {
    name => "machine id",
    fn from_bytes(bytes: [u8;32]) -> MachineId {
        MachineId(bytes)
    }
}

crate::impl_display_debug_serialize! {
    fn to_bytes(machine_id: &MachineId) -> [u8;32] {
        machine_id.0
    }
}

crate::impl_fromstr_deserialize! {
    name => "binary id",
    fn from_bytes(bytes: [u8;32]) -> BinaryId {
        BinaryId(bytes)
    }
}

crate::impl_display_debug_serialize! {
    fn to_bytes(binary_id: &BinaryId) -> [u8;32] {
        binary_id.0
    }
}

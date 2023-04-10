#![no_std]

#[cfg(feature = "alloc")]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

/// Re-export `serde`
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
pub use serde;

/// Re-export `bincode`
#[cfg_attr(docsrs, doc(cfg(feature = "bincode")))]
#[cfg(feature = "bincode")]
pub use bincode;


pub mod hex;
mod macros;
use sha2::{Digest, Sha256};


#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::BorrowDecode) )]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
pub struct FullActivation<'a> {
    pub binary: &'a [u8],
    pub parameters: &'a [u8],
    pub activation_input: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct MachineId([u8;32]);


impl MachineId {
    pub fn new(binary_hash: [u8;32], params: &[u8]) -> Self {
        let mut hash = Sha256::default();
        hash.update(&[0x0]); // kind so we can change structure later
        hash.update(binary_hash);
        hash.update(params);
        Self(hash.finalize().into())
    }
}

impl AsRef<[u8;32]> for MachineId {
    fn as_ref(&self) -> &[u8;32] {
        &self.0
    }
}

crate::impl_fromstr_deserialize! {
    name => "contract id",
    fn from_bytes(bytes: [u8;32]) -> Option<MachineId> {
        Some(MachineId(bytes))
    }
}

crate::impl_display_debug_serialize! {
    fn to_bytes(machine_id: &MachineId) -> &[u8;32] {
        &machine_id.0
    }
}

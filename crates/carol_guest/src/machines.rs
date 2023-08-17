use crate::bind::carol::machine::machines;
use carol_core::MachineId;

pub trait Cap {
    fn self_activate(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, Error>;
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Error {
    Panic { reason: String, machine: MachineId },
}

impl From<machines::Error> for Error {
    fn from(value: machines::Error) -> Self {
        match value {
            machines::Error::Panic(machines::PanicInfo { reason, machine }) => Error::Panic {
                reason,
                machine: MachineId::from_slice(&machine[..]).unwrap(),
            },
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Panic { reason, machine } => write!(
                f,
                "call to machine {} failed because it panicked: {}",
                machine, reason
            ),
        }
    }
}

impl std::error::Error for Error {}

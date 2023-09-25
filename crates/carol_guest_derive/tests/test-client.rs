#![allow(renamed_and_removed_lints, unknown_lints, disallowed_names)]
use carol_guest_derive::{activate, codec, machine};
use core::any::Any;

#[codec]
pub struct Thing;

#[machine]
impl Thing {
    #[activate]
    pub fn one(&self, _cap: &impl Any, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }

    #[activate]
    pub fn two(&self, _cap: &impl Any, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }
}

#[cfg(test)]
mod test {

    use carol_guest::carol_core::MachineId;

    #[test]
    fn client_receives_correct_name_and_data() {
        struct TestClient;
        struct Error;

        impl From<bincode::error::DecodeError> for Error {
            fn from(_: bincode::error::DecodeError) -> Self {
                todo!()
            }
        }

        impl From<bincode::error::EncodeError> for Error {
            fn from(_: bincode::error::EncodeError) -> Self {
                todo!()
            }
        }

        impl carol_guest::Client for TestClient {
            type Error = Error;

            fn activate(
                &self,
                _: carol_guest::carol_core::MachineId,
                method_name: &str,
                input: &[u8],
            ) -> Result<Vec<u8>, Self::Error> {
                if method_name == "two" && input == [0x01, 0x02] {
                    Ok(vec![0x03])
                } else {
                    Err(Error)
                }
            }
        }

        let client = super::client::Client {
            client: TestClient,
            machine_id: MachineId::default(),
        };

        assert!(matches!(client.two(1, 2), Ok(3)));
    }
}

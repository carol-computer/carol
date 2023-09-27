use carol_core::MachineId;

pub trait Client {
    type Error: From<bincode::error::DecodeError> + From<bincode::error::EncodeError>;
    fn activate(
        &self,
        machine_id: MachineId,
        method_name: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;
}

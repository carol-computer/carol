use alloc::{string::String, vec::Vec};
use bech32::{FromBase32, ToBase32};
use carol_core::MachineId;

pub fn host_header_label_for_machine(machine_id: MachineId) -> String {
    bech32::encode_without_checksum("carol", machine_id.to_bytes().to_base32()).unwrap()
}

pub fn parse_host_header_label_for_machine(string: &str) -> Option<MachineId> {
    let (hrp, data) = bech32::decode_without_checksum(string).ok()?;
    if hrp != "carol" {
        return None;
    }
    let vec_data = Vec::from_base32(&data).ok()?;
    Some(MachineId::from_bytes(<[u8; 32]>::try_from(vec_data).ok()?))
}

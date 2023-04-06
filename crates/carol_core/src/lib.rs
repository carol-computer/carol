#[derive(bincode::Encode, bincode::BorrowDecode)]
pub struct FullActivation<'a> {
    pub binary: &'a [u8],
    pub parameters: &'a [u8],
    pub activation_input: &'a [u8],
}

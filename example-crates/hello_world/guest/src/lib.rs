use carol_guest::{activate, bincode, cap, carol, serde};

#[derive(bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
#[serde(crate = "carol_guest::serde")]
#[bincode(crate = "carol_guest::bincode")]
pub struct HelloWorld;

#[carol]
impl HelloWorld {
    #[activate]
    pub fn say(&self, cap: &impl cap::Log, message: String) -> String {
        let response = format!("hello {}", message);
        cap.log_info(&response);
        response
    }
}

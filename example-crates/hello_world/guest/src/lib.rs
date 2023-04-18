use carol_guest::*;

#[derive(bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
#[serde(crate = "carol_guest::serde")]
#[bincode(crate = "carol_guest::bincode")]
pub struct HelloWorld;

#[carol]
impl HelloWorld {
    #[activate]
    pub fn say(&self, message: String) -> String {
        let response = format!("hello {}", message);
        log::info(&response);
        response
    }
}

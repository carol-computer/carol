use carol_guest::{activate, codec, log, machine};

#[codec]
pub struct HelloWorld;

#[machine]
impl HelloWorld {
    #[activate(http(POST))]
    pub fn say(&self, cap: &impl log::Cap, message: String) -> String {
        let response = format!("hello {}", message);
        cap.log_info(&response);
        response
    }
}

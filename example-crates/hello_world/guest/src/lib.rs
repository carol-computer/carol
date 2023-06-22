use carol_guest::{activate, cap, codec, machine};

#[codec]
pub struct HelloWorld;

#[machine]
impl HelloWorld {
    #[activate]
    pub fn say(&self, cap: &impl cap::Log, message: String) -> String {
        let response = format!("hello {}", message);
        cap.log_info(&response);
        response
    }
}

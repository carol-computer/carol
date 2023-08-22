use carol_guest_derive::{activate, codec, machine};
use core::any::Any;

#[codec]
pub struct Foo;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Param(String);

// can't actually test this yet just checking it builds
#[machine]
impl Foo {
    #[activate]
    pub fn echo(&self, _cap: &impl Any, #[with_serde] param: Param) -> String {
        param.0
    }

    #[activate]
    pub fn skipped(&self, _cap: &impl Any, #[with_serde(skip)] param: Param) -> String {
        param.0
    }
}

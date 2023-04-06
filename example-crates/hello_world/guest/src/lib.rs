#![no_std]

#[macro_use]
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use carol_guest::*;

set_contract!(HelloWorld);

#[derive(bincode::Encode, bincode::Decode)]
pub struct HelloWorld;

#[carol_contract]
impl HelloWorld {
    pub fn say(&self, message: String) -> String {
        let response = format!("hello {}", message);
        log::info(&response);
        response
    }
}

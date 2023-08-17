#![allow(renamed_and_removed_lints, unknown_lints, disallowed_names)]
use carol_guest::bind::exports::carol::machine::guest::Guest;
use carol_guest_derive::{activate, codec, machine};
use core::any::Any;

#[codec]
pub struct Foo;

#[machine]
impl Foo {
    #[activate]
    pub fn add(&self, _cap: &impl Any, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }

    #[activate]
    pub fn checked_sub(&self, _cap: &impl Any, lhs: u32, rhs: u32) -> Option<u32> {
        lhs.checked_sub(rhs)
    }

    pub fn non_activate(&self) {
        unreachable!()
    }
}

use carol_activate::{Add, CheckedSub};

#[test]
fn call_add() {
    let input =
        bincode::encode_to_vec(Add { lhs: 7, rhs: 3 }, bincode::config::standard()).unwrap();
    let foo = Foo;
    let output = Foo::activate(
        bincode::encode_to_vec(foo, bincode::config::standard()).unwrap(),
        "add".into(),
        input,
    );
    let (result, _): (u32, _) =
        bincode::decode_from_slice(&output, bincode::config::standard()).unwrap();
    assert_eq!(result, 10);
}

#[test]
fn call_sub() {
    let input =
        bincode::encode_to_vec(CheckedSub { lhs: 7, rhs: 3 }, bincode::config::standard()).unwrap();
    let foo = Foo;
    let output = Foo::activate(
        bincode::encode_to_vec(foo, bincode::config::standard()).unwrap(),
        "checked_sub".into(),
        input,
    );
    let (result, _): (Option<u32>, _) =
        bincode::decode_from_slice(&output, bincode::config::standard()).unwrap();
    assert_eq!(result, Some(4));
}

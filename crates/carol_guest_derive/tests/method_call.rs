#![allow(renamed_and_removed_lints, unknown_lints, disallowed_names)]
use carol_guest::bind::machine::Machine;
use carol_guest_derive::{activate, carol};
use core::any::Any;

#[derive(bincode::Encode, bincode::Decode)]
pub struct Foo;

#[carol]
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

use carol_activate::{Activate, Add, CheckedSub};

#[test]
fn call_add() {
    let method = Activate::Add(Add { lhs: 7, rhs: 3 });
    let call = bincode::encode_to_vec(method, bincode::config::standard()).unwrap();
    let foo = Foo;
    let output = Foo::activate(
        bincode::encode_to_vec(foo, bincode::config::standard()).unwrap(),
        call,
    );
    let (result, _): (u32, _) =
        bincode::decode_from_slice(&output, bincode::config::standard()).unwrap();
    assert_eq!(result, 10);
}

#[test]
fn call_sub() {
    let method = Activate::CheckedSub(CheckedSub { lhs: 7, rhs: 3 });
    let call = bincode::encode_to_vec(method, bincode::config::standard()).unwrap();
    let foo = Foo;
    let output = Foo::activate(
        bincode::encode_to_vec(foo, bincode::config::standard()).unwrap(),
        call,
    );
    let (result, _): (Option<u32>, _) =
        bincode::decode_from_slice(&output, bincode::config::standard()).unwrap();
    assert_eq!(result, Some(4));
}

#[test]
fn only_activate_attribute_makes_activation_possible() {
    let method = Activate::Add(Add { lhs: 7, rhs: 3 });
    // exhaustively match
    match method {
        Activate::Add(_) => {}
        Activate::CheckedSub(_) => {}
    }
}

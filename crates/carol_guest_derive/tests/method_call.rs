use carol_guest::contract::Contract;
use carol_guest_derive::carol_contract;

#[derive(bincode::Encode, bincode::Decode)]
pub struct Foo;

#[carol_contract]
impl Foo {
    pub fn add(&self, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }

    pub fn checked_sub(&self, lhs: u32, rhs: u32) -> Option<u32> {
        lhs.checked_sub(rhs)
    }
}

#[test]
fn call_add() {
    let method = FooMethods::Add { lhs: 7, rhs: 3 };
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
    let method = FooMethods::CheckedSub { lhs: 7, rhs: 3 };
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

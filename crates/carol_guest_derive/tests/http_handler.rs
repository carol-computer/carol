use carol_guest::bind::exports::machine::Machine;
use carol_guest::http;
use carol_guest_derive::{activate, codec, machine};
use core::any::Any;

#[codec]
pub struct Foo;

#[derive(bincode::Decode, bincode::Encode, Debug, Clone)]
pub struct NoSerde;

#[machine]
impl Foo {
    #[activate(http(POST))]
    pub fn post_add(&self, _cap: &impl Any, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }

    #[activate(http(GET))]
    pub fn get_add(&self, _cap: &impl Any, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }

    #[activate]
    pub fn no_http(&self, _cap: &impl Any, _arg: NoSerde) {
        unreachable!()
    }
}

// note these should_panic not becuase they are wrong but because they work and get past
// deserialization
#[test]
#[should_panic]
fn post_request() {
    let _response = Foo::handle_http(http::Request {
        method: http::Method::Post,
        uri: "/activate/post_add".into(),
        body: r#"{"lhs": 3, "rhs": 4}"#.as_bytes().to_vec(),
        headers: vec![],
    });
}

#[test]
#[should_panic]
fn get_request() {
    let _response = Foo::handle_http(http::Request {
        method: http::Method::Get,
        uri: "/activate/get_add?lhs=3&rhs=4".into(),
        body: vec![],
        headers: vec![],
    });
}

#[test]
fn post_request_invalid_params() {
    let response = Foo::handle_http(http::Request {
        method: http::Method::Post,
        uri: "/activate/post_add".into(),
        body: r#"{"lhs": 3, "rhs": "foo"}"#.as_bytes().to_vec(),
        headers: vec![],
    });

    assert_eq!(response.status, 400);
}

#[test]
fn get_request_invalid_params() {
    let response = Foo::handle_http(http::Request {
        method: http::Method::Get,
        uri: "/activate/get_add?lhs=3&rhs=foo".into(),
        body: vec![],
        headers: vec![],
    });

    assert_eq!(response.status, 400);
}

#[test]
fn method_wihout_http_should_404() {
    let response = Foo::handle_http(http::Request {
        method: http::Method::Get,
        uri: "/activate/no_http".into(),
        body: vec![],
        headers: vec![],
    });

    assert_eq!(response.status, 404);
}

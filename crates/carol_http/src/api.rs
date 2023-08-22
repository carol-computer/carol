use alloc::{collections::BTreeMap, string::String};
use carol_core::{serde, BinaryId, MachineId};
use hyper::{header, http::HeaderValue, HeaderMap, StatusCode};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BinaryCreated {
    pub id: BinaryId,
}

impl BinaryCreated {
    pub fn location(&self) -> String {
        format!("/binaries/{}", self.id)
    }
}

pub trait Response: serde::Serialize {
    fn status(&self) -> StatusCode {
        StatusCode::OK
    }
    fn set_headers(&self, _headers: &mut HeaderMap) {}
}

impl Response for BinaryCreated {
    fn status(&self) -> StatusCode {
        StatusCode::CREATED
    }

    fn set_headers(&self, headers: &mut HeaderMap) {
        headers.insert(
            header::LOCATION,
            HeaderValue::from_str(&self.location()).unwrap(),
        );
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MachineCreated {
    pub id: MachineId,
}

impl Response for MachineCreated {
    fn status(&self) -> StatusCode {
        StatusCode::CREATED
    }

    fn set_headers(&self, headers: &mut HeaderMap) {
        headers.insert(
            header::LOCATION,
            HeaderValue::from_str(&format!("/machines/{}", self.id)).unwrap(),
        );
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetMachine<'a> {
    pub binary_id: BinaryId,
    pub params: &'a [u8],
}

impl<'a> Response for GetMachine<'a> {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BinaryDescription {
    pub activations: BTreeMap<String, AcivationDescription>,
}

impl Response for BinaryDescription {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AcivationDescription {
    // I am sure this will map to some kind of metadata in the future
    //empty for now
}

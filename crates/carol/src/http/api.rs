use carol_core::{BinaryId, MachineId};
use hyper::{header, http::HeaderValue, HeaderMap, StatusCode};

#[derive(serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct BinaryCreated {
    pub id: BinaryId,
}

pub trait Response: serde::Serialize + bincode::Encode {
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
            HeaderValue::from_str(&format!("/binaries/{}", self.id)).unwrap(),
        );
    }
}

#[derive(serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
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

#[derive(serde::Serialize, serde::Deserialize, bincode::Encode, bincode::BorrowDecode)]
pub struct GetMachine<'a> {
    pub binary_id: BinaryId,
    pub params: &'a [u8],
}

impl<'a> Response for GetMachine<'a> {}

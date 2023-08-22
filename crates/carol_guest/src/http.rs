use super::*;
use alloc::{string::ToString, vec::Vec};
use bind::carol::machine::http;
pub use http::{Error as BindError, Method, Request, Response};

pub trait Cap {
    fn http_execute(&self, request: Request) -> Result<Response, Error>;
    fn http_get(&self, uri: &str) -> Result<Response, Error> {
        self.http_execute(Request {
            headers: vec![],
            body: vec![],
            method: Method::Get,
            uri: uri.into(),
        })
    }
}

impl From<Method> for http_crate::Method {
    fn from(value: Method) -> Self {
        use Method::*;
        match value {
            Get => http_crate::Method::GET,
            Post => http_crate::Method::POST,
            Put => http_crate::Method::PUT,
            Delete => http_crate::Method::DELETE,
            Patch => http_crate::Method::PATCH,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Error {
    InvalidUrl(String),
    InvalidHeader(String),
    Timeout,
    Connection(String),
    Unexpected(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidUrl(e) => write!(f, "invalid url: {}", e),
            Error::InvalidHeader(e) => write!(f, "invalid header: {}", e),
            Error::Timeout => write!(f, "HTTP request timed out"),
            Error::Connection(e) => write!(f, "HTTP connection failed: {}", e),
            Error::Unexpected(e) => write!(f, "unexpected HTTP error; {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<BindError> for Error {
    fn from(value: BindError) -> Self {
        match value {
            BindError::InvalidUrl(s) => Error::InvalidUrl(s),
            BindError::InvalidHeader(s) => Error::InvalidHeader(s),
            BindError::Timeout => Error::Timeout,
            BindError::Connection(s) => Error::Connection(s),
            BindError::Unexpected(s) => Error::Unexpected(s),
        }
    }
}

impl From<http_crate::Response<Vec<u8>>> for Response {
    fn from(res: http_crate::Response<Vec<u8>>) -> Self {
        Response {
            status: res.status().as_u16(),
            headers: res
                .headers()
                .iter()
                .map(|(key, value)| (key.as_str().to_string(), value.as_bytes().to_vec()))
                .collect(),
            body: res.into_body(),
        }
    }
}

impl Request {
    pub fn uri(&self) -> http_crate::Uri {
        use core::str::FromStr;
        http_crate::Uri::from_str(&self.uri).unwrap()
    }
}

impl Response {
    pub fn status(&self) -> http_crate::StatusCode {
        http_crate::StatusCode::from_u16(self.status)
            .expect("status is valid if this comes from host")
    }
}

impl From<http_crate::Error> for Error {
    fn from(e: http_crate::Error) -> Self {
        use http_crate::{header, uri};
        if e.is::<uri::InvalidUri>() || e.is::<uri::InvalidUriParts>() {
            Error::InvalidUrl(e.to_string())
        } else if e.is::<header::InvalidHeaderName>() || e.is::<header::InvalidHeaderValue>() {
            Error::InvalidHeader(e.to_string())
        } else {
            Error::Unexpected(e.to_string())
        }
    }
}

impl TryFrom<Request> for http_crate::Request<Vec<u8>> {
    type Error = http_crate::Error;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        let mut builder = http_crate::Request::builder()
            .method(req.method)
            .uri(req.uri);

        for (key, value) in req.headers {
            builder = builder.header(key, value);
        }

        builder.body(req.body)
    }
}

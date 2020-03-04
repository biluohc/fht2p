pub mod ctx;
pub mod handler;
pub mod middleware;
mod router;
mod server;
mod service;

pub use router::Router;
pub use server::Server;
pub use service::Service;

pub type Body = hyper::Body;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;
pub type ResponseBuilder = http::response::Builder;

pub use hyper::body::to_bytes as body_to_bytes;

pub use http;
pub fn response() -> ResponseBuilder {
    ResponseBuilder::new()
}

use hyper::{
    header::{AsHeaderName, HeaderValue},
    HeaderMap,
};

pub trait HeaderGetStr {
    fn get_str_option<K: AsHeaderName>(&self, key: K) -> Option<&str>;
    fn get_str<K: AsHeaderName>(&self, key: K) -> &str {
        self.get_str_option(key).unwrap_or_default()
    }
}

impl HeaderGetStr for HeaderMap<HeaderValue> {
    fn get_str_option<K: AsHeaderName>(&self, key: K) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }
}

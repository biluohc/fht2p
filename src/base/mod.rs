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

pub fn response() -> http::response::Builder {
    http::response::Builder::new()
}

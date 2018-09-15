use http::Request as HttpRequest;
use http::Response as HttpResponse;
use http::StatusCode;
use hyper::service::Service;
use hyper::{error, Body, Method};

use futures::{future, Future};

use super::{Request, Response};

use std::net::SocketAddr;

pub struct BaseService {
    peer_addr: SocketAddr,
}

impl BaseService {
    pub fn new(peer_addr: SocketAddr) -> Self {
        BaseService { peer_addr }
    }
}

static INDEX: &'static [u8] = b"Try POST /echo\n";

impl Service for BaseService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = error::Error;
    type Future = Box<Future<Item = HttpResponse<Self::ResBody>, Error = Self::Error> + Send + 'static>;

    fn call(&mut self, req: HttpRequest<Self::ReqBody>) -> Self::Future {
        let _req = Request::new(self.peer_addr, req);
        let addr = self.peer_addr;

        Box::new(future::ok(HttpResponse::new(Body::from(INDEX))).inspect(move |res| info!("[{}]: {}", addr, res.status().as_u16())))
    }
}

use http::Request as HttpRequest;
use http::Response as HttpResponse;
use http::StatusCode;
use hyper::service::Service;
use hyper::{error, Body, Method};

use futures::{future, Future};

use super::{Request, Response};

use std::net::SocketAddr;

pub struct BaseService {
    peer_addr: Option<SocketAddr>,
}

impl BaseService {
    pub fn new(peer_addr: SocketAddr) -> Self {
        let peer_addr = Some(peer_addr);
        BaseService { peer_addr }
    }
}

static INDEX: &'static [u8] = b"Try POST /echo\n";

impl Service for BaseService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = error::Error;
    type Future =
        Box<Future<Item = HttpResponse<Self::ResBody>, Error = Self::Error> + Send + 'static>;

    fn call(&mut self, req: HttpRequest<Self::ReqBody>) -> Self::Future {
        let req = Request::new(self.peer_addr.unwrap(), req);
        
        Box::new(
            future::ok(response)
                .inspect(move |res| println!("[{}]: {}", peer_addr, res.status().as_u16())),
        )
    }
}


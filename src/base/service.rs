use futures::{Future, FutureExt};
use http;
use hyper::{Body, Request, Response};
use tower_service::Service as TowerService;

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{base::Router, service::GlobalState};

pub struct Service {
    pub(crate) peer_addr: SocketAddr,
    pub(crate) state: GlobalState,
}

impl Service {
    pub fn new(peer_addr: SocketAddr, state: GlobalState) -> Self {
        Self { peer_addr, state }
    }
}

impl TowerService<Request<Body>> for Service {
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Router::call(self.peer_addr, req, self.state).boxed()
    }
}

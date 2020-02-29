use futures::{Future, FutureExt};
use hyper::header;
use tower_service::Service as TowerService;

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    base::{http, Request, Response, Router},
    service::GlobalState,
};

pub struct Service {
    pub(crate) peer_addr: SocketAddr,
    pub(crate) state: GlobalState,
}

impl Service {
    pub fn new(peer_addr: SocketAddr, state: GlobalState) -> Self {
        Self { peer_addr, state }
    }
}

#[allow(clippy::type_complexity)]
impl TowerService<Request> for Service {
    type Response = Response;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let keepalive = if self.state.config().keep_alive {
            "Keep-Alive"
        } else {
            "Close"
        };

        Router::call(self.peer_addr, req, self.state)
            .map(move |resp| {
                resp.map(|mut resp| {
                    resp.headers_mut().insert(header::CONNECTION, keepalive.parse().unwrap());
                    resp
                })
            })
            .boxed()
    }
}

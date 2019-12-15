use futures::{future, Future, FutureExt};
use http;
use hyper::{header, Body, Request, Response, StatusCode};
use tokio::{task, time};
use tower_service;

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use crate::service::GlobalState;

pub struct Service {
    pub(crate) peer_addr: SocketAddr,
    pub(crate) state: GlobalState,
}

impl Service {
    pub fn new(peer_addr: SocketAddr, state: GlobalState) -> Self {
        Self { peer_addr, state }
    }
}

impl tower_service::Service<Request<Body>> for Service {
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let start = Instant::now();

        let addr = self.peer_addr;
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        future::ok::<_, String>(200u16)
            .then(move |api| {
                let (code, body) = match &api {
                    Ok(code) => (code % 1000, Body::from("()")),
                    Err(e) => {
                        error!("serde::to_string() failed: {:?}", e);
                        (500, Body::empty())
                    }
                };

                let code = StatusCode::from_u16(code)
                    .map_err(|e| error!("invalid error code: {:?} -> {}", api, e))
                    .unwrap_or_else(|_| StatusCode::from_u16(500).unwrap());

                let targ = format!("{}=x", 1);

                info!(
                    target: &targ,
                    "[{} {:?}]: {} {} {}",
                    addr,
                    start.elapsed(),
                    method,
                    path,
                    code.as_u16()
                );

                future::ready(
                    Response::builder()
                        .status(code)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(body),
                )
            })
            .boxed()
    }
}

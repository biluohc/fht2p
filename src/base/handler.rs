use futures::{future, Future, FutureExt};
use hyper::{header, Body, StatusCode};

use std::{net::SocketAddr, pin::Pin};

use crate::base::ctx::Ctx;
use crate::base::{http, response, Request, Response};

pub type Responder<'a> = Pin<Box<dyn Future<Output = Result<Response, http::Error>> + Send + 'a>>;
pub type Handler = dyn for<'a> Fn(Request, &'a SocketAddr, &'a mut Ctx) -> Responder<'a> + Send + Sync + 'static;

pub type BoxedHandler = Box<Handler>;

pub fn default_handler() -> BoxedHandler {
    Box::new(move |_req: Request, addr: &SocketAddr, _ctx: &mut Ctx| {
        future::ready(
            response()
                .status(StatusCode::from_u16(200).unwrap())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("{{\"socket\": \"{}\"}}", addr))),
        )
        .boxed()
    })
}

pub fn exception_handler<'a>(
    code: u16,
    _req: Request,
    addr: &'a SocketAddr,
    _ctx: &'a mut Ctx,
) -> impl Future<Output = Result<Response, http::Error>> {
    future::ready(
        response()
            .status(StatusCode::from_u16(code).unwrap())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!("{{\"socket\": \"{}\", \"code\": \"{}\" }}", addr, code))),
    )
}

pub fn redirect_handler<S: Into<String>>(
    permanent: bool,
    dest: S,
    _req: Request,
    _addr: &SocketAddr,
    _ctx: &mut Ctx,
) -> impl Future<Output = Result<Response, http::Error>> {
    future::ready(
        response()
            .status(if permanent {
                StatusCode::MOVED_PERMANENTLY
            } else {
                StatusCode::TEMPORARY_REDIRECT
            })
            .header(header::LOCATION, dest.into())
            .body(Body::empty()),
    )
}

use futures::Future;

use std::{net::SocketAddr, pin::Pin};

use crate::base::ctx::Ctx;
use crate::base::{http, Request, Response};

pub type Responder<'a> = Pin<Box<dyn Future<Output = Result<Response, http::Error>> + Send + 'a>>;
pub type Handler = dyn for<'a> Fn(Request, &'a SocketAddr, &'a mut Ctx) -> Responder<'a> + Send + Sync + 'static;

pub type BoxedHandler = Box<Handler>;

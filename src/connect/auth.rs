use chrono::naive::NaiveDateTime;
use chrono::offset::Local;
use failure::Error;
use futures::{future, Future};
use http::uri::Uri;
use http::{self, Request, Response, StatusCode};

use std::net::SocketAddr;

pub fn handle(
    addr: SocketAddr,
    req: Request<()>,
) -> impl Future<Item = (Option<Auth>, Response<&'static str>), Error = Error> + Send + 'static {
    future::ok((
        Some(Auth::new(addr, "dev", "dev-token", req.uri().clone())),
        Response::builder().status(StatusCode::OK).body("").unwrap(),
    ))
}

#[derive(Debug)]
pub struct Auth {
    pub addr: SocketAddr,
    pub user: String,
    pub passwd: String,
    pub uri: Uri,
    pub datetime: NaiveDateTime,
}

impl Auth {
    pub fn new<S: Into<String>>(addr: SocketAddr, user: S, passwd: S, uri: Uri) -> Self {
        let (user, passwd) = (user.into(), passwd.into());
        let datetime = Local::now().naive_local();

        Self {
            addr,
            user,
            passwd,
            uri,
            datetime,
        }
    }
}

use std::fmt;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};

use http::Request as HttpRequest;

pub struct Request<Bd> {
    remote_addr: SocketAddr,
    http_req: HttpRequest<Bd>,
}

impl<Bd> Request<Bd> {
    pub fn new(remote_addr: SocketAddr, http_req: HttpRequest<Bd>) -> Request<Bd> {
        Request {remote_addr, http_req }
    }

    #[inline]
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    pub fn into_inner(self) -> HttpRequest<Bd> {
        self.http_req
    }
}

impl<Bd> Deref for Request<Bd> {
    type Target = HttpRequest<Bd>;
    #[inline]
    fn deref(&self) -> &HttpRequest<Bd> {
        &self.http_req
    }
}

impl<Bd> DerefMut for Request<Bd> {
    #[inline]
    fn deref_mut(&mut self) -> &mut HttpRequest<Bd> {
        &mut self.http_req
    }
}

impl<Bd> fmt::Debug for Request<Bd> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("remote address", &self.remote_addr)
            .field("http request", &"...")
            .finish()
    }
}

use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};

use http::Response as HttpResponse;
use http::request::Parts as RequestHead;

use super::request::Request;

pub struct Response<Bd> {
    http_req: Request<Bd>,
    http_res: HttpResponse<Bd>,
}

impl<Bd> Response<Bd> {
    pub fn new(http_req: Request<Bd>, http_res: HttpResponse<Bd>) -> Response<Bd> {
        Response { http_req, http_res }
    }

    #[inline]
    pub fn remote_addr(&self) -> SocketAddr {
        self.http_req.remote_addr()
    }

    #[allow(dead_code)]
    pub fn into_inner(self) -> HttpResponse<Bd> {
        self.http_res
    }

    pub fn into_parts(self) -> (RequestHead, Bd, HttpResponse<Bd>) {
        let (req_head, body) = self.http_req.into_inner().into_parts();

        (req_head, body, self.http_res)
    }
}

impl<Bd> Deref for Response<Bd> {
    type Target = HttpResponse<Bd>;
    #[inline]
    fn deref(&self) -> &HttpResponse<Bd> {
        &self.http_res
    }
}

impl<Bd> DerefMut for Response<Bd> {
    #[inline]
    fn deref_mut(&mut self) -> &mut HttpResponse<Bd> {
        &mut self.http_res
    }
}

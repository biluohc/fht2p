use askama::Template;
use futures::{future, FutureExt};
use hyper::header;

use http::status::InvalidStatusCode;
use hyper::StatusCode;

use std::borrow::Cow;
use std::convert::TryInto;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;

use crate::base::{ctx::Ctx, handler::BoxedHandler, http, response, Request, Response};
use crate::consts;
use crate::tools::url_for_parent;
use crate::views::ErrorTemplate;

pub fn redirect_handler_sync<S: Into<String>>(permanent: bool, dest: S) -> Result<Response, http::Error> {
    response()
        .status(if permanent {
            StatusCode::MOVED_PERMANENTLY
        } else {
            StatusCode::TEMPORARY_REDIRECT
        })
        .header(header::LOCATION, dest.into())
        .body(Default::default())
}

pub fn notfound_handler() -> BoxedHandler {
    Box::new(move |req: Request, addr: &SocketAddr, _ctx: &mut Ctx| {
        future::ready(exception_handler_sync(404, None, &req, addr)).boxed()
    })
}

pub fn exception_handler_sync<'a, C>(
    code: C,
    msg: Option<&'a str>,
    req: &Request,
    addr: &'a SocketAddr,
) -> Result<Response, http::Error>
where
    C: TryInto<StatusCode, Error = InvalidStatusCode>,
{
    let code = code.try_into().map_err(Into::into);

    code.and_then(|code| {
        let title = msg.map(Cow::Borrowed).unwrap_or_else(|| Cow::Owned(code.to_string()));
        let parent = url_for_parent(req.uri().path());
        let template = ErrorTemplate::new(&title, &title, &parent, addr);
        let html = template.render().unwrap();

        response()
            .status(code)
            .header(header::CONTENT_TYPE, consts::CONTENT_TYPE_HTML)
            .body(html.into())
    })
}

pub fn io_exception_handler_sync<'a>(e: io::Error, req: &Request, addr: &'a SocketAddr) -> Result<Response, http::Error> {
    let code = match e.kind() {
        ErrorKind::NotFound => 404,
        ErrorKind::PermissionDenied => 403,
        _ => 500,
    };

    exception_handler_sync(code, None, &req, addr)
}

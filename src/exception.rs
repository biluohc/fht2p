use hyper::{header, Error, StatusCode};
use hyper::server::{Request, Response};
use hyper_fs::{Exception, ExceptionHandlerService};
use askama::Template;

use views::ErrorTemplate;
use tools::url_for_parent;
use consts;

use std::io::ErrorKind;

#[derive(Default, Debug, Clone)]
pub struct ExceptionHandler;

impl ExceptionHandler {
    pub fn render(code: StatusCode, req: &Request) -> Result<Response, Error> {
        let remote_addr = req.remote_addr().unwrap();
        // if `/` encoding as '%2F', brower will concat origin'path with req_path_parent

        let title = code.to_string();
        let parent = url_for_parent(req.uri().path());
        let template = ErrorTemplate::new(&title, &title, &parent, &remote_addr);
        let html = template.render().unwrap();

        let mut res = Response::new()
            .with_status(code)
            .with_header(header::ContentLength(html.len() as u64))
            .with_body(html);
        res.headers_mut()
            .set_raw(consts::CONTENT_TYPE, consts::HTML_CONTENT_TYPE);
        Ok(res)
    }
}
impl ExceptionHandlerService for ExceptionHandler {
    fn call<E>(e: E, req: Request) -> Result<Response, Error>
    where
        E: Into<Exception>,
    {
        use self::Exception::*;
        let code = match e.into() {
            Io(i) => match i.kind() {
                ErrorKind::NotFound => StatusCode::NotFound,
                ErrorKind::PermissionDenied => StatusCode::Forbidden,
                _ => StatusCode::InternalServerError,
            },
            Method => StatusCode::MethodNotAllowed,
            Typo | Route => StatusCode::InternalServerError,
        };
        Self::render(code, &req)
    }
}

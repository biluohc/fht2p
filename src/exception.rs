use hyper::{header, Error, StatusCode};
use hyper::server::{Request, Response};
use hyper_fs::{Exception, ExceptionHandlerService};
use url::percent_encoding::{percent_decode, percent_encode_byte};

use consts;

use std::io::ErrorKind;
use std::env;

/// Default `ExceptionHandler`
///
/// You can impl `ExceptionHandlerService` for your owner type, and use it by `with_handler`.
#[derive(Default, Debug, Clone)]
pub struct ExceptionHandler;

impl ExceptionHandler {
    pub fn render(code: StatusCode, req: &Request) -> Result<Response, Error> {
        let remote_addr = req.remote_addr().unwrap();
        let req_path_dec = percent_decode(req.path().as_bytes()).decode_utf8().unwrap();
        let cow_str = req_path_dec.as_ref();
        let req_path_components = cow_str
            .split('/')
            .filter(|c| !c.is_empty())
            .collect::<Vec<_>>();
        // if `/` encoding as '%2F', brower will concat origin'path with req_path_parent
        let req_path_parent = if req_path_components.len() > 1 {
            req_path_components[..req_path_components.len() - 1]
                .iter()
                .flat_map(|c| {
                    "/".chars()
                        .chain(c.bytes().flat_map(|cc| percent_encode_byte(cc).chars()))
                })
                .collect::<String>()
        } else {
            "/".to_owned()
        };

        debug!("{:?}", req_path_parent);
        let html = format!(
            "<!DOCTYPE html><html><head>
<meta charset=\"UTF-8\">
<style type=\"text/css\">{}</style> 
<title>{}</title></head>
<body><h1><span id=\"client\">{}{}:{}{}</span><a href=\"{}\">{}</a></h1></body>
<address><a href=\"{}\">{}/{}</a>({}/{}) server at <a href=\"/\">{}:{}</a></address></html>",
            consts::CSS,
            &code,
            consts::SPACEHOLDER.repeat(8),
            remote_addr.ip(),
            remote_addr.port(),
            consts::SPACEHOLDER.repeat(8),
            req_path_parent,
            &code,
            consts::URL,
            consts::NAME,
            env!("CARGO_PKG_VERSION"),
            env::consts::OS,
            env::consts::ARCH,
            consts::SERVER_ADDR.get().ip(),
            consts::SERVER_ADDR.get().port()
        );
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

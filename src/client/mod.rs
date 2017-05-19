use std::collections::HashMap as Map;
use std::net::TcpStream;
// use std::fs::File;
use std::io;

use super::{statics, Time};
use self::statics::Route;

/// `ContentType`
pub mod content_type;
mod status_code;
pub use self::status_code::StatusCode;
mod req;
pub use self::req::*;
mod resp;
pub use self::resp::*;

/// `HTTP`
pub const HTTP_PROTOCOL: &'static str = "HTTP";
/// `1.1`
pub const HTTP_VERSION: &'static str = "1.1";

/// `Request` and `Response`
#[derive(Debug)]
pub struct Client {
    pub req: Request,
    pub resp: Response,
}

impl Client {
    pub fn req(&self) -> &Request {
        &self.req
    }
    pub fn resp(&self) -> &Response {
        &self.resp
    }
    pub fn method_call(mut self, mut stream: &mut TcpStream) {
        match self.req.line().method() {
            "GET" | "HEAD" => self.resp.get(&self.req),
            s => {
                errln!("Method don't support: {:?}", s);
                self.resp.code_set(405_u16);
            }
        };
        // 127.0.0.1:50822**[Thu, 18 May 2017 17:10:31] 200 "GET /fht2p.css HTTP/1.1" -> "/fht2p.css"
        let path = if let Some(r) = self.req.route() {
            if *r.is_sfs() {
                format!(" -> {:?}", r.img())
            } else {
                format!(" -> {:?}", r.rel())
            }
        } else {
            String::new()
        };
        println!(r#"[{}**{}] {} "{} {} {}/{}"{}"#,
                 self.req.client_addr(),
                 self.req.time().hms(),
                 self.resp.line().code().code(),
                 self.req.line().method(),
                 self.req.line().path(),
                 self.resp.line().protocol(),
                 self.resp.line().version(),
                 path);
        self.resp.call(&mut stream, &self.req);
    }
}

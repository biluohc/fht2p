use base64::{decode_config, URL_SAFE};
use hyper::header;

use std::{net::SocketAddr, str};

use crate::{
    base::{ctx::Ctx, middleware::MiddleWare, response, Request, Response},
    config::Auth,
};

#[derive(Debug, Clone)]
pub struct Authenticator {
    auth: Auth,
}

impl Authenticator {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }
}

// HTTP/1.0 401 Authorization Required
// WWW-Authenticate: Basic realm="Secure Area"

// Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==
// tips: base64encode(Aladdin:open sesame)=QWxhZGRpbjpvcGVuIHNlc2FtZQ==

impl MiddleWare for Authenticator {
    fn before(&self, req: &Request, _addr: &SocketAddr, _ctx: &mut Ctx) -> Result<(), Response> {
        // info!("url: {:?}", req.uri());
        // info!("header: {:?}", req.headers());

        let f = move |code, desc: &'static str| {
            response()
                .status(code)
                .header(header::WWW_AUTHENTICATE, "Basic realm=\"User:Password\"")
                .body(desc.into())
                .unwrap()
        };

        let www = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        let auth = match www_base64_to_auth(www) {
            Ok(a) => a,
            Err(e) => return Err(f(401, e)),
        };

        debug!("www: {}, auth: {:?}", www, auth);

        if auth != self.auth {
            return Err(f(401, "wrong username or password"));
        }

        Ok(())
    }
}

fn www_base64_to_auth(value: &str) -> Result<Auth, &'static str> {
    let value = value.trim();

    if value.len() <= "basic".len() || value[0.."basic".len()].to_lowercase() != "basic" {
        return Err("not basic");
    }

    let value = (&value["basic".len()..]).trim();

    decode_config(value, URL_SAFE)
        .map_err(|_| "invalid basic value")
        .and_then(|bs| String::from_utf8(bs).map_err(|_| "invalid string"))
        .and_then(|str| str.parse().map_err(|_| "invalid authorised infomation"))
}

use std::{fs, net::SocketAddr, path::Path, str};

use crate::base::{body_to_bytes, http, response, Body, Request, Response};
use crate::config::Route;
use crate::service::GlobalState;

// .to_lowercase()
pub fn method_maybe_mkdir(req: &Request) -> bool {
    req.uri()
        .path_and_query()
        .and_then(|pq| pq.query().map(|q| q.ends_with("method=mkdir")))
        .unwrap_or(false)
}

// text-plain X: ascii only
// application/x-www-urlencoded
// multipart/form-data  X

// curl "0.0.0.0:9000/?method=mkdir" -d "mkdir=new%E5%B0%8Ffile"
// curl "0.0.0.0:9000/?method=mkdir" --data "mkdir=new%E5%B0%8Ffile" -X POST
pub async fn mkdir_handler<'a>(
    _route: &'a Route,
    _reqpath: &'a str,
    path: &'a Path,
    req: Request,
    addr: &'a SocketAddr,
    _state: GlobalState,
) -> Result<Response, http::Error> {
    let f = |code: u16, s: &'static str| response().status(code).body(s.into());

    let (_parts, body) = req.into_parts();

    let mkdir = match body_to_mkdir(body).await {
        Ok(m) => m,
        Err(e) => return f(400, e),
    };

    let mkdirp = path.join(&mkdir);
    warn!("{} mkdir {} -> {}", addr, mkdir, mkdirp.display());

    if mkdirp.exists() {
        return f(400, "File exists");
    }

    if let Err(e) = fs::create_dir(mkdirp) {
        warn!("{} mkdir {} failed: {:?}", addr, mkdir, e);
        return f(500, "create_dir error");
    }

    f(200, "mkdir ok")
}

async fn body_to_mkdir(body: Body) -> Result<String, &'static str> {
    let body = match body_to_bytes(body).await {
        Ok(b) => b,
        Err(_) => return Err("request eof early"),
    };

    let text = str::from_utf8(body.as_ref()).map_err(|_| "invalid string")?;

    str_to_mkdir(text)
}

fn str_to_mkdir(input: &str) -> Result<String, &'static str> {
    // let kvs = serde_urlencoded::from_str::<Vec<(&str, String)>>(input).map_err(|_| "invalid form")?;
    // kvs.into_iter()
    //     .find_map(|(k, v)| if k == "mkdir" { Some(v) } else { None })
    //     .ok_or("invalid form")

    serde_urlencoded::from_str::<MkdirForm>(input)
        .map_err(|_| "invalid form")
        .and_then(|s| {
            if s.mkdir.trim().is_empty() || s.mkdir.contains('/') || s.mkdir.contains('\\') {
                Err("invalid dirname")
            } else {
                Ok(s.mkdir)
            }
        })
}

#[test]
fn str_to_mkdir_test() {
    assert!(str_to_mkdir("mkdir=kl%2Fl").is_err());
    assert!(str_to_mkdir("mkdir=kl%5C%5Ca").is_err());
    assert_eq!(str_to_mkdir("mkdir=newfile").unwrap(), "newfile");
    assert_eq!(str_to_mkdir("mkdir=new%20file").unwrap(), "new file");
    assert_eq!(str_to_mkdir("mkdir=new%E5%B0%8Ffile").unwrap(), "new小file");
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MkdirForm {
    mkdir: String,
}

impl MkdirForm {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self { mkdir: s.into() }
    }
}

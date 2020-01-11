use chrono::{offset::Local, DateTime};
use http;
use hyper::{header, Body, Method, Request, Response, StatusCode};
use tokio::task;

use std::{
    fs::{self, File},
    io,
    net::SocketAddr,
    path::Path,
};

use super::ranges::{RangesForm, RangesResp};
use super::send::send_resp;
use crate::base::ctx::ctxs;
use crate::config::Route;
use crate::contentype::guess_contentype;
use crate::service::GlobalState;

pub async fn file_handler<'a>(
    route: &'a Route,
    reqpath: &'a str,
    path: &'a Path,
    meta: &'a fs::Metadata,
    req: Request<Body>,
    addr: &'a SocketAddr,
    state: GlobalState,
) -> Result<Response<Body>, http::Error> {
    match file_handler2(route, reqpath, path, meta, state, req, addr) {
        Ok(resp) => resp,
        Err(e) => {
            error!("file_handler2 faield: {:?}", e);
            Response::builder().status(500).body(Body::empty())
        }
    }
}

// https://github.com/rust-lang/rust/issues/59001
pub fn file_handler2(
    _route: &Route,
    _reqpath: &str,
    path: &Path,
    meta: &fs::Metadata,
    state: ctxs::State,
    req: Request<Body>,
    addr: &SocketAddr,
) -> io::Result<Result<Response<Body>, http::Error>> {
    let mut resp = Response::builder();
    let cache_secs = state.config().cache_secs;

    if cache_secs > 0 {
        let last_modified = meta.modified()?;
        let last_modified: DateTime<Local> = last_modified.into();
        let http_last_modified = last_modified.to_rfc2822();

        // W/"80-5d564a70.3797f8b1"
        let etag = format!(
            "W/\"{:x}-{:x}.{:x}\"",
            meta.len(),
            last_modified.timestamp_millis(),
            last_modified.timestamp_subsec_nanos(),
        );

        let http_etag = req
            .headers()
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        let http_if_modified_since = req.headers().get(header::IF_MODIFIED_SINCE);
        let if_modified_since = http_if_modified_since
            .and_then(|v| v.to_str().ok())
            .and_then(|v| DateTime::parse_from_rfc2822(v).ok())
            .map(|v| v.with_timezone(&Local));

        if etag.as_str() == http_etag
            || if_modified_since
                .map(|v| v.timestamp() <= last_modified.timestamp())
                .unwrap_or_default()
        {
            // 304
            return Ok(resp.status(StatusCode::NOT_MODIFIED).body(Body::empty()));
        }
        resp = resp.header(header::CACHE_CONTROL, format!("public, max-age={}", cache_secs).as_str());
        resp = resp.header(header::LAST_MODIFIED, http_last_modified);
        resp = resp.header(header::ETAG, etag);
    }

    let mut file = File::open(path)?;
    let mut contentype = guess_contentype(&mut file, meta, path)?;

    let rangestr = req.headers().get(header::RANGE).and_then(|v| v.to_str().ok());

    let (rangesform, contentlen, contentrange) = if let Some(rangestr) = rangestr {
        match rangestr
            .parse::<RangesForm>()
            .and_then(|mut rf| rf.build(meta.len(), &mut contentype).map(|(cl, cr)| (rf, cl, cr)))
        {
            Ok(rcc) => rcc,
            // 416
            Err(e) => {
                error!("{} Ranges error: {:?}", addr, e);
                return Ok(resp.status(StatusCode::RANGE_NOT_SATISFIABLE).body(Body::empty()));
            }
        }
    } else {
        // 200
        let contentlen = meta.len();
        let rangesform: RangesForm = contentlen.into();
        (rangesform, contentlen, String::new())
    };

    if !contentrange.is_empty() {
        resp = resp.header(header::CONTENT_RANGE, contentrange);
    }
    resp = resp.header(header::CONTENT_LENGTH, contentlen);
    resp = resp.header(header::CONTENT_TYPE, contentype);
    resp = resp.header(header::ACCEPT_RANGES, "bytes");

    debug!("{}'s ranges str: {:?}, form: {:?}", addr, rangestr, rangesform);

    match *req.method() {
        Method::GET => {
            let code = if rangesform.is_partail() {
                StatusCode::PARTIAL_CONTENT
            } else {
                StatusCode::OK
            };
            let body = if contentlen > 0 {
                let (sender, body) = Body::channel();
                task::spawn(send_resp(RangesResp::new(rangesform, file), sender, *addr));
                body
            } else {
                Body::empty()
            };

            Ok(resp.status(code).body(body))
        }
        // 204： curl -Lv -X HEAD "0.0.0.0:8000/src/main.rs"
        Method::HEAD => Ok(resp.status(StatusCode::NO_CONTENT).body(Body::empty())),
        _ => unreachable!(),
    }
}

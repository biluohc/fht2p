use http;
use hyper::{Body, StatusCode};

use std::{
    // fs::{self, File},
    // io,
    net::SocketAddr,
    path::Path,
};

// use crate::base::ctx::ctxs;
use crate::base::{response, Request, Response};
use crate::config::Route;
use crate::service::GlobalState;

use super::multipart::MultiPart;

// curl  -F "filename=@pkg.jl" 0.0.0.0:8000/src/up.jl
// curl  -F "filename=@pkg.jl" -F "filename=@pkg.jl"  0.0.0.0:8000/src/up.jl
// curl  -F "filename=@pkg.jl" -F "filename=@log.jl"  0.0.0.0:8000/src/up.jl
pub async fn file_upload_handler<'a>(
    route: &'a Route,
    reqpath: &'a str,
    path: &'a Path,
    req: Request,
    addr: &'a SocketAddr,
    state: GlobalState,
) -> Result<Response, http::Error> {
    info!(
        "{}'s reqpath: {}, path: {}, header: {:?}",
        addr,
        reqpath,
        path.display(),
        req.headers()
    );

    let (parts, body) = req.into_parts();

    let f = |code: u16, s: &'static str| response().status(code).body(s.into());

    let mut ps = match MultiPart::new(body, &parts.headers) {
        Ok(ps) => ps,
        Err(e) => {
            error!("MultiPart::new(body, &parts.headers): {:?}", e);
            return f(400, "MultiPart::new");
        }
    };

    'w0: while let Some(part) = ps.next_part().await {
        match part {
            Ok(mut part) => {
                info!("part: {}, {}", part.filename(), part.contentype());
                while let Some(chunk) = part.next_chunk().await {
                    match chunk {
                        Ok(bytes) => info!("{}", std::str::from_utf8(bytes.as_ref()).unwrap()),
                        Err(e) => {
                            error!("chunk error: {:?}", e);
                            break 'w0;
                        }
                    }
                }
            }
            Err(e) => {
                error!("nextpart: {:?}", e);
                break;
            }
        }
    }

    response().status(200).body(Body::empty())
}

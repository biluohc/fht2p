use bytesize::ByteSize;
use std::{net::SocketAddr, path::Path};

// use crate::base::ctx::ctxs;
use crate::base::{http, response, Request, Response};
use crate::config::Route;
use crate::service::GlobalState;

use super::multipart::MultiPart;

// curl  -F "filename=@pkg.jl" 0.0.0.0:8000/src/up.jl
// curl  -F "filename=@pkg.jl" -F "filename=@pkg.jl"  0.0.0.0:8000/src/up.jl
// curl  -F "filename=@pkg.jl" -F "filename=@log.jl"  0.0.0.0:8000/src/up.jl
pub async fn file_upload_handler<'a>(
    _route: &'a Route,
    _reqpath: &'a str,
    path: &'a Path,
    req: Request,
    addr: &'a SocketAddr,
    _state: GlobalState,
) -> Result<Response, http::Error> {
    let (parts, body) = req.into_parts();

    let f = |code: u16, s: &'static str| response().status(code).body(s.into());

    let mut ps = match MultiPart::new(body, &parts.headers) {
        Ok(ps) => ps,
        Err(e) => {
            error!("MultiPart::new() failed: {:?}", e);
            return f(400, "MultiPart parse");
        }
    };

    while let Some(part) = ps.next_part().await {
        match part {
            Ok(mut part) => {
                let file = path.join(part.filename());
                warn!(
                    "addr: {}, part.filename: {}, conetentype: {}, will save to {}",
                    addr,
                    part.filename(),
                    part.contentype(),
                    file.display()
                );

                match part.save(file.as_path()).await {
                    Ok(writec) => {
                        let size = ByteSize::b(writec).to_string_as(true);
                        warn!(
                            "addr: {}, part.filename: {}, conetentype: {}, saved to {} ok: {}",
                            addr,
                            part.filename(),
                            part.contentype(),
                            file.display(),
                            size
                        );
                    }
                    Err(e) => {
                        warn!(
                            "addr: {}, part.filename: {}, conetentype: {}, saved to {} error: {:?}",
                            addr,
                            part.filename(),
                            part.contentype(),
                            file.display(),
                            e
                        );

                        return f(400, "upload save error");
                    }
                }
            }
            Err(e) => {
                error!("{} nextpart error: {:?}", addr, e);
                return f(400, "upload nextpart error");
            }
        }
    }

    f(200, "upload ok")
}

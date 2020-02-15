use flate2::{
    write::{DeflateEncoder, GzEncoder},
    Compression,
};
use hyper::header;
use tokio::task::spawn_blocking;

use std::io::{self, Write};
use std::net::SocketAddr;

use super::exception::exception_handler_sync;
use crate::base::{http, Body, Request, Response, ResponseBuilder};

// bitflags! {
//     pub struct Algorithms: u8 {
//         const NONE =  0b00000000;
//         const DEFLATE = 0b10000000;
//         const GZIP =  0b01000000;
//         const SDCN =0b00100000;
//         const BR = 0b00010000;
//         const ZSTD = 0b00001000;
//     }
// }

pub struct Compressor {
    algorithm: &'static str,
    compress_level: Compression,
}

const GZIP: &str = "gzip";
const DEFLATE: &str = "deflate";

impl Compressor {
    pub fn algorithm(&self) -> &'static str {
        self.algorithm
    }
    pub fn new<S>(accept_encodings_str: S, compress_level: u32) -> Result<Self, S>
    where
        S: AsRef<str>,
    {
        let compress_level = Compression::new(compress_level);

        // Algorithm priority is selected by the browser
        let mut algorithm = "";
        for value in accept_encodings_str.as_ref().split(',') {
            let algs = value.trim();
            match algs {
                GZIP => {
                    algorithm = GZIP;
                    break;
                }
                DEFLATE => {
                    algorithm = DEFLATE;
                    break;
                }
                _ => {}
            }
        }

        if algorithm.is_empty() {
            Err(accept_encodings_str)
        } else {
            Ok(Self {
                algorithm,
                compress_level,
            })
        }
    }

    pub fn compress<I, W>(&self, input: I, writer: W) -> io::Result<W>
    where
        I: AsRef<[u8]>,
        W: Write,
    {
        match self.algorithm {
            GZIP => {
                let mut encoder = GzEncoder::new(writer, self.compress_level);
                encoder.write_all(input.as_ref())?;
                encoder.finish()
            }
            DEFLATE => {
                let mut encoder = DeflateEncoder::new(writer, self.compress_level);
                encoder.write_all(input.as_ref())?;
                encoder.finish()
            }
            _ => unreachable!(),
        }
    }
}

pub async fn compress_handler<B>(
    req: &Request,
    addr: &SocketAddr,
    mut resp: ResponseBuilder,
    body: B,
    compress_level: u32,
) -> Result<Response, http::Error>
where
    B: Into<Body> + AsRef<[u8]> + Send + 'static,
{
    let bodysize = body.as_ref().len();

    if compress_level == 0 || bodysize <= 32 {
        return resp.body(body.into());
    }

    let accept_encodings_str = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    match Compressor::new(accept_encodings_str, compress_level) {
        Ok(compressor) => {
            let algs = compressor.algorithm();
            let res = spawn_blocking(move || compressor.compress(&body, Vec::new()).map_err(move |e| (e, body))).await;

            match res {
                Ok(Ok(compressed)) => {
                    resp = resp.header(header::CONTENT_ENCODING, algs);
                    resp.headers_mut().and_then(|header| header.remove(header::CONTENT_LENGTH));
                    resp = resp.header(header::CONTENT_LENGTH, compressed.len());

                    debug!(
                        "body {}/{}: {}, header: {:?}",
                        compressed.len(),
                        bodysize,
                        (compressed.len() as f64 / bodysize as f64),
                        resp.headers_ref().unwrap()
                    );

                    resp.body(compressed.into())
                }
                Ok(Err((e, body))) => {
                    error!("addr: {}, bodysize: {}, compress failed: {}", addr, bodysize, e);
                    resp.body(body.into())
                }
                Err(e) => {
                    error!("addr: {}, bodysize: {}, spawn_blocking failed: {}", addr, bodysize, e);
                    exception_handler_sync(500, None, req, addr)
                }
            }
        }
        Err(_) => resp.body(body.into()),
    }
}

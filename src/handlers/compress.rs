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
use crate::how;

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

pub trait CompressorTrait<W: Write, I: AsRef<[u8]>> {
    fn compress(self) -> io::Result<W>;
}

impl<W: Write + Default, I: AsRef<[u8]>> CompressorTrait<W, I> for Compressor<I> {
    fn compress(self) -> io::Result<W> {
        match self.algorithm {
            GZIP => {
                let mut encoder = GzEncoder::new(W::default(), self.compress_level);
                encoder.write_all(self.input.as_ref())?;
                encoder.finish()
            }
            DEFLATE => {
                let mut encoder = DeflateEncoder::new(W::default(), self.compress_level);
                encoder.write_all(self.input.as_ref())?;
                encoder.finish()
            }
            _ => unreachable!(),
        }
    }
}

pub struct Compressor<I: AsRef<[u8]>> {
    input: I,
    algorithm: &'static str,
    compress_level: Compression,
}

const GZIP: &str = "gzip";
const DEFLATE: &str = "deflate";

impl<I: AsRef<[u8]>> Compressor<I> {
    pub fn algorithm(&self) -> &'static str {
        self.algorithm
    }
    pub fn new<S: AsRef<str>>(input: I, accept_encodings_str: S, compress_level: u32) -> Result<Self, I> {
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
            Err(input)
        } else {
            Ok(Self {
                input,
                algorithm,
                compress_level,
            })
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
    let accept_encodings_str = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let bodysize = body.as_ref().len();
    match Compressor::new(body, accept_encodings_str, compress_level) {
        Ok(compressor) => {
            let algs = compressor.algorithm();
            let res = spawn_blocking(move || compressor.compress().map_err(Into::into))
                .await
                .map_err(Into::into)
                .and_then(|res: how::Result<Vec<u8>>| res);

            match res {
                Ok(compressed) => {
                    resp = resp.header(header::CONTENT_ENCODING, algs);
                    resp.headers_mut().and_then(|header| header.remove(header::CONTENT_LENGTH));
                    resp = resp.header(header::CONTENT_LENGTH, compressed.len());

                    info!(
                        "body {}/{}: {}, header: {:?}",
                        compressed.len(),
                        bodysize,
                        (compressed.len() as f64 / bodysize as f64),
                        resp.headers_ref().unwrap()
                    );

                    resp.body(compressed.into())
                }
                Err(e) => {
                    error!(
                        "addr: {}, bodysize: {}, spawn_blocking(compressor.compress()) failed: {}",
                        addr, bodysize, e
                    );
                    exception_handler_sync(500, None, req, addr)
                }
            }
        }
        Err(body) => resp.body(body.into()),
    }
}

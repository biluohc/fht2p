use futures::future;
use futures::Stream;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use rustls::internal::pemfile;
use rustls;
use std::{env, fs, io, sync};
use tokio_rustls::ServerConfigExt;
use tokio_tcp;
use tokio_core;

use args::{Config, Route};

static INDEX: &'static [u8] = b"Try POST /echo\n";

type ResponseFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn echo(req: Request<Body>) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from(INDEX);
        }
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Box::new(future::ok(response))
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = io::BufReader::new(certfile);
    pemfile::certs(&mut reader).unwrap()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = io::BufReader::new(keyfile);
    let keys = pemfile::rsa_private_keys(&mut reader).unwrap();
    assert!(keys.len() == 1);
    keys[0].clone()
}

use std;
pub fn run(_: Config) -> std::io::Result<()> {
    let port = match env::args().nth(1) {
        Some(ref p) => p.to_owned(),
        None => "8080".to_owned(),
    };
    let addr = format!("127.0.0.1:{}", port).parse().unwrap();

    let tls_cfg = {
        let certs = load_certs("/home/mxo/Documents/git/github/fht2p/cert/rsa_public_key.pem");
        let key = load_private_key("/home/mxo/Documents/git/github/fht2p/cert/rsa_private_key.pem");
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        cfg.set_single_cert(certs, key);
        sync::Arc::new(cfg)
    };

    let tcp = tokio_tcp::TcpListener::bind(&addr)?;
    println!("Starting to serve on https://{}.", addr);
    let tls = tcp.incoming().and_then(|s| tls_cfg.accept_async(s));
    let fut = Server::builder(tls).serve(|| service_fn(echo));

    let mut core = tokio_core::reactor::Core::new()?;
    core.run(fut).unwrap();
    Ok(())
}

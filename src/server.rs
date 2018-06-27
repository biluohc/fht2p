use futures::future;
use futures::Stream;
use hyper::rt::Future;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use rustls;
use rustls::internal::pemfile;
use std::{env, fs, io, sync};
use tokio_core;
use tokio_rustls::{self, ServerConfigExt};
use tokio_tcp;

use num_cpus;
use tokio::runtime::current_thread;
use tokio::runtime::Builder as RuntimeBuilder;
use tokio_threadpool::Builder as ThreadPoolBuilder;

use std;
use std::error::Error;

use args::{Config, Route};
use base::BaseService;

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

pub fn run(_: Config) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", "8081").parse().unwrap();

    let tls_cfg = {
        let certs = load_certs("/home/mxo/Documents/git/github/fht2p/cert/rsa_public_key.pem");
        let key = load_private_key("/home/mxo/Documents/git/github/fht2p/cert/rsa_private_key.pem");
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        cfg.set_single_cert(certs, key);
        sync::Arc::new(cfg)
    };

    let http = Http::new();

    let pool_size = num_cpus::get();
    let mut threadpool_builder = ThreadPoolBuilder::new();
    threadpool_builder.name_prefix("fht2p-worker-");
    threadpool_builder.pool_size(pool_size);
    threadpool_builder.max_blocking(2);
    let mut runtime = RuntimeBuilder::new()
        .threadpool_builder(threadpool_builder)
        .build()
        .expect("failed to create event loop and thread pool");

    let executor_for_tcp_listener = runtime.executor();

    let tcp = tokio_tcp::TcpListener::bind(&addr)?;
    println!("Starting to serve on https://{}.", addr);
    let tcp_listener_module = tcp
        .incoming()
        .and_then(move |s| tls_cfg.accept_async(s))
        .map(|s| Some(s))
        .or_else(|e| {
            error!("error in accepting connection: {:?}", e.description());
            let tmp: Option<
                tokio_rustls::TlsStream<tokio_tcp::TcpStream, rustls::ServerSession>,
            > = None;
            future::ok::<_, std::io::Error>(tmp)
        })
        .filter_map(|s| s)
        // already or_else..
        .map_err(|e| unreachable!(e))
        .for_each(move |s| {
            let remote_addr = s.get_ref().0.peer_addr().unwrap();
            let connection = http.serve_connection(s, BaseService::new(remote_addr));

            executor_for_tcp_listener.spawn(connection.then(move |connection_res| {
                if let Err(e) = connection_res {
                    error!("client[{}]: {}", remote_addr, e.description());
                }
                Ok(())
            }));
            Ok(())
        });

    runtime.spawn(tcp_listener_module);
    // current_thread_runtime.spawn(log_module);
    // current_thread_runtime.run().unwrap();
    runtime.shutdown_on_idle().wait().unwrap();
    Ok(())
}

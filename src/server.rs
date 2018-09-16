use futures::future;
use futures::Stream;
use hyper::rt::Future;
use hyper::server::conn::Http;
// use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use failure::Error;
use rustls;
use rustls::internal::pemfile;
use tokio::net as tcp;
use tokio_rustls::{self, ServerConfigExt};

use tokio::runtime::current_thread;
use tokio::runtime::Builder as RuntimeBuilder;
use tokio::runtime::TaskExecutor;
use tokio_threadpool::Builder as ThreadPoolBuilder;

use std;
use std::net::SocketAddr;
use std::{fs, io, sync, thread, time};
use StdError;

use base::BaseService;
use config::{Config, Route};
use reuse::reuse_address;
use stat::print as stat_print;

pub fn run(config: Config) -> Result<(), Error> {
    let mut threadpool_builder = ThreadPoolBuilder::new();
    threadpool_builder.name_prefix("fht2p-worker-");

    let runtime = RuntimeBuilder::new().threadpool_builder(threadpool_builder).build()?;

    let executor = runtime.executor();

    let mut tcp_listener: Option<Box<dyn Future<Item = (), Error = ()> + 'static>> = None;
    let mut error = None;

    for addr in config.addrs.clone() {
        match tcp_listener_module(&addr, config.clone(), executor.clone()) {
            Ok(tcp) => {
                tcp_listener = Some(tcp);
                stat_print(
                    &addr,
                    if config.cert.is_some() { "https" } else { "http" },
                    config.routes.values().cloned().collect(),
                );
                break;
            }
            Err(e) => {
                warn!("listen {}: {}", addr, e);
                error = Some(e);
            }
        }
    }

    if tcp_listener.is_none() {
        if let Some(e) = error {
            return Err(e);
        } else {
            unreachable!("tcp-listener and error is None");
        }
    }

    let mut current_thread_runtime = current_thread::Runtime::new()?;
    current_thread_runtime.spawn(tcp_listener.unwrap());
    current_thread_runtime.run()?;
    Ok(())
}

pub fn load_certs(filename: &str) -> Result<Vec<rustls::Certificate>, Error> {
    let certfile = fs::File::open(filename).map_err(|e| format_err!("open certificate file({}) failed: {:?}", filename, e))?;
    let mut reader = io::BufReader::new(certfile);
    pemfile::certs(&mut reader).map_err(|e| format_err!("load certificate({}) failed: {:?}", filename, e))
}

pub fn load_private_key(filename: &str) -> Result<rustls::PrivateKey, Error> {
    let keyfile = fs::File::open(filename).map_err(|e| format_err!("open private key file({}) failed: {:?}", filename, e))?;
    let mut reader = io::BufReader::new(keyfile);
    let keys =
        pemfile::rsa_private_keys(&mut reader).map_err(|e| format_err!("load private key({}) failed: {:?}", filename, e))?;
    assert!(keys.len() == 1);
    Ok(keys[0].clone())
}

pub fn tcp_listener_module(
    addr: &SocketAddr,
    config: Config,
    executor: TaskExecutor,
) -> Result<Box<dyn Future<Item = (), Error = ()> + 'static>, Error> {
    let tcp = reuse_address(&addr)?;
    let http = Http::new();

    if let Some(cert) = &config.cert {
        let tls_cfg = {
            let certs = load_certs(&cert.pub_)?;
            let key = load_private_key(&cert.key)?;
            let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
            cfg.set_single_cert(certs, key)
                .map_err(|e| format_err!("set single cert failed: {:?}", e))?;
            sync::Arc::new(cfg)
        };

        let tcp_listener_module = tcp
            .incoming()
            .and_then(move |socket| tls_cfg.accept_async(socket))
            .and_then(|socket| socket.get_ref().0.peer_addr().map(|addr| Some((addr, socket))))
            .or_else(|e| {
                tcp_listener_sleep_ms(e, 1);

                let tmp: Option<(SocketAddr, tokio_rustls::TlsStream<tcp::TcpStream, rustls::ServerSession>)> = None;
                future::ok::<_, std::io::Error>(tmp)
            }).filter_map(|s| s)
            // already or_else..
            .map_err(|e| unreachable!(e))
            .for_each(move |(addr, socket)| {
                let connection = http.serve_connection(socket, BaseService::new(addr));

                executor.spawn(connection.then(move |connection_res| {
                    if let Err(e) = connection_res {
                        error!("client[{}]: {}", addr, e.description());
                    }
                    Ok(())
                }));
                Ok(())
            });
        Ok(Box::new(tcp_listener_module) as _)
    } else {
        let tcp_listener_module = tcp
            .incoming()
            .and_then(|socket| socket.peer_addr().map(|addr| Some((addr, socket))))
            .or_else(|e| {
                tcp_listener_sleep_ms(e, 1);

                let tmp: Option<(SocketAddr, tcp::TcpStream)> = None;
                future::ok::<_, std::io::Error>(tmp)
            }).filter_map(|s| s)
            // already or_else..
            .map_err(|e| unreachable!(e))
            .for_each(move |(addr, socket)| {
                let connection = http.serve_connection(socket, BaseService::new(addr));

                executor.spawn(connection.then(move |connection_res| {
                    if let Err(e) = connection_res {
                        error!("client[{}]: {}", addr, e.description());
                    }
                    Ok(())
                }));

                Ok(())
            });
        Ok(Box::new(tcp_listener_module) as _)
    }
}

fn tcp_listener_sleep_ms<T: StdError>(error: T, ms: u64) {
    error!(
        "error in accepting connection: {}, tcp_listener will sleep {} ms",
        error.description(),
        ms
    );
    thread::sleep(time::Duration::from_millis(ms));
}

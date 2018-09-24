use futures::{future, Async, Future, Poll, Stream};
use hyper::server::conn::Http;
// use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use failure::Error;
use futures::future::Either;
use rustls;
use rustls::internal::pemfile;
use tokio::net as tcp;
use tokio_rustls::{self, ServerConfigExt};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::runtime::current_thread;
use tokio::runtime::Builder as RuntimeBuilder;
use tokio::runtime::TaskExecutor;
use tokio_threadpool::Builder as ThreadPoolBuilder;

use std::net::SocketAddr;
use std::{fs, io, mem, str, sync, thread, time};
use StdError;

use base::BaseService;
use config::{Config, Route};
use connect;
use reuse::reuse_address;
use stat::print as stat_print;

pub fn run(config: Config) -> Result<(), Error> {
    let mut threadpool_builder = ThreadPoolBuilder::new();
    threadpool_builder.name_prefix("worker-");

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
            .and_then(|socket| socket.peer_addr().map(|addr| (addr, socket)))
            .and_then(move |(addr, socket)| tls_cfg.accept_async(socket).map(move |socket| (addr, socket)));

        Ok(Box::new(sockets_stream_handle(tcp_listener_module, http, executor)) as _)
    } else {
        let tcp_listener_module = tcp
            .incoming()
            .and_then(|socket| socket.peer_addr().map(|addr| (addr, socket)));

        Ok(Box::new(sockets_stream_handle(tcp_listener_module, http, executor)) as _)
    }
}

pub fn sockets_stream_handle<RW>(
    socket: impl Stream<Item = (SocketAddr, RW), Error = io::Error> + Send + 'static,
    http: Http,
    executor: TaskExecutor,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    RW: AsyncRead + AsyncWrite + AsyncPeek + Send + 'static,
{
    socket
        .and_then(|(addr, socket)| Box::new(MethodDetection::new(true, addr, socket).map(|addrs| Some(addrs))))
        .or_else(|e| {
            tcp_listener_sleep_ms(e, 1);
            future::ok(None)
        }).filter_map(|s: Option<Either<(SocketAddr, _), (SocketAddr, _)>>| s)
        // already or_else..
        // .map_err(|e: io::Error| unreachable!(e))
        .for_each(move |addrs| {
            match addrs {
                Either::A((addr, socket)) => {
                    let connection = http.serve_connection(socket, BaseService::new(addr));

                    executor.spawn(connection.then(move |connection_res| {
                        if let Err(e) = connection_res {
                            error!("client[{}]: {}", addr, e.description());
                        }
                        Ok(())
                    }));
                }
                Either::B((addr, socket)) => {
                    info!("{} Use CONNECT Method ~", addr);
                    executor.spawn(connect::process_socket(addr, socket));
                }
            }
            Ok(())
        })
}

// todo: async if need
pub fn tcp_listener_sleep_ms<T: StdError>(error: T, ms: u64) {
    error!(
        "error in accepting connection: {}, tcp_listener will sleep {} ms",
        error.description(),
        ms
    );
    thread::sleep(time::Duration::from_millis(ms));
}

pub struct MethodDetection<S> {
    pub socket: Option<S>,
    pub addr: Option<SocketAddr>,
    pub proxy: bool,
}

impl<S> MethodDetection<S>
where
    S: AsyncPeek + Send + 'static,
{
    pub fn new(proxy: bool, addr: SocketAddr, socket: S) -> Self {
        let addr = Some(addr);
        let socket = Some(socket);
        Self { proxy, addr, socket }
    }
}

impl<S> Future for MethodDetection<S>
where
    S: AsyncPeek + Send + 'static,
{
    type Item = Either<(SocketAddr, S), (SocketAddr, S)>;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.proxy {
            let mut buf = [0u8; 16];

            match self.socket.as_mut().unwrap().async_peek(&mut buf[..]) {
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(len)) => {
                    debug!("{:?}", str::from_utf8(&buf[..]));

                    let addr = mem::replace(&mut self.addr, None).unwrap();
                    let socket = mem::replace(&mut self.socket, None).unwrap();

                    // 如果太短可能需要忽视这一次，然后等下一次?
                    if len < 7 {
                        if b"CONNECT".starts_with(&buf[..len]) {
                            Ok(Async::Ready(Either::B((addr, socket))))
                        } else {
                            Ok(Async::Ready(Either::A((addr, socket))))
                        }
                    } else {
                        if buf.starts_with(b"CONNECT") {
                            Ok(Async::Ready(Either::B((addr, socket))))
                        } else {
                            Ok(Async::Ready(Either::A((addr, socket))))
                        }
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            let addr = mem::replace(&mut self.addr, None).unwrap();
            let socket = mem::replace(&mut self.socket, None).unwrap();

            Ok(Async::Ready(Either::A((addr, socket))))
        }
    }
}

pub trait AsyncPeek {
    fn async_peek(&mut self, buf: &mut [u8]) -> Result<Async<usize>, io::Error>;
}

impl AsyncPeek for tcp::TcpStream {
    fn async_peek(&mut self, buf: &mut [u8]) -> Result<Async<usize>, io::Error> {
        self.poll_peek(buf)
    }
}

impl AsyncPeek for tokio_rustls::TlsStream<tcp::TcpStream, rustls::ServerSession> {
    fn async_peek(&mut self, buf: &mut [u8]) -> Result<Async<usize>, io::Error> {
        self.get_mut().0.poll_peek(buf)
    }
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

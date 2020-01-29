use bytesize::ByteSize;
use futures::{future, FutureExt};
use hyper::{upgrade::Upgraded, Body};
use regex::Regex;
use tokio::{io, net::TcpStream, task, time};

use std::{
    // task::{Context, Poll},
    net::SocketAddr,
    time::Duration,
};

use crate::base::{
    ctx::Ctx,
    handler::{exception_handler, BoxedHandler},
    http, response, Request, Response,
};

pub fn proxy_handler<'a>(path: &str) -> BoxedHandler {
    let reg = Regex::new(path).expect("proxy_handler<'a>().Regex::new");

    Box::new(move |req: Request, addr: &SocketAddr, ctx: &mut Ctx| {
        let reg = &reg;
        connect_handler(unsafe { std::mem::transmute(reg) }, req, addr, ctx).boxed()
    })
}

pub async fn connect_handler<'a>(
    reg: &'a Regex,
    req: Request,
    addr: &'a SocketAddr,
    ctx: &'a mut Ctx,
) -> Result<Response, http::Error> {
    let dest_addr = host_addr(req.uri()).and_then(|uri| if reg.is_match(&uri) { Some(uri) } else { None });

    if let Some(dest_addr) = dest_addr {
        let proxy_socket = time::timeout(Duration::from_millis(5000), TcpStream::connect(dest_addr)).await;

        match proxy_socket {
            Ok(Ok(proxy_socket)) => {
                info!(
                    "[{} -> {}] connect ok: {}",
                    addr,
                    dest_addr,
                    proxy_socket.peer_addr().expect("proxy_socket.peer_addr()")
                );

                let addr = *addr;
                let dest_addr = dest_addr.to_owned();

                task::spawn(async move {
                    match req.into_body().on_upgrade().await {
                        Ok(upgraded) => {
                            http_tunnel(upgraded, &addr, proxy_socket, &dest_addr).await;
                        }
                        Err(e) => error!("[{} -> {}] upgrade error: {}", addr, dest_addr, e),
                    }
                });
                response().body(Body::empty())
            }
            Ok(Err(e)) => {
                error!("[{} -> {}] connect error: {}", addr, dest_addr, e);
                exception_handler(502, req, addr, ctx).await
            }
            Err(e) => {
                error!("[{} -> {}] connect error: {}", addr, dest_addr, e);
                exception_handler(504, req, addr, ctx).await
            }
        }
    } else {
        exception_handler(400, req, addr, ctx).await
    }
}

fn host_addr(uri: &http::Uri) -> Option<&str> {
    uri.authority().map(|auth| auth.as_str())
}

async fn http_tunnel(upgraded: Upgraded, addr: &SocketAddr, mut proxy_socket: TcpStream, dest_addr: &str) {
    let (mut client_r, mut client_w) = io::split(upgraded);
    let (mut proxy_r, mut proxy_w) = proxy_socket.split();

    let upload = io::copy(&mut client_r, &mut proxy_w);
    let download = io::copy(&mut proxy_r, &mut client_w);

    // maybe replace by select, it's close connection slowly
    match future::try_join(upload, download).await {
        Ok((upbs, downbs)) => {
            info!(
                "[{} -> {}] tunnel finish: up: {}, down: {}",
                addr,
                dest_addr,
                ByteSize::b(upbs).to_string_as(true),
                ByteSize::b(downbs).to_string_as(true),
            );
        }
        Err(e) => {
            error!("[{} -> {}] tunnel error: {}", addr, dest_addr, e);
        }
    }
}

#[test]
fn proxy_uri_regex_test() {
    let reg = Regex::new("").unwrap();
    assert!(reg.is_match("www.google.com"));
    assert!(reg.is_match("www.github.com"));

    let reg = Regex::new("google").unwrap();
    assert!(reg.is_match("www.google.com"));
    assert!(!reg.is_match("www.github.com"));
}

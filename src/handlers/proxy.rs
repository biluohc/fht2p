use bytesize::ByteSize;
use futures::{future, FutureExt};
use http::request::Parts;
use hyper::{header, upgrade::Upgraded, Body, Method};
use regex::Regex;
use tokio::{io, net::TcpStream, task, time};

use std::{net::SocketAddr, time::Duration};

use crate::base::{
    ctx::{ctxs, Ctx},
    handler::{exception_handler, BoxedHandler},
    http, response, Request, Response,
};

pub fn method_maybe_proxy(req: &Request) -> Option<bool> {
    // info!("url-maybe: {:?}, authority: {:?}, host: {:?}", req.uri(), req.uri().authority(), req.uri().host());

    if *req.method() == Method::CONNECT {
        Some(true)
    } else if req.uri().authority().is_some() {
        Some(false)
    } else {
        None
    }
}

pub fn proxy_handler<'a>(path: &str) -> BoxedHandler {
    let reg = Regex::new(path).expect("proxy_handler<'a>().Regex::new");

    Box::new(move |req: Request, addr: &SocketAddr, ctx: &mut Ctx| {
        let reg = &reg;
        proxy_handler2(unsafe { std::mem::transmute(reg) }, req, addr, ctx).boxed()
    })
}

// timeout:
// curl --proxy http://www:yos@127.0.0.1:8000 127.0.0.0:8080/
// normal:
// curl --proxy http://www:yos@127.0.0.1:8000 127.0.0.1:8080/
// tunnel:
// curl --proxy http://www:yos@127.0.0.1:8000 https://tools.ietf.org/favicon.ico
pub async fn proxy_handler2<'a>(
    reg: &'a Regex,
    req: Request,
    addr: &'a SocketAddr,
    ctx: &'a mut Ctx,
) -> Result<Response, http::Error> {
    if req.method() == Method::CONNECT {
        http_proxy_tunnel(reg, req, addr, ctx).await
    } else if req.method() != Method::CONNECT
        && req
            .uri()
            .authority()
            .map(|a| a.as_str())
            .map(|h| reg.is_match(h))
            .unwrap_or(false)
    {
        http_proxy_normal(req, addr, ctx).await
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

async fn http_proxy_tunnel<'a>(
    reg: &'a Regex,
    req: Request,
    addr: &'a SocketAddr,
    ctx: &'a mut Ctx,
) -> Result<Response, http::Error> {
    let dest_addr = match host_addr(req.uri()).and_then(|uri| if reg.is_match(&uri) { Some(uri) } else { None }) {
        Some(da) => da,
        None => return response().status(400).body("connect addr exception".into()),
    };
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
}
async fn http_proxy_normal<'a>(mut req: Request, addr: &'a SocketAddr, ctx: &'a mut Ctx) -> Result<Response, http::Error> {
    let header = req.headers_mut();
    header.remove(header::PROXY_AUTHORIZATION);
    header.remove("proxy-connection");

    // info!("url: {:?}", req.uri());
    // info!("header: {:?}", req.headers());

    let state = ctx.get::<ctxs::State>().unwrap();

    let (
        Parts {
            method, uri, headers, ..
        },
        body,
    ) = req.into_parts();

    let uris = uri.to_string();
    let uri = uris.parse::<reqwest::Url>().expect("hyper's Url to url::Url");
    let mut req2 = reqwest::Request::new(method, uri);
    *req2.headers_mut() = headers;
    *req2.body_mut() = Some(reqwest::Body::wrap_stream(body));

    let resp2 = match state.client().execute(req2).await {
        Ok(res) => res,
        Err(e) => {
            error!("[{} -> {}] reqwest error: {}", addr, uris, e);
            return response().status(502).body(format!("{:?}", e).into());
        }
    };

    let mut resp = response().status(resp2.status());
    *resp.headers_mut().expect("*resp.headers_mut()") = resp2.headers().clone();
    resp.body(Body::wrap_stream(resp2.bytes_stream()))
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
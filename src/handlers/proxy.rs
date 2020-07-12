use bytesize::ByteSize;
use futures::{future, FutureExt};
use http::request::Parts;
use hyper::{header, upgrade::Upgraded, Body, Method};
use regex::Regex;
use reqwest::Client;
use tokio::{
    io,
    net::{lookup_host, TcpStream},
    task, time,
};

use std::{net::SocketAddr, time::Duration};

use super::exception::exception_handler_sync;
use crate::base::{ctx::Ctx, handler::BoxedHandler, http, response, Request, Response};

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

pub fn proxy_handler(path: &str) -> BoxedHandler {
    let reg = Regex::new(path).expect("proxy_handler<'a>().Regex::new");
    let client = Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Client::builder().use_rustls_tls().build()");

    Box::new(move |req: Request, addr: &SocketAddr, ctx: &mut Ctx| {
        proxy_handler2(
            unsafe { std::mem::transmute(&reg) },
            unsafe { std::mem::transmute(&client) },
            req,
            addr,
            ctx,
        )
        .boxed()
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
    client: &'a Client,
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
        http_proxy_normal(client, req, addr, ctx).await
    } else {
        exception_handler_sync(400, None, &req, addr)
    }
}

fn host_addr(uri: &http::Uri) -> Option<&str> {
    uri.authority().map(|auth| auth.as_str())
}

async fn connect_host(uri: &str) -> io::Result<TcpStream> {
    let mut sa = None;
    for a in lookup_host(uri).await? {
        sa = Some(a);
        if a.is_ipv4() {
            break;
        }
    }
    let sa = sa.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to lookup address information: Name or service not known",
        )
    })?;

    TcpStream::connect(sa).await
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
    _ctx: &'a mut Ctx,
) -> Result<Response, http::Error> {
    let dest_addr = match host_addr(req.uri()).and_then(|uri| if reg.is_match(&uri) { Some(uri) } else { None }) {
        Some(da) => da,
        None => return response().status(400).body("connect addr exception".into()),
    };
    let proxy_socket = time::timeout(Duration::from_millis(5000), connect_host(dest_addr)).await;

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
            exception_handler_sync(502, None, &req, addr)
        }
        Err(e) => {
            error!("[{} -> {}] connect error: {}", addr, dest_addr, e);
            exception_handler_sync(504, None, &req, addr)
        }
    }
}
async fn http_proxy_normal<'a>(
    client: &'a Client,
    mut req: Request,
    addr: &'a SocketAddr,
    _ctx: &'a mut Ctx,
) -> Result<Response, http::Error> {
    let header = req.headers_mut();
    header.remove(header::PROXY_AUTHORIZATION);
    header.remove("proxy-connection");

    // info!("url: {:?}", req.uri());
    // info!("header: {:?}", req.headers());

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

    let resp2 = match client.execute(req2).await {
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

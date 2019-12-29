use bytesize::ByteSize;
use futures::{future, Future, FutureExt};
use http;
use hyper::{header, upgrade::Upgraded, Body, Request, Response, StatusCode};
use tokio::{io, net::TcpStream, task, time};

use std::{
    // task::{Context, Poll},
    net::SocketAddr,
    pin::Pin,
    time::Duration,
};

use crate::base::ctx::{ctxs, Ctx};
use crate::index::index_handler;

pub type Responder<'a> = Pin<Box<dyn Future<Output = Result<Response<Body>, http::Error>> + Send + 'a>>;
pub type Handler = dyn for<'a> Fn(Request<Body>, &'a SocketAddr, &'a mut Ctx) -> Responder<'a> + Send + Sync + 'static;

pub type BoxedHandler = Box<Handler>;

pub fn default_handler() -> BoxedHandler {
    Box::new(move |_req: Request<Body>, addr: &SocketAddr, _ctx: &mut Ctx| {
        future::ready(
            Response::builder()
                .status(StatusCode::from_u16(200).unwrap())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("{{\"socket\": \"{}\"}}", addr))),
        )
        .boxed()
    })
}

pub fn exception_handler<'a>(
    code: u16,
    _req: Request<Body>,
    addr: &'a SocketAddr,
    _ctx: &'a mut Ctx,
) -> impl Future<Output = Result<Response<Body>, http::Error>> {
    future::ready(
        Response::builder()
            .status(StatusCode::from_u16(code).unwrap())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!("{{\"socket\": \"{}\", \"code\": \"{}\" }}", addr, code))),
    )
}

pub fn redirect_handler<S: Into<String>>(
    permanent: bool,
    dest: S,
    _req: Request<Body>,
    _addr: &SocketAddr,
    _ctx: &mut Ctx,
) -> impl Future<Output = Result<Response<Body>, http::Error>> {
    future::ready(
        Response::builder()
            .status(if permanent {
                StatusCode::MOVED_PERMANENTLY
            } else {
                StatusCode::TEMPORARY_REDIRECT
            })
            .header(header::LOCATION, dest.into())
            .body(Body::empty()),
    )
}

pub fn fs_handler() -> BoxedHandler {
    use std::path::Path;

    fn fs_handler2<'a>(req: Request<Body>, addr: &'a SocketAddr, ctx: &'a mut Ctx) -> Responder<'a> {
        // let state = ctx.get::<ctxs::State>().unwrap();
        let route = ctx.get::<ctxs::Route>().unwrap();
        let reqpath = ctx.get::<ctxs::ReqPath>().unwrap();
        let reqpathcs = ctx.get::<ctxs::ReqPathCs>().unwrap();

        let mut reqpath_fixed = Path::new(&route.path);
        let mut reqpathbuf_fixed;

        let reqpathcs_remaining = &reqpathcs[route.urlcs..];

        if !reqpathcs_remaining.is_empty() {
            reqpathbuf_fixed = reqpath_fixed.to_owned();
            for cs in reqpathcs_remaining {
                reqpathbuf_fixed.push(cs);
            }
            reqpath_fixed = reqpathbuf_fixed.as_path();
        }

        let meta = if let Ok(meta) = if route.follow_links {
            reqpath_fixed.metadata()
        } else {
            reqpath_fixed.symlink_metadata()
        } {
            meta
        } else {
            return exception_handler(404, req, addr, ctx).boxed();
        };

        match (meta.is_dir(), meta.is_file()) {
            (true, false) => {
                if !reqpath.ends_with('/') {
                    let mut dest = reqpath.to_owned();
                    dest.push('/');
                    return redirect_handler(true, dest, req, addr, ctx).boxed();
                }
                return index_handler(route, reqpath, reqpath_fixed, &meta, req, addr, ctx).boxed();
            }
            (false, true) => {
                if reqpath.ends_with('/') {
                    return redirect_handler(true, reqpath.trim_end_matches('/').to_owned(), req, addr, ctx).boxed();
                }
            }
            (d, f) => {
                error!(
                    "[{}: {} -> {}] is-dir: {}, is-file: {}, is-symlink: {}",
                    addr,
                    reqpath,
                    reqpath_fixed.display(),
                    d,
                    f,
                    meta.file_type().is_symlink()
                );
                return exception_handler(403, req, addr, ctx).boxed();
            }
        }

        info!("reqpath: {}, fixed: {}", reqpath, reqpath_fixed.display());

        future::ready(
            Response::builder()
                .status(StatusCode::from_u16(200).unwrap())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("{{\"socket\": \"{}\"}}", addr))),
        )
        .boxed()
    }

    Box::new(fs_handler2)
}

pub fn proxy_handler<'a>() -> BoxedHandler {
    Box::new(|req: Request<Body>, addr: &SocketAddr, ctx: &mut Ctx| connect_handler(req, addr, ctx).boxed())
}

pub async fn connect_handler<'a>(
    req: Request<Body>,
    addr: &'a SocketAddr,
    ctx: &'a mut Ctx,
) -> Result<Response<Body>, http::Error> {
    let dest_addr = host_addr(req.uri());

    if let Some(dest_addr) = dest_addr {
        let proxy_socket = time::timeout(Duration::from_millis(5000), TcpStream::connect(dest_addr)).await;

        match proxy_socket {
            Ok(Ok(proxy_socket)) => {
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
                Response::builder().body(Body::empty())
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

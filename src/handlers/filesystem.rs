use futures::FutureExt;
use hyper::Method;

use std::{net::SocketAddr, path::PathBuf};

use super::{
    exception::{exception_handler_sync, redirect_handler_sync},
    file::file_handler,
    file_upload::file_upload_handler,
    index::index_handler,
    mkdir::{method_maybe_mkdir, mkdir_handler},
};

use crate::base::{
    ctx::{ctxs, Ctx},
    handler::BoxedHandler,
    http, Request, Response,
};

pub fn fs_handler() -> BoxedHandler {
    use std::path::Path;

    async fn fs_handler2<'a>(req: Request, addr: &'a SocketAddr, ctx: &'a mut Ctx) -> Result<Response, http::Error> {
        let route = ctx.get::<ctxs::Route>().unwrap();
        let reqpath = ctx.get::<ctxs::ReqPath>().unwrap();
        let reqpathcs = ctx.get::<ctxs::ReqPathCs>().unwrap();
        let state = ctx.get::<ctxs::State>().unwrap();

        let mut reqpath_fixed = Path::new(&route.path);
        let mut reqpathbuf_fixed;

        let reqpathcs_remaining = &reqpathcs[route.urlcs..];
        let reqpathcs_remaining_is_empty = reqpathcs_remaining.is_empty();

        if !reqpathcs_remaining_is_empty {
            reqpathbuf_fixed = reqpath_fixed.to_owned();
            for cs in reqpathcs_remaining {
                reqpathbuf_fixed.push(cs);
            }
            reqpath_fixed = reqpathbuf_fixed.as_path();
        }

        debug!("reqpath: {}, route: {:?}", reqpath, route);

        let meta = if let Ok(meta) = if route.follow_links {
            reqpath_fixed.metadata()
        } else {
            reqpath_fixed.symlink_metadata()
        } {
            meta
        } else {
            return exception_handler_sync(404, None, &req, addr);
        };

        match (meta.is_dir(), meta.is_file()) {
            (true, false) => {
                if !reqpath.ends_with('/') {
                    let mut dest = reqpath.to_owned();
                    dest.push('/');
                    return redirect_handler_sync(true, dest);
                }

                if req.method() == Method::POST {
                    if method_maybe_mkdir(&req) {
                        if !route.mkdir {
                            return exception_handler_sync(403, None, &req, addr);
                        }
                        return mkdir_handler(route, reqpath, reqpath_fixed, req, addr, state).await;
                    }
                    if !route.upload {
                        return exception_handler_sync(403, None, &req, addr);
                    }
                    return file_upload_handler(route, reqpath, reqpath_fixed, req, addr, state).await;
                };

                if ![Method::GET, Method::HEAD].contains(req.method()) {
                    return exception_handler_sync(405, None, &req, addr);
                }

                if route.redirect_html {
                    let mut file = "index.html";
                    let mut fp = reqpath_fixed.join(file);

                    let fm_is_file = move |f: PathBuf| f.metadata().map(|m| m.is_file()).unwrap_or(false);
                    let mut fm = fm_is_file(fp);

                    if !fm {
                        file = "index.htm";
                        fp = reqpath_fixed.join(file);
                        fm = fm_is_file(fp);
                    }

                    if fm {
                        let dest = format!("{}{}", reqpath, file);
                        return redirect_handler_sync(true, dest);
                    }
                }

                index_handler(route, reqpath, reqpath_fixed, &meta, req, addr, state).await
            }
            (false, true) => {
                // route is a file.. &&
                if !reqpathcs_remaining_is_empty && reqpath.ends_with('/') {
                    return redirect_handler_sync(true, reqpath.trim_end_matches('/'));
                }

                if ![Method::GET, Method::HEAD].contains(req.method()) {
                    return exception_handler_sync(405, None, &req, addr);
                }

                file_handler(route, reqpath, reqpath_fixed, &meta, req, addr, state).await
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
                exception_handler_sync(403, None, &req, addr)
            }
        }
    }

    Box::new(|req: Request, addr: &SocketAddr, ctx: &mut Ctx| fs_handler2(req, addr, ctx).boxed())
}

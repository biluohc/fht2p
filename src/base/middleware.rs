use hyper::{header, Body, Request, Response, StatusCode};
use std::{net::SocketAddr, ops::Index, time::Instant};

use crate::{
    base::ctx::{ctxs, Ctx},
    config::Auth,
    tools::{url_for_path, url_path_decode},
};

pub trait MiddleWare {
    fn before(&self, _req: &Request<Body>, _addr: &SocketAddr, _ctx: &mut Ctx) -> Result<(), Response<Body>> {
        Ok(())
    }
    fn after(&self, _resp: &Response<Body>, _addr: &SocketAddr, _ctx: &mut Ctx) {}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Logger;

impl MiddleWare for Logger {
    fn before(&self, req: &Request<Body>, _addr: &SocketAddr, ctx: &mut Ctx) -> Result<(), Response<Body>> {
        let start = Instant::now();
        let method = req.method().clone();
        let path = url_path_decode(req.uri().path()).into_owned();
        let query: ctxs::ReqQuery = req.uri().query().into();

        ctx.insert(start);
        ctx.insert(method);
        ctx.insert(path);
        ctx.insert(query);

        Ok(())
    }
    fn after(&self, resp: &Response<Body>, addr: &SocketAddr, ctx: &mut Ctx) {
        let start = ctx.get::<ctxs::ReqStart>().unwrap();
        let method = ctx.get::<ctxs::ReqMethod>().unwrap();
        let path = ctx.get::<ctxs::ReqPath>().unwrap();
        let query = ctx.get::<ctxs::ReqQuery>().unwrap();
        let code = resp.status().as_u16();

        info!("[{} {:?}]: {} {}{} {}", addr, start.elapsed(), method, path, query, code);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PathNormalizer;

impl MiddleWare for PathNormalizer {
    fn before(&self, req: &Request<Body>, _addr: &SocketAddr, ctx: &mut Ctx) -> Result<(), Response<Body>> {
        let reqpath = ctx.get::<ctxs::ReqPath>().unwrap();
        let reqpath_components_raw = reqpath.split('/').filter(|c| !c.is_empty() && c != &".");

        let reqpath_components = reqpath_components_raw
            .fold(Ok(vec![]), |cs, c| {
                cs.and_then(move |mut cs| match (cs.len() > 0, c == "..") {
                    (_, false) => {
                        cs.push(c);
                        Ok(cs)
                    }
                    (true, true) => {
                        cs.pop();
                        Ok(cs)
                    }
                    (false, true) => Err(cs),
                })
            })
            .unwrap_or_else(|e| e);

        let mut reqpath_expected = String::with_capacity(reqpath.len());
        for component in &reqpath_components {
            reqpath_expected.push('/');
            reqpath_expected.push_str(component);
        }
        if reqpath.ends_with('/') {
            reqpath_expected.push('/');
        }

        debug!("reqpath: {} -> {}, {:?}", reqpath, reqpath_expected, reqpath_components);

        if *reqpath != reqpath_expected {
            let mut reqpath_expected = url_for_path(&reqpath_expected);

            if let Some(query) = req.uri().query() {
                reqpath_expected.push('?');
                reqpath_expected.push_str(query);
            }

            let resp = Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, reqpath_expected)
                .body(Body::empty())
                .unwrap();

            return Err(resp);
        }

        let reqpath_components = reqpath_components.into_iter().map(|c| c.to_owned()).collect::<Vec<String>>();
        ctx.insert(reqpath_components);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Authenticator {
    auth: Auth,
}

impl Authenticator {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }
}

impl MiddleWare for Authenticator {
    fn before(&self, _req: &Request<Body>, _addr: &SocketAddr, _ctx: &mut Ctx) -> Result<(), Response<Body>> {
        Ok(())
    }
}

pub type BoxedMiddleWare = Box<dyn MiddleWare + Send + Sync + 'static>;

#[derive(Default)]
pub struct MiddleWares(Vec<BoxedMiddleWare>);

impl MiddleWares {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn push<M>(&mut self, middleware: M)
    where
        M: MiddleWare + Send + Sync + 'static,
    {
        self.0.push(Box::new(middleware) as _)
    }
}

impl Index<usize> for MiddleWares {
    type Output = dyn MiddleWare;

    fn index(&self, idx: usize) -> &Self::Output {
        &*self.0[idx]
    }
}

impl AsRef<[BoxedMiddleWare]> for MiddleWares {
    fn as_ref(&self) -> &[BoxedMiddleWare] {
        self.0.as_ref()
    }
}

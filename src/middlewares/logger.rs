use std::{net::SocketAddr, time::Instant};

use crate::{
    base::{
        ctx::{ctxs, Ctx},
        middleware::MiddleWare,
        Request, Response,
    },
    tools::url_path_decode,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Logger;

impl MiddleWare for Logger {
    fn before(&self, req: &Request, _addr: &SocketAddr, ctx: &mut Ctx) -> Result<(), Response> {
        let start = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();
        let path = url_path_decode(req.uri().path()).into_owned();

        ctx.insert(start);
        ctx.insert(method);
        ctx.insert(uri);
        ctx.insert(path);

        Ok(())
    }
    fn after(&self, resp: &Response, addr: &SocketAddr, ctx: &mut Ctx) {
        let start = ctx.get::<ctxs::ReqStart>().unwrap();
        let method = ctx.get::<ctxs::ReqMethod>().unwrap();
        let uri = ctx.get::<ctxs::ReqUri>().unwrap();
        let code = resp.status().as_u16();

        let uristr = uri.to_string();
        let uri = url_path_decode(&uristr);

        info!("[{} {:?}]: {} {} {}", addr, start.elapsed(), method, uri, code);
    }
}

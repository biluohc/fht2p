use std::net::SocketAddr;

use crate::{
    base::{
        ctx::{ctxs, Ctx},
        middleware::MiddleWare,
        Request, Response,
    },
    handlers::exception::redirect_handler_sync,
    tools::url_for_path,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct PathNormalizer;

impl MiddleWare for PathNormalizer {
    fn before(&self, req: &Request, _addr: &SocketAddr, ctx: &mut Ctx) -> Result<(), Response> {
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

            return Err(redirect_handler_sync(true, reqpath_expected).unwrap());
        }

        let reqpath_components = reqpath_components.into_iter().map(|c| c.to_owned()).collect::<Vec<String>>();
        ctx.insert(reqpath_components);

        Ok(())
    }
}

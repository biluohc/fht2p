use std::net::SocketAddr;

use crate::{
    base::{
        ctx::{ctxs, Ctx},
        handler::{default_handler, BoxedHandler},
        http,
        middleware::MiddleWares,
        Request, Response,
    },
    config::{Config, Route},
    handlers::{fs_handler, method_maybe_proxy, proxy_handler},
    middlewares::{auth::Authenticator, logger::Logger, path::PathNormalizer},
    service::GlobalState,
};

pub struct Router {
    global_middlewares: MiddleWares,
    routes: Vec<(Route, MiddleWares, BoxedHandler)>,
    notfound: BoxedHandler,
    proxy: Option<(Route, MiddleWares, BoxedHandler)>,
}

impl Router {
    pub fn new(config: &Config) -> Self {
        let mut routes = config
            .routes
            .values()
            .map(|r| {
                let mut r = r.clone();
                r.urlcs = r.url.split('/').filter(|c| !c.is_empty()).count();

                let mut middlewares = MiddleWares::new();
                if let Some(auth) = &config.auth {
                    if r.authorized {
                        let authenticator = Authenticator::new(auth.clone());
                        middlewares.push(authenticator);
                    }
                }

                (r, middlewares, fs_handler())
            })
            .collect::<Vec<(Route, _, _)>>();

        routes.sort_by(|a, b| b.0.urlcs.cmp(&a.0.urlcs));

        let mut global_middlewares = MiddleWares::with_capacity(2);
        global_middlewares.push(Logger);
        global_middlewares.push(PathNormalizer);

        Self {
            routes,
            global_middlewares,
            notfound: default_handler(),
            proxy: config.proxy.as_ref().map(|route| {
                (
                    route.clone(),
                    {
                        let mut middlewares = MiddleWares::new();
                        if let Some(auth) = &config.auth {
                            if route.authorized {
                                let authenticator = Authenticator::new(auth.clone());
                                middlewares.push(authenticator);
                            }
                        }

                        middlewares
                    },
                    proxy_handler(&route.path),
                )
            }),
        }
    }

    pub async fn call(addr: SocketAddr, req: Request, state: GlobalState) -> Result<Response, http::Error> {
        let this = state.router();

        let mut ctx = Ctx::with_capacity(ctxs::CAPACITY);
        ctx.insert(state);

        for idx in 0..this.global_middlewares.len() {
            if let Err(resp) = (this.global_middlewares[idx]).before(&req, &addr, &mut ctx) {
                // take global_middlewares return ok
                for gm in this.global_middlewares.as_ref()[0..idx].iter().rev() {
                    gm.after(&resp, &addr, &mut ctx);
                }

                return Ok(resp);
            }
        }

        let reqpath = ctx.get::<ctxs::ReqPath>().unwrap();
        let matched = if method_maybe_proxy(&req).is_some() {
            this.proxy.as_ref()
        } else {
            this.routes.iter().find(|&(route, _, _)| {
                reqpath.starts_with(&route.url) && (route.url.ends_with('/') || reqpath.len() == route.url.len())
                    || reqpath.trim_end_matches("/") == route.url.trim_end_matches("/")
            })
        };

        debug!(
            "matched: {} -> {:?}",
            reqpath,
            matched.as_ref().map(|m| m.0.url.as_str()) // .unwrap_or("")
        );

        let resp = if let Some((route, middlewares, handler)) = matched {
            ctx.insert(route);
            // assert_eq!(route, *ctx.get::<ctx::Route>().unwrap());

            let mut resp = None;
            for idx in 0..middlewares.len() {
                if let Err(resp_) = (middlewares[idx]).before(&req, &addr, &mut ctx) {
                    // take middlewares return ok
                    for lm in middlewares.as_ref()[0..idx].iter().rev() {
                        lm.after(&resp_, &addr, &mut ctx);
                    }
                    resp = Some(resp_);
                }
            }

            let resp = if resp.is_none() {
                (*handler)(req, &addr, &mut ctx).await?
            } else {
                resp.unwrap()
            };

            for lm in middlewares.as_ref().iter().rev() {
                lm.after(&resp, &addr, &mut ctx);
            }

            resp
        } else {
            (this.notfound)(req, &addr, &mut ctx).await?
        };

        for gm in this.global_middlewares.as_ref().iter().rev() {
            gm.after(&resp, &addr, &mut ctx);
        }

        Ok(resp)
    }
}

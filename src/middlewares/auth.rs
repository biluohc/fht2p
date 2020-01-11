use std::net::SocketAddr;

use crate::{
    base::{ctx::Ctx, middleware::MiddleWare, Request, Response},
    config::Auth,
};

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
    fn before(&self, _req: &Request, _addr: &SocketAddr, _ctx: &mut Ctx) -> Result<(), Response> {
        Ok(())
    }
}

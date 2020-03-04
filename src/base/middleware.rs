use std::{net::SocketAddr, ops::Index};

use crate::{
    base::ctx::Ctx,
    base::{Request, Response},
};

pub trait MiddleWare {
    fn before(&self, _req: &Request, _addr: &SocketAddr, _ctx: &mut Ctx) -> Result<(), Response> {
        Ok(())
    }
    fn after(&self, _resp: &mut Response, _addr: &SocketAddr, _ctx: &mut Ctx) {}
}

pub type BoxedMiddleWare = Box<dyn MiddleWare + Send + Sync + 'static>;

#[derive(Default)]
pub struct MiddleWares(Vec<BoxedMiddleWare>);

#[allow(clippy::len_without_is_empty)]
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

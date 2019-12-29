use crate::{config, service::GlobalState, typemap::TypeMap};

pub type Ctx = TypeMap;

pub mod ctxs {
    use super::*;

    pub const CAPACITY: usize = 7;

    pub type State = GlobalState;
    pub type Route = &'static config::Route;
    pub type ReqMethod = hyper::Method;
    pub type ReqStart = std::time::Instant;
    pub type ReqUri = hyper::Uri;
    pub type ReqPath = String;
    pub type ReqPathCs = Vec<String>;
}

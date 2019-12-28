use crate::{config, service::GlobalState, typemap::TypeMap};

pub type Ctx = TypeMap;

pub mod ctxs {
    use super::*;
    use std::fmt;

    pub const CAPACITY: usize = 7;

    pub type State = GlobalState;
    pub type Route = &'static config::Route;
    pub type ReqMethod = hyper::Method;
    pub type ReqStart = std::time::Instant;
    pub type ReqUri = hyper::Uri;
    pub type ReqPath = String;
    pub type ReqPathCs = Vec<String>;
    #[derive(Debug, Default, Clone)]
    pub struct ReqQuery(Option<String>);

    impl From<Option<&str>> for ReqQuery {
        fn from(s: Option<&str>) -> Self {
            Self(s.map(|s| s.to_owned()))
        }
    }

    impl fmt::Display for ReqQuery {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use fmt::Write;

            if let Some(s) = &self.0 {
                f.write_char('?')?;
                f.write_str(s)?;
            }

            Ok(())
        }
    }
}

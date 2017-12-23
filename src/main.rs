#[macro_use]
extern crate log;
extern crate mxo_env_logger;
use mxo_env_logger::*;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate app;
extern crate bytes;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate hyper_fs;
extern crate tokio_core;
extern crate url;
extern crate walkdir;

pub(crate) mod consts;
pub mod exception;
pub mod index;
pub mod server;
pub mod args;

fn main() {
    init().expect("Init log failed");

    if let Err(e) = server::run(args::parse()) {
        error!("{}", e.description())
    }
}

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
extern crate url;
extern crate bytes;
extern crate hyper;
extern crate walkdir;
extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate hyper_fs;

extern crate signalbool;
use signalbool::{Flag, Signal, SignalBool};

pub(crate) mod consts;
pub mod exception;
pub mod index;
pub mod server;
pub mod args;

use std::thread::{sleep, Builder};
use std::sync::mpsc::{channel, TryRecvError};
use std::time::Duration;
use std::process::exit;
use std::io;

/// `Http.serve_addr_handle()` can not get `Request'`s remote addr now...
fn main() {
    init().expect("Init log failed");

    let config = args::parse();
    debug!("{:?}", config);

    let sb = SignalBool::new(&[Signal::SIGINT], Flag::Restart)
        .map_err(|e| eprintln!("Register Signal failed: {:?}", e))
        .unwrap();
    let (mp, sc) = channel::<io::Error>();

    Builder::new()
        .name("event-loop".to_owned())
        .spawn(move || server::run(config).map_err(|e| mp.send(e).unwrap()))
        .unwrap();

    loop {
        sleep(Duration::from_millis(10));
        match sc.try_recv() {
            Ok(e) => {
                error!("{}", e.description());
                exit(1);
            }
            Err(TryRecvError::Disconnected) => unreachable!(),
            _ => {}
        }
        if sb.caught() {
            break;
        }
    }
}

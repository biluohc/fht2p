/*!
# A HTTP Server for Static File written with Rust.

## Support Unix-like and windows 7+.

## Usage  
```sh
    cargo install --git https://github.com/biluohc/fht2p  fht2p

    # running fht2p --help(-h) to get help.

    fht2p -h
```
### Or
```sh
    git clone https://github.com/biluohc/fht2p --depth 1 
    # cargo  install --path fht2p/ fht2p
    
    cd fht2p 
    cargo build --release

    ./target/release/fht2p --help
```

## Binary

* [The Release Page](https://github.com/biluohc/fht2p/releases)  

## Help
```sh
fht2p 0.8.1 (3d7711e7@master rustc1.24.0-nightly 2017-12-26UTC)
A HTTP Server for Static File written with Rust
Wspsxing <biluohc@qq.com>
Github: https://github.com/biluohc/fht2p

USAGE:
   fht2p [options] [<PATH>...]

OPTIONS:
   -h, --help                               Show the help message
   -V, --version                            Show the version message
   -r, --redirect-html                      Redirect dir to 'index.html/htm`, if it exists
   -m, --magic-limit <byte>[10485760]       The limit for parse mimetype(use 0 to close)
   -k, --keep-alive                         Close HTTP keep alive
   -c, --config <config>(optional)          Sets a custom config file
   -C, --config-print                       Print the default config file
   -i, --ip <ip>[0.0.0.0]                   Sets listenning ip
   -p, --port <port>[8080]                  Sets listenning port

ARGS:
   <PATH>["./"]     Sets the paths to share
```
*/
#![allow(unknown_lints, clone_on_ref_ptr, boxed_local)]
#[macro_use]
extern crate log;
extern crate mxo_env_logger;
use mxo_env_logger::*;
extern crate app;
extern crate bytes;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate hyper_fs;
#[macro_use]
extern crate lazy_static;
extern crate mime_guess;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate tokio_core;
extern crate toml;
extern crate url;

extern crate signalbool;
use signalbool::{Flag, Signal, SignalBool};

pub(crate) mod consts;
pub mod content_type;
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

/*!
# A HTTP Server for Static File written with Rust.

## Support Unix-like and windows 7+.

## Snapshot

![snapshot.png](https://raw.githubusercontent.com/biluohc/fht2p/master/config/assets/snapshot.png)

## Usage  
```sh
    cargo install --git https://github.com/biluohc/fht2p fht2p -f

    # running fht2p --help(-h) to get help.

    fht2p -h
```
### Or
```sh
    git clone https://github.com/biluohc/fht2p
    # cargo  install --path fht2p/ fht2p -f
    
    cd fht2p 
    cargo build --release

    ./target/release/fht2p --help
```

## Help
```sh
fht2p 0.8.2 (f9dbc530@master rustc1.27.0-nightly 2018-04-18UTC)
A HTTP Server for Static File written with Rust
Wspsxing <biluohc@qq.com>
Github: https://github.com/biluohc/fht2p

USAGE:
   fht2p [options] [<PATH>...]

OPTIONS:
   -h, --help                               Show the help message
   -V, --version                            Show the version message
   -r, --redirect-html                      Redirect dir to `index.html/htm`, if it exists
   -m, --magic-limit <byte>[10485760]       The limit for detect file ContenType(use 0 to close)
   -k, --keep-alive                         Close HTTP keep alive
   -c, --config <config>(optional)          Sets a custom config file
   -C, --config-print                       Print the default config file
   -s, --cache-secs <secs>[60]              Sets cache secs(use 0 to close)
   -f, --follow-links                       Whether follow links(default follow)
   -i, --ip <ip>[0.0.0.0]                   Sets listenning ip
   -p, --port <port>[8080]                  Sets listenning port

ARGS:
   <PATH>["./"]     Sets the paths to share
```
*/
#![allow(unknown_lints, clone_on_ref_ptr, boxed_local)]
#[macro_use]
extern crate log;
extern crate clap;
extern crate fern;
// #[macro_use]
// extern crate cfg_if;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate lazy_static;
extern crate mime_guess;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate json5;
extern crate serde;
extern crate url;
#[macro_use]
extern crate askama;

extern crate bytes;
extern crate bytesize;
extern crate futures;
extern crate http;
extern crate httparse;
extern crate hyper;
extern crate net2;
extern crate num_cpus;
extern crate systemstat;
extern crate tokio;
extern crate tokio_retry;
extern crate tokio_rustls;

#[macro_use(signalfn, ctrlcfn)]
extern crate signalfn;
use signalfn::register_ctrlcfn;

pub mod base;
pub mod consts;
pub mod reuse;
// pub(crate) mod content_type;
// pub(crate) mod exception;
pub mod server;
// pub(crate) mod router;
// pub(crate) mod views;
// pub(crate) mod index;
// pub(crate) mod tools;
pub mod args;
pub mod config;
pub mod connect;
pub mod logger;
pub mod stat;

pub use std::error::Error as StdError;
pub use std::process::exit as process_exit;

fn callback() {
    info!("Received a CtrlC, exiting...");
    process_exit(1)
}

ctrlcfn!(ctrlc_exit, callback);

fn main() {
    let config = args::parse();

    debug!("{:?}", config);

    register_ctrlcfn(ctrlc_exit)
        .map_err(|e| error!("Register CtrlC Signal failed: {:?}", e))
        .ok();

    let rest = server::run(config);

    debug!("{:?}", rest);
    if let Err(e) = rest {
        error!("{}", e);
        process_exit(1)
    }
}

// htop -p `ps -a|grep fht2p|awk -F ' '  '{print $1}' `

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
#![allow(unknown_lints)]
#[macro_use]
pub extern crate nonblock_logger;
#[macro_use]
extern crate nom;
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;

pub mod how {
    pub type Error = anyhow::Error;
    pub type Result<T> = anyhow::Result<T>;
}

pub mod args;
pub mod base;
pub mod config;
pub mod consts;
pub mod contentype;
pub mod file;
pub mod file_upload;
pub mod index;
pub mod logger;
pub mod service;
pub mod stat;
pub mod tools;
pub mod typemap;
pub mod views;

pub use std::process::exit as process_exit;

fn main() {
    logger::fun(|| {
        let config = args::parse();

        debug!("{:?}", config);
        if let Err(e) = service::run(config) {
            error!("{}", e);
            process_exit(1)
        }
    })
}

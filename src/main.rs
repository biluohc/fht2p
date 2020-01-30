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
fht2p 0.9.0 (8fe4f69b-modified@0.9 rustc1.40.0 2020-01-30~13:04:13UTC)
Wspsxing <biluohc@qq.com>
A HTTP Server for Static File written with Rust

USAGE:
    fht2p [FLAGS] [OPTIONS] [PATH]...

FLAGS:
        --config-print     Print the default config file
    -f, --follow-links     Whether follow links(default not)
    -h, --help             Prints help information
    -k, --keepalive        Close HTTP keep alive
    -m, --mkdir            Whether enable mkdir(default not)
    -r, --redircet-html    Redirect dir to `index.html` or `index.htm` if it exists
    -s, --show-hider       show entries starts with '.'
    -u, --upload           Whether enable upload(default not)
    -V, --version          Prints version information
    -v                     Increases logging verbosity each use for up to 2 times(info0_debug1_trace2+)

OPTIONS:
    -a, --auth <auth>                  Set the username:password for authorization
        --cache-secs <cache-secs>      Set cache secs(use 0 to close)
    -C, --cert <cert>                  Set the cert for https,  public_key_file:private_key_file
    -c, --config <config>              Set the path to a custom config file
    -i, --ip <ip>                      Set listenning ip [default: 0.0.0.0]
    -M, --magic-limit <magic-limit>    The limit for detect file ContenType(use 0 to close)
    -p, --port <port>                  Set listenning port [default: 8000]
    -P, --proxy <proxy>                Enable http tunnel proxy(CONNECT)

ARGS:
    <PATH>...    Set the paths to share
```
*/
#[macro_use]
pub extern crate nonblock_logger;
pub extern crate libfht2p;

use libfht2p::{args, service};

pub use std::process::exit as process_exit;

fn main() {
    let (config, _handle) = args::parse();

    trace!("{:?}\n", &config);
    if let Err(e) = service::run(config) {
        error!("{}", e);
        process_exit(1)
    }
}

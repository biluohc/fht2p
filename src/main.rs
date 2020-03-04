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
fht2p 0.9.3 (8d85ce30-modified@master rustc1.41.1 2020-03-04~15:35:48UTC)
Wspsxing <biluohc@qq.com>
A HTTP Server for Static File written with Rust

USAGE:
    fht2p [FLAGS] [OPTIONS] [PATH]...

FLAGS:
    -F, --config-print     Print the content of default config file
    -d, --disable-index    Disable index(directory) view(will return 403)
    -f, --follow-links     Enable follow links
    -h, --help             Prints help information
    -k, --keepalive        Close HTTP keep alive
    -m, --mkdir            Enable mkdir function
    -Q, --qr-code          Show URL's QR code at startup
    -r, --redirect-html    Redirect dir to `index.html` or `index.htm` if it exists
    -s, --show-hider       Show entries starts with '.'
    -u, --upload           Enable upload function
    -V, --version          Prints version information
    -v, --verbose          Increases logging verbosity each use(warn0_info1_debug2_trace3+)

OPTIONS:
    -a, --auth <auth>                  Set the username:password for authorization
    -S, --cache-secs <cache-secs>      Set the secs of cache(use 0 to close) [default: 60]
    -C, --cert <cert>                  Set the cert for https,  public_cert_file:private_key_file
        --compress <compress>          Set the level for index compress, should between 0~9, use 0 to close [default: 5]
    -c, --config <config>              Set the path to use a custom config file
                                       default path: ~/.config/fht2p/fht2p.json
    -i, --ip <ip>                      Set listenning ip address [default: 127.0.0.1]
    -M, --magic-limit <magic-limit>    The size limit for detect file ContenType(use 0 to close) [default: 10485760]
    -p, --port <port>                  Set listenning port [default: 8000]
    -P, --proxy <proxy>                Enable http proxy(Regular for allowed domains, empty string can allow all)

ARGS:
    <PATH>...    Set the paths to share [Default: "."]
```
*/
#[macro_use]
pub extern crate nonblock_logger;
pub extern crate fht2plib;

use fht2plib::{args, process_exit, service::Service};

fn main() {
    let (config, mut handle) = args::parse();

    trace!("{:?}\n", &config);

    if let Err(e) = config.startup() {
        error!("{}", e);
        handle.join();
        process_exit(1)
    }
}

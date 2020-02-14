[![Build status](https://travis-ci.org/biluohc/fht2p.svg?branch=master)](https://github.com/biluohc/fht2p)

## A HTTP Server for Static File written with Rust.

### Support Unix-like and windows 7+.

### Snapshot

![snapshot.png](https://raw.githubusercontent.com/biluohc/fht2p/master/config/assets/snapshot.png)

### Usage
```sh
    cargo install --git https://github.com/biluohc/fht2p fht2p -f

    # running fht2p --help(-h) to get help.

    fht2p -h
```
#### Or
```sh
    git clone https://github.com/biluohc/fht2p
    # cargo  install --path fht2p/ fht2p -f

    cd fht2p
    cargo build --release

    ./target/release/fht2p --help
```

### Help
```sh
fht2p 0.9.2 (a91f0a0f-modified@qr rustc1.41.0 2020-02-14~14:15:32UTC)
Wspsxing <biluohc@qq.com>
A HTTP Server for Static File written with Rust

USAGE:
    fht2p [FLAGS] [OPTIONS] [PATH]...

FLAGS:
    -F, --config-print     Print the content of default config file
    -f, --follow-links     Enable follow links
    -h, --help             Prints help information
    -k, --keepalive        Close HTTP keep alive
    -m, --mkdir            Enable mkdir function
    -Q, --qr-code          Show URL's QR code at startup
    -r, --redirect-html    Redirect dir to `index.html` or `index.htm` if it exists
    -s, --show-hider       Show entries starts with '.'
    -u, --upload           Enable upload function
    -V, --version          Prints version information
    -v                     Increases logging verbosity each use for up to 3 times(warn0_info1_debug2_trace3+)

OPTIONS:
    -a, --auth <auth>                  Set the username:password for authorization
    -S, --cache-secs <cache-secs>      Set the secs of cache(use 0 to close) [default: 60]
    -C, --cert <cert>                  Set the cert for https,  public_cert_file:private_key_file
    -c, --config <config>              Set the path to use a custom config file
                                       default path: ~/.config/fht2p/fht2p.json
    -i, --ip <ip>                      Set listenning ip address [default: 127.0.0.1]
    -M, --magic-limit <magic-limit>    The size limit for detect file ContenType(use 0 to close) [default: 10485760]
    -p, --port <port>                  Set listenning port [default: 8000]
    -P, --proxy <proxy>                Enable http proxy(Regular for allowed domains, empty string can allow all)

ARGS:
    <PATH>...    Set the paths to share [Default: "."]
```

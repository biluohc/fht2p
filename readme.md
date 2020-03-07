[![Build status](https://travis-ci.org/biluohc/fht2p.svg?branch=master)](https://github.com/biluohc/fht2p)

[中文](https://github.com/biluohc/fht2p/blob/master/readme_zh.md)
fht2p is a cross-platform HTTP static file server developed using Rust. The project has been tested on Linux, MacOS and Windows.

## Features
- Reliable: Implemented in Rust, integrated testing, safe and reliable
- Universal: Fully cross-platform, available for both Unix-like and Windows systems
- Convenient: static linking, does not depend on external dynamic libraries such as openssl, download and use
- Fast: Asynchronous multi-threaded, built on tokio and hyper, fast response, massive concurrency
- Functions: file download and upload, directory browsing, resume from breakpoint, proxy function, configuration options, everything

### Functions
1. Multi-path sharing
1. File breakpoint resume
1. Closeable directory browsing (and sorting and other functions)
1. HTTP Cache
1. File upload, make directory
1. HTTPS (tokio-rustls, does not depend on external dynamic libraries)
1. HTTP proxy (tunnel proxy, general proxy)
1. Basic Authentication
1. Cross-Origin Resource Sharing (CORS)
1. Directory page compression (GZIP)
1. Command line arguments
1. Configuration file (format is json5, similar to json but supports comments, etc.)
1. Terminal log optional level
1. Output service's URL at startup, output QR code optionally 

### Snapshot

![snapshot.png](https://raw.githubusercontent.com/biluohc/fht2p/master/config/assets/snapshot.png)

### Install
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

### The Help information, the configuration file can refer to the config directory under this project
```sh
fht2p 0.9.3 (8d8f1c78-modified@master rustc1.41.1 2020-03-07~12:16:57UTC)
Wspsxing <biluohc@qq.com>
A cross-platform HTTP static file server developed using Rust

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

### Tips
1. About the priority of options and profiles

    The default configuration file is located in `$HOME/.config/fht2p/fht2p.json`, if not exist, you can create

    There are four types of options:
- The first is --help, --version and --config-print. Programs will exit very quickly, regardless of priority.
- The second is --verbose and --qr-code They ignore priority and have no conflict with other options
- The third is --config specifies the configuration file, the fourth option is ignored
- The fourth is other options and parameters, once you have it, the default configuration file will be ignored (this is to prevent the priority from being too complicated)

2. About security and HTTPS

- HTTP is a plain text protocol based on TCP. There is no security at all. If security is required, HTTPS must be used.
- The program listens to the local loopback address (`127.0.0.1`) by default for security. If you want to access outside the machine, you can monitor` 0.0.0.0` or a specific address and configure your firewall
- The program listens to the current directory by default. Please do not share the home directory or the root directory on the network unless you understand what you are doing

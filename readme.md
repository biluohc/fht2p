[![Build status](https://travis-ci.org/biluohc/fht2p.svg?branch=master)](https://github.com/biluohc/fht2p)

## A HTTP Server for Static File written with Rust.

### Support Unix-like and windows 7+.

### Usage
```sh
    cargo install --git https://github.com/biluohc/fht2p  fht2p

    # running fht2p --help(-h) to get help.

    fht2p -h
```
#### Or
```sh
    git clone https://github.com/biluohc/fht2p --depth 1
    # cargo  install --path fht2p/ fht2p

    cd fht2p
    cargo build --release

    ./target/release/fht2p --help
```

### Binary

* [The Release Page](https://github.com/biluohc/fht2p/releases)

### Help
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

[![Build status](https://travis-ci.org/biluohc/fht2p.svg?branch=master)](https://github.com/biluohc/fht2p)

# A HTTP Server for Static File written with Rust.

## Support Unix-like and windows 7+.

## Usage  
```sh
    cargo +nightly install --git https://github.com/biluohc/fht2p  fht2p

    # running fht2p --help(-h) to get help.

    fht2p -h
```
### Or
```sh
    git clone https://github.com/biluohc/fht2p --depth 1 
    # cargo  install --path fht2p/ fht2p
    
    cd fht2p 
    cargo +nightly build --release

    ./target/release/fht2p --help
```

## Binary

* [The Release Page](https://github.com/biluohc/fht2p/releases)  

## Help
```sh
fht2p 0.8.0
A HTTP Server for Static File written with Rust

AUTHOR:
   Wspsxing <biluohc@qq.com>

ADDRESS:
   Github: https://github.com/biluohc/fht2p

USAGE:
   fht2p [options] [<PATH>...]

OPTIONS:
   -h, --help                            Show the help message
   -V, --version                         Show the version message
   -c, --config <config>(optional)       Sets a custom config file
   -C, --cp                              Print the default config file
   -i, --ip <ip>[0.0.0.0]                Sets listenning ip
   -p, --port <port>[8080]               Sets listenning port
   -r, --rr                              Redirect root('/') to `/index.htm[l]`
   -k, --ka <secs>(optional)             Time HTTP keep alive(default not use)

ARGS:
   <PATH>["./"]     Sets the path to share
```

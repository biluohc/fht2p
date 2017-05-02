[![Build status](https://travis-ci.org/biluohc/fht2p.svg?branch=master)](https://github.com/biluohc/fht2p)

# A HTTP Server for Static File written with Rust.

## Support Unix-like and windows 7+.

## Usage  
```sh
    cargo +nightly install --git https://github.com/biluohc/fht2p  fht2p

    # running fht2p --help(-h) to get help.

    fht2p --help
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

## Ps
* 0.0.0.0 is default listenning address.
* 8080 is default port.
* keep-alive is default close.
* ./ is default dir.

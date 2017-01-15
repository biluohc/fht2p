# A HTTP Server for Static File written with Rust.

## Support Unix-like and windows 7+.

## Usage  
```shell
    git clone https://github.com/biluohc/fht2p --depth 1
    cd fht2p 
    cargo  build   --release

    # running fht2p --help(-h) to get help.
    target/release/fht2p --help
```
### If on windows,you should running `windres favicon.rc favicon.o`  before run `cargo  build   --release`.

### 
* 0.0.0.0 is default listenning address.
* 8080 is default port.
* keep-alive is default close.
* ./ is default dir.

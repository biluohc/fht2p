// #![allow(dead_code)]

extern crate urlparse;
extern crate  chrono;

mod server;
use server::fht2p;

fn main() {
    match fht2p() {
        Ok(ok) => println!("{:?}", ok),
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
}

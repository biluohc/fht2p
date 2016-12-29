// #![allow(dead_code)]

extern crate urlparse;
extern crate  chrono;
#[macro_use]
extern crate stderr;

mod server;
use server::fht2p;

fn main() {
    match fht2p() {
        Ok(ok) => println!("{:?}", ok),
        Err(e) => {
            errln!("{}", e);
            std::process::exit(1);
        }
    };
}

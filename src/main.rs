extern crate signal_monitor as Sm;
#[macro_use]
extern crate stderr;

extern crate fht2p;

use std::process::exit;

fn main() {
    if let Err(e) = fht2p::fun() {
        assert_ne!("", e.trim());
        errln!("{}", e);
        exit(1);
    };
    Sm::join(); //Ctrlc Signal
}

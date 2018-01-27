#[macro_use(ctrlcfn, signalfn)]
extern crate signalfn;
use signalfn::register_ctrlcfn;

fn call_fn() {
    println!("\rCatch CtrlC, exit!");
    std::process::exit(0);
}

ctrlcfn!(catch_ctrlc, call_fn);

use std::{thread, time};

fn main() {
    println!("{:?}", register_ctrlcfn(catch_ctrlc));
    println!("{:?}", std::io::Error::last_os_error());
    thread::sleep(time::Duration::from_secs(1000))
}

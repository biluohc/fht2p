#![feature(link_args)]
#![allow(dead_code)]

extern crate urlparse;
extern crate time;
extern crate tini;

extern crate signal_monitor as Sm;
extern crate poolite;
extern crate html5;
extern crate app;
#[macro_use]
extern crate stderr;
use stderr::Loger;

use std::process::exit;

mod server;
use server::fht2p;

fn main() {
    dbstln!("before: {:?}-->{:?}", Loger::get(), Loger::status());
    init!(); // 初始化--log debug
    dbstln!("After: {:?}-->{:?}", Loger::get(), Loger::status());

    if let Err(e) = fht2p() {
        assert_ne!("", e.trim());
        errln!("{}", e);
        exit(1);
    };
    Sm::join(); //Ctrlc Signal
}

#![feature(link_args)]
#![allow(dead_code)]

extern crate urlparse;
extern crate time;
extern crate tini;

extern crate app;
extern crate html5;
extern crate poolite;
#[macro_use]
extern crate stderr;
use stderr::Loger;

extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread::sleep;
use std::sync::Arc;
use std::process;

mod server;
use server::fht2p;

fn main() {
    // 初始化--log debug
    dbstln!("before: {:?}-->{:?}", Loger::get(), Loger::status());
    init!();
    dbstln!("After: {:?}-->{:?}", Loger::get(), Loger::status());

    if let Err(e) = fht2p() {
        assert_ne!("", e.trim());
        errln!("{}", e);
        process::exit(1);
    };

    let waiting = Arc::new(AtomicBool::new(true));
    let wait = waiting.clone();
    ctrlc::set_handler(move || { wait.store(false, Ordering::SeqCst); }).expect("Setting Ctrl-C handler fails");
    while waiting.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(100)); // 100 ms
    }
}

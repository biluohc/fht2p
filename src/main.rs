#![feature(link_args)]
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate urlparse;
extern crate time;

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
    init!(); // 初始化--log debug
    if let Err(e) = fht2p() {
        assert_ne!("", e.trim());
        errln!("{}", e);
        exit(1);
    };
    Sm::join(); //Ctrlc Signal
}

// ini同键值后者覆盖前者(此次引入，以后用toml之类吧)，windows->404,233(以前引入，版本未验证)

// 以后分离出crate: url(虚拟-原始映射)，html ,Status(enum_struct S_200,etc) ,等模块，或许可以试试workspace

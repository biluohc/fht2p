#![feature(link_args)]
#![allow(dead_code)]

#![feature(plugin)]
#![plugin(maud_macros)]
extern crate maud;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate urlparse;
extern crate time;

extern crate poolite;
use self::poolite::{Pool, IntoIOResult};
extern crate app;
#[macro_use]
extern crate stderr;
use stderr::Loger;

use std::net::{TcpListener, TcpStream};
use std::error::Error;
use std::thread;
use std::io;

/// `static variables`
pub mod statics;
use self::statics::*;
/// Handle `args`
pub mod args;
use self::args::Config;
mod time_;
/// `Date`: `Local` and `UTC` time
pub use time_::Time;
/// `Resquest` and `Response`
pub mod client;
use client::*;

/// `main` without wait
pub fn fun() -> Result<(), String> {
    init!(); // 初始化--log debug
    let config = args::parse();
    Route::init(&config.routes);
    dbln!("{:?}\n", config);
    spawn(&config)
        .map_err(|e| format!("{:?}", e.description()))?;
    // std::process::exit(0);
    Ok(())
}

fn spawn(config: &Config) -> io::Result<()> {
    let format = |map| {
        let mut str = String::new();
        for (k, v) in map {
            str.push_str(&format!("   {:?} -> {:?}\n", k, v));
        }
        str
    };
    let mut rest: io::Result<()> = Ok(());
    for server in &config.servers {
        match TcpListener::bind(server) {
            Ok(tcp_listener) => {
                println!("{}/{} Serving at {} for:\n{}",
                         NAME,
                         VERSION,
                         server,
                         format(&config.routes).trim_right());
                println!("You can visit http://127.0.0.1:{}", server.port());
                let pool = Pool::new()
                    .load_limit(Pool::num_cpus() * Pool::num_cpus())
                    .run()
                    .into_iorst()?;
                thread::Builder::new()
                    .spawn(move || { for_listener(&tcp_listener, &pool); })?;
                return Ok(());
            }
            Err(e) => rest = Err(e),
        }
    }
    rest
}

fn for_listener(tcp_listener: &TcpListener, pool: &Pool) {
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                // 一个进程一个堆，一个线程一个栈。
                // 栈大小，linux主默认8m，副线程和win一样 2m(这里给rust坑死了,一直stackover。以后要注意默认值)。
                pool.push(move || match_client(stream));
            }
            Err(e) => {
                errstln!("{:?}", e);
                // connection failed
            }
        };
    }
}

fn match_client(mut stream: TcpStream) {
    if let Err(e) = handle_client(&mut stream) {
        errstln!("{}_Warning@TcpStream: {}", NAME, e.description())
    }
}

fn handle_client(mut stream: &mut TcpStream) -> io::Result<()> {
    let req = Request::from_stream(&mut stream)?;
    let client = req.into_client();
    dbstln!("{:?}", client.req());
    client.method_call(&mut stream);
    Ok(())
}

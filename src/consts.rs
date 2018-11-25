// crate's info
pub const NAME: &str = env!("CARGO_PKG_NAME");
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/fht2p.txt"));
pub const AUTHOR: &str = "Wspsxing";
pub const EMAIL: &str = "biluohc@qq.com";
pub const DESC: &str = env!("CARGO_PKG_DESCRIPTION");
pub const URL_NAME: &str = "Github";
pub const URL: &str = "https://github.com/biluohc/fht2p";

// config file
pub const CONFIG_STR_PATH: &str = "fht2p.json";
pub const CONFIG_STR: &str = include_str!("../config/fht2p.json");

pub const CONTENT_TYPE: &str = "Content-Type";
pub const CHARSET: &str = "charset=utf-8";
pub const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";

// use hyper::header::HeaderMap;

use std::cell::UnsafeCell;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

lazy_static!{
    //  10485760 = 10M
    pub static ref MAGIC_LIMIT: MutStatic<u64> = MutStatic::new(1024*1024*10);
    // pub static ref HTML_HEADERS:  MutStatic<HeaderMap> = {
    //     let mut tmp =HeaderMap::new();
    //     tmp.set_raw(CONTENT_TYPE, HTML_CONTENT_TYPE);
    //      MutStatic::new(tmp)
    // };
    pub static ref SERVER_ADDR: MutStatic<SocketAddr> = MutStatic::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080));
}

/// it's unsync, but only modify it before read it concurrent, lazy to use `RwLock`..
pub struct MutStatic<T>(UnsafeCell<T>);

impl<T> MutStatic<T> {
    pub fn new(data: T) -> MutStatic<T> {
        MutStatic(UnsafeCell::new(data))
    }
    pub fn get(&self) -> &T {
        unsafe { self.0.get().as_ref().unwrap() }
    }
    // tls
    pub fn get_mut(&self) -> &mut T {
        unsafe { self.0.get().as_mut().unwrap() }
    }
    // modify it before read it concurrent
    pub fn set(&self, new: T) {
        unsafe { self.0.get().as_mut().map(|d| *d = new).unwrap() }
    }
}

unsafe impl<T> Send for MutStatic<T> {}
unsafe impl<T> Sync for MutStatic<T> {}

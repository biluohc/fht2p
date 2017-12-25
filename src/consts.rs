// crate's info
pub const NAME: &str = env!("CARGO_PKG_NAME");
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
include!(concat!(env!("OUT_DIR"), "/fht2p.txt"));
pub const AUTHOR: &str = "Wspsxing";
pub const EMAIL: &str = "biluohc@qq.com";
pub const DESC: &str = env!("CARGO_PKG_DESCRIPTION");
pub const URL_NAME: &str = "Github";
pub const URL: &str = "https://github.com/biluohc/fht2p";

// config file
pub const CONFIG_STR_PATH: &str = "fht2p.toml";
pub const CONFIG_STR: &str = include_str!("../config/fht2p.toml");
pub const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/fht2p.css"));

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::cell::UnsafeCell;

lazy_static!{
    pub static ref SERVER_ADDR: MutStatic<SocketAddr> = MutStatic::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0));
}

/// it unsync, but only modify it before read it concurrent, lazy to use `RwLock`..
pub struct MutStatic<T>(UnsafeCell<T>);

impl<T> MutStatic<T> {
    pub fn new(data: T) -> MutStatic<T> {
        MutStatic(UnsafeCell::new(data))
    }
    pub fn get(&self) -> &T {
        unsafe { self.0.get().as_ref().unwrap() }
    }
    pub fn set(&self, new: T) {
        unsafe { self.0.get().as_mut().map(|d| *d = new).unwrap() }
    }
}

unsafe impl<T> Send for MutStatic<T> {}
unsafe impl<T> Sync for MutStatic<T> {}

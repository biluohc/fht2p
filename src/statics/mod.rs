// crate's info
pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &'static str = "Wspsxing";
pub const EMAIL: &'static str = "biluohc@qq.com";
pub const DESC: &'static str = env!("CARGO_PKG_DESCRIPTION");
pub const URL_NAME: &'static str = "Github";
pub const URL: &'static str = "https://github.com/biluohc/fht2p";

// resource files
pub const FAVICON_ICO: &'static [u8; 4286] = include_bytes!("../../config/default/favicon.ico");
pub const FAVICON_ICO_PATH: &'static str = "/favicon.ico";
pub const CSS: &'static str = include_str!("../../config/default/fht2p.css");
pub const CSS_PATH: &'static str = "/fht2p.css";
pub const JS: &'static str = include_str!("../../config/default/fht2p.js");
pub const JS_PATH: &'static str = "/fht2p.js";

// config file
pub const CONFIG_STR_PATH: &'static str = "fht2p.toml";
pub const CONFIG_STR: &'static str = include_str!("../../config/fht2p.toml");

pub const BUFFER_SIZE: usize = 1024 * 1024; //字节1024*1024=>1m
pub const TIME_OUT: u64 = 5; // 5 secs 以后放到选项/配置
// set_nonblocking 不能使用,因为读取文件会阻塞，只能set_write/read_timeout() 来断开一直阻塞的连接。

// exe的图标。非Windows平台不需要rc资源。
#[cfg(windows)]
mod win_ico {
    #[link_args = "favicon.o"]
    extern "C" {}
}

use super::Time;
mod use_after_init;
pub use self::use_after_init::UseAfterInit;
mod route;
pub use self::route::Route;

/// Default whether keep alive
pub static mut KEEP_ALIVE: bool = false;
/// Default time out(ms) for socket read
pub static mut SOCKET_TIMEOUT: u64 = 30_000; //ms
pub fn keep_alive() -> bool {
    unsafe { KEEP_ALIVE }
}
pub fn keep_alive_set(b: bool) {
    unsafe {
        KEEP_ALIVE = b;
    }
}
pub fn socket_timeout() -> u64 {
    unsafe { SOCKET_TIMEOUT }
}
pub fn socket_timeout_set<T: Into<u64>>(time_out: T) {
    unsafe { SOCKET_TIMEOUT = time_out.into() }
}

lazy_static! {
   static ref UPTIME:Time = Time::now(); 
}
pub fn uptime() -> &'static Time {
    &UPTIME
}

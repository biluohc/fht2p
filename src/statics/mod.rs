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
// set_nonblocking 不能使用,因为读取文件会阻塞，只能set_write/read_timeout() 来断开一直阻塞的连接。

// exe的图标。非Windows平台不需要rc资源。
#[cfg(windows)]
mod win_ico {
    #[link_args = "favicon.o"]
    extern "C" {}
}

use std::time::Duration;
use super::Time;
use stderr::StaticMut;
mod route;
pub use self::route::Route;

pub static mut REDIRECT_ROOT: bool = false;
pub fn redirect_root() -> &'static bool {
    unsafe { &REDIRECT_ROOT }
}
pub fn redirect_root_set(b: bool) {
    unsafe { REDIRECT_ROOT = b }
}
lazy_static!{
    static ref SOCKET_TIME_OUT: StaticMut<Duration> = StaticMut::new(Duration::new(unsafe {SOCKET_TIME_OUT_MS},0));
}
/// `ms`
pub static mut SOCKET_TIME_OUT_MS: u64 = 5000;

pub fn socket_timeout_ms() -> &'static u64 {
    unsafe { &SOCKET_TIME_OUT_MS }
}
pub fn socket_timeout() -> &'static Duration {
    SOCKET_TIME_OUT.as_ref()
}
pub fn socket_timeout_set<T: Into<u64>>(time_out: T) {
    let time_out = time_out.into();
    unsafe { SOCKET_TIME_OUT_MS = time_out }
    let tmp = SOCKET_TIME_OUT.as_mut();
    *tmp = Duration::new(*socket_timeout_ms(), 0)
}
/// `sec`
pub static mut HTTP_TIME_OUT_SEC: u64 = 5;
lazy_static ! {
static ref HTTP_TIME_OUT: StaticMut<Option<Duration>> = 
    // StaticMut::new(Some(Duration::new(unsafe{HTTP_TIME_OUT_SEC} * 1000, 0)));
    StaticMut::new(None);
}
pub fn http_timeout_sec() -> Option<&'static u64> {
    HTTP_TIME_OUT
        .as_ref()
        .map(|_| unsafe { &HTTP_TIME_OUT_SEC })
}
pub fn http_timeout() -> Option<&'static Duration> {
    HTTP_TIME_OUT.as_ref().as_ref()
}
pub fn http_timeout_set<T: Into<u64>>(time_out: Option<T>) {
    let tmp = HTTP_TIME_OUT.as_mut();
    *tmp = time_out
        .map(|t| t.into())
        .map(|s| {
                 unsafe {
                     HTTP_TIME_OUT_SEC = s;
                 }
                 Some(Duration::new(s * 1000, 0))
             })
        .unwrap_or(None);
}
lazy_static! {
   static ref UPTIME:Time = Time::now(); 
}
pub fn uptime() -> &'static Time {
    &UPTIME
}

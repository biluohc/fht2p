extern crate sig;
use sig::set_signal_handler;

extern crate libc;
#[allow(unused_imports)]
use libc::{c_int, c_uint};

use std::thread::sleep;
use std::time::Duration;

#[cfg(unix)]
static SIGNAL_NUM: c_int = 2;
#[cfg(windows)]
static SIGNAL_NUM: c_int = 0;

static mut SIG_STATUS: bool = false;

/// Watching the Signal
pub fn watch() {
    unsafe {
        let _ = set_signal_handler(SIGNAL_NUM, handler);
    }
}
/// Will get `true` if the Signal being watching and it occurs
pub fn get() -> bool {
    unsafe { SIG_STATUS }
}
/// Update the value of Signal to `false`
pub fn clean() {
    unsafe {
        SIG_STATUS = false;
    }
}
/// Watching and waiting the Signal(100ms)
pub fn join() {
    join_ms(100);
}

/// Watching and waiting the Signal with time(ms)
pub fn join_ms(ms: u64) {
    watch();
    while !get() {
        sleep(Duration::from_millis(ms));
    }
}

#[cfg(unix)]
extern "C" fn handler(_: c_int) {
    unsafe {
        SIG_STATUS = true;
    }
}

#[cfg(windows)]
extern "system" fn handler(msg: c_uint) -> c_int {
    if msg as c_int == SIGNAL_NUM {
        unsafe {
            SIG_STATUS = true;
        }
        1
    } else {
        0
    }
}


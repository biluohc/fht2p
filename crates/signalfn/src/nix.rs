pub extern crate libc;

use libc::{sigaction, sigfillset, sighandler_t};
pub use libc::{c_int, SIGINT}; //i32

use std::{mem, ptr};
use std::io::{self, Error};

pub fn register_signalfn(signal: c_int, handler: extern "C" fn(c_int)) -> io::Result<()> {
    unsafe {
        let mut sigset = mem::uninitialized();
        let state = sigfillset(&mut sigset);
        if state == 0 {
            let mut action: sigaction = mem::zeroed();
            action.sa_mask = sigset;
            action.sa_sigaction = handler as sighandler_t;
            if sigaction(signal, &action, ptr::null_mut()) == 0 {
                return Ok(());
            }
        }
        Err(Error::last_os_error())
    }
}

pub fn register_ctrlcfn(handler: extern "C" fn(c_int)) -> io::Result<()> {
    register_signalfn(SIGINT, handler)
}

#[macro_export]
macro_rules! signalfn {
    ($signal:expr, $name: ident, $call_back: ident) => (
        extern "C" fn $name(sig: $crate::c_int) {
            assert_eq!(sig, $signal);
            $call_back();
        }
    )
}

#[macro_export]
macro_rules! ctrlcfn {
    ($name: ident, $call_back: ident) => (
        signalfn!($crate::SIGINT, $name, $call_back);
    )
}

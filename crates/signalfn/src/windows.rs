// PS：Win10上 Powershell 不能正确 exit, echo $? 一直为 false, C版都这样。。
// http://andylin02.iteye.com/blog/661431
// https://docs.microsoft.com/zh-cn/windows/console/setconsolectrlhandler

pub extern crate winapi;

use winapi::um::consoleapi::SetConsoleCtrlHandler;
pub use winapi::shared::minwindef::{BOOL, TRUE, FALSE}; // i32
pub use winapi::shared::minwindef::DWORD; // u32
pub use winapi::um::wincon::CTRL_C_EVENT; // u32

use std::io::{self, Error};

pub fn register_signalfn(handler: extern "system" fn( signal: DWORD) -> BOOL,  add: BOOL) -> io::Result<()> {
    if unsafe { SetConsoleCtrlHandler(Some(handler), add) } == TRUE {
        Ok(())
    } else {
        Err(Error::last_os_error())
    }
}

pub fn register_ctrlcfn(handler: extern "system" fn( signal: DWORD) -> BOOL)-> io::Result<()> {
    register_signalfn(handler, TRUE)
}

#[macro_export]
macro_rules! signalfn {
    ($signal:expr, $name: ident, $call_back: ident) => (
        extern "system" fn $name(sig: $crate::DWORD) -> $crate::BOOL {
            if sig == $signal {
                $call_back();
                $crate::TRUE
            } else {
                $crate::FALSE
            }
        }
    )
}

#[macro_export]
macro_rules! ctrlcfn {
    ($name: ident, $call_back: ident) => (
        signalfn!($crate::CTRL_C_EVENT, $name, $call_back);
    )
}
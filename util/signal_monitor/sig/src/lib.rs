pub use platform::set_signal_handler;

extern crate libc;

#[cfg(unix)]
mod platform {
    use super::libc;
    use self::libc::{c_int, sigaction, sighandler_t, sigfillset};
    use std::{mem, ptr};

    pub unsafe fn set_signal_handler(signal: c_int, handler: unsafe extern "C" fn(c_int)) {
        let mut sigset = mem::uninitialized();
        if sigfillset(&mut sigset) != -1 {
            let mut action: sigaction = mem::zeroed();
            action.sa_mask = sigset;
            action.sa_sigaction = handler as sighandler_t;
            sigaction(signal, &action, ptr::null_mut());
        }
    }
}

#[cfg(windows)]
mod platform {
    extern crate kernel32;
    extern crate winapi;
    
    use self::kernel32::SetConsoleCtrlHandler;
    #[allow(unused_imports)]
    use self::winapi::{TRUE, FALSE};
    
    use super::libc::{c_int, c_uint};

    pub unsafe fn set_signal_handler(_: c_int, handler: unsafe extern "system" fn(c_uint) -> c_int) -> c_int {
        SetConsoleCtrlHandler(Some(handler), TRUE)
    }
}

// PS： win部分不能正确拦截 Signal, powershell echo $? 一直为false,C版都这样。。
// use self::winapi::wincon::*;
// CTRL_C_EVENT - 当用户按下了CTRL+C,或者由GenerateConsoleCtrlEvent API发出.
// CTRL_BREAK_EVENT - 用户按下CTRL+BREAK, 或者由GenerateConsoleCtrlEvent API发出.
// CTRL_CLOSE_EVENT - 当试图关闭控制台程序，系统发送关闭消息。
// CTRL_LOGOFF_EVENT - 用户退出时，但是不能决定是哪个用户.
// CTRL_SHUTDOWN_EVENT - 当系统被关闭时.
// CTRL_C_EVENT, CTRL_CLOSE_EVENT,CTRL_BREAK_EVENT,CTRL_LOGOFF_EVENT,CTRL_SHUTDOWN_EVENT: 0/2/1/5/6
// FALSE/TRUE: 0/1

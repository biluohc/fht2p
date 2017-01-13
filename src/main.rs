#![feature(link_args)]
extern crate urlparse;
use urlparse::quote;
use urlparse::unquote;
extern crate  chrono;
extern crate sys_info;

extern crate poolite;
#[macro_use]
extern crate stderr;

extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread::sleep;
use std::sync::Arc;
use std::process;

mod server;
use server::fht2p;

fn main() {
    match fht2p() {
        Ok(..) => {}
        Err(e) => {
            match e.as_ref() {
                "" => return,
                _ => {
                    errln!("{}", e);
                    process::exit(1);
                }
            }
        }
    };

    let waiting = Arc::new(AtomicBool::new(true));
    let wait = waiting.clone();
    ctrlc::set_handler_with_polling_rate(move || {
                                             wait.store(false, Ordering::SeqCst);
                                         },
                                         Duration::from_millis(100));
    while waiting.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(100)); // 100 ms
    }

    #[cfg(windows)]
    #[path = "win_ico.rs"]
    mod win_ico;
    // exe的图标。非Windows平台不需要rc资源。
}

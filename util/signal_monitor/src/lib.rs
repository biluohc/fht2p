extern crate sig;
use sig::set_signal_handler;

extern crate libc;
use libc::c_int;

use std::thread::sleep;
use std::time::Duration;

static mut SIGS_STATUS: [bool; 64] = [false; 64];
/// Watching the Signal
pub fn watch(num: usize) {
    idx_test(num);
    unsafe {
        set_signal_handler(num as c_int, handler);
    }
}
/// Will get `true` if the Signal being watching and it occurs
pub fn get(num: usize) -> bool {
    idx_test(num);
    unsafe { SIGS_STATUS[num - 1] }
}
/// Update the value of Signal to `false`
pub fn clean(num: usize) {
    idx_test(num);
    unsafe {
        SIGS_STATUS[num - 1] = false;
    }
}
/// Watching and waiting the Signal(100ms)
pub fn join(num: usize) {
    join_ms(num, 100);
}

/// Watching and waiting the Signal with time(ms)
pub fn join_ms(num: usize, ms: u64) {
    watch(num);
    while !get(num) {
        sleep(Duration::from_millis(ms));
    }
}
fn idx_valid(num: usize) -> bool {
    num > 0 && unsafe { num + 1 <= SIGS_STATUS.len() }
}
fn idx_test(num: usize) {
    if !idx_valid(num) {
        panic!("Signal idx is invalid: {} of '{}~{}'",
               num,
               1,
               unsafe { SIGS_STATUS.len() });
    }
}

extern "C" fn handler(msg: c_int) {
    unsafe {
        SIGS_STATUS[msg as usize - 1] = true;
    }
}

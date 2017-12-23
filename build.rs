extern crate encoding;
extern crate chardet;
extern crate time;

use encoding::label::encoding_from_whatwg_label;
use chardet::{detect, charset2encoding};
use encoding::DecoderTrap;
use time::now_utc;

use std::process::Command as Cmd;
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs::File;
use std::env;

/// `include!(concat!(env!("OUT_DIR"), "/zipcs.txt"));`
fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_path = out_dir.join("fht2p.txt");
    File::create(&out_path)
        .and_then(|mut f| f.write_all(fun().as_bytes()))
        .unwrap()
}

fn fun() -> String {
    let rustc = rustc_version()
        .map(|s| format!(" rustc{}", s.split(' ').nth(1).unwrap()))
        .unwrap_or_default();
    let git = commit_hash()
        .map(|s| (&s[0..8]).to_string())
        .and_then(|s| branch_name().map(|b| format!("{}@{}{} ", s, b, rustc)))
        .unwrap_or_default();

    let version = format!("{} ({}{})", env!("CARGO_PKG_VERSION"), git, date_time());
    format!("pub const VERSION: &str = \"{}\";", version)
}

fn date_time() -> String {
    now_utc()
    // .strftime("%Y-%m-%d/%I:%M:%SUTC")
    .strftime("%Y-%m-%dUTC")
    .map(|dt| dt.to_string())
    .unwrap_or_default()
}

fn commit_hash() -> io::Result<String> {
    Cmd::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|o| decode_utf8_unchecked(o.stdout))
        .map(|s| s.trim().to_string())
}

fn branch_name() -> io::Result<String> {
    Cmd::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|o| decode(o.stdout.as_slice()).trim().to_string())
}

fn rustc_version() -> io::Result<String> {
    Cmd::new("rustc").arg("--version").output().map(|o| {
        decode_utf8_unchecked(o.stdout).trim().to_string()
    })
}

fn decode_utf8_unchecked(bytes: Vec<u8>) -> String {
    unsafe { String::from_utf8_unchecked(bytes) }
}

fn decode(bytes: &[u8]) -> String {
    encoding_from_whatwg_label(charset2encoding(&detect(bytes).0))
        .and_then(|code| code.decode(bytes, DecoderTrap::Strict).ok())
        .unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned().to_owned())
}
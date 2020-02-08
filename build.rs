extern crate chrono;
extern crate rsass;

use chrono::offset::Utc;
use rsass::{compile_scss_file, OutputStyle};

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as Cmd;

const VERSION_FILE_NAME: &str = "fht2p.txt";
const CSS_PATH: &str = "templates/fht2p.css";
const CSS_FILE_NAME: &str = "fht2p.css";

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    css(&out_dir).unwrap();
    version(&out_dir).unwrap();
}

// use sass to compress css file
fn css(out_dir: &PathBuf) -> io::Result<()> {
    let css = compile_scss_file(Path::new(CSS_PATH), OutputStyle::Compressed).unwrap();
    let out_path = out_dir.join(CSS_FILE_NAME);
    File::create(&out_path).and_then(|mut f| f.write_all(css.as_slice()))
}

// pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/fht2p.txt"));
fn version(out_dir: &PathBuf) -> io::Result<()> {
    let out_path = out_dir.join(VERSION_FILE_NAME);
    File::create(&out_path).and_then(|mut f| f.write_all(fun().as_bytes()))
}

fn fun() -> String {
    let rustc = rustc_version()
        .map(|s| format!(" rustc{}", s.split(' ').nth(1).unwrap()))
        .unwrap_or_default();
    let git = commit_hash()
        .and_then(|s| branch_name().map(|b| format!("{}@{}{} ", s, b, rustc)))
        .unwrap_or_default();

    format!("{} ({}{})", env!("CARGO_PKG_VERSION"), git, date_time())
}

// date --help
fn date_time() -> String {
    // Utc::now().format("%Y-%m-%dUTC").to_string()
    Utc::now().format("%Y-%m-%d~%H:%M:%SUTC").to_string()
}

// git describe --always --abbrev=10 --dirty=-modified
fn commit_hash() -> io::Result<String> {
    Cmd::new("git")
        .args(&["describe", "--always", "--abbrev=8", "--dirty=-modified"])
        .output()
        .map(|o| decode(&o.stdout))
        .map(|s| s.trim().to_string())
}

fn branch_name() -> io::Result<String> {
    Cmd::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|o| decode(&o.stdout).trim().to_string())
}

fn rustc_version() -> io::Result<String> {
    Cmd::new("rustc")
        .arg("--version")
        .output()
        .map(|o| decode(&o.stdout).trim().to_string())
}

fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

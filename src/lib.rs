// #![allow(unknown_lints)]
// #![allow(clippy::all)]
// #[rustfmt::skip]
#[macro_use]
pub extern crate nonblock_logger;
// #[macro_use]
extern crate clap;
extern crate nom;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;

pub use std::process::exit as process_exit;

pub mod how {
    pub type Error = anyhow::Error;
    pub type Result<T> = anyhow::Result<T>;
}

pub mod args;
pub mod base;
pub mod config;
pub mod consts;
pub mod contentype;
pub mod handlers;
pub mod logger;
pub mod middlewares;
pub mod service;
pub mod stat;
pub mod tools;
pub mod typemap;
pub mod views;

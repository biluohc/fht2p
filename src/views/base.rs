use askama::Template;

use crate::consts;

use std;
use std::net::SocketAddr;

#[derive(Template)]
// #[template(path = "base.html", print = "code", escape= "none")]
#[template(path = "base.html", print = "none", escape = "none")]
pub struct BaseTemplate<'a> {
    pub css: &'a str,
    pub title: &'a str,
    pub h1: &'a str,
    pub parent: &'a str,
    pub client: &'a SocketAddr,
    pub server: &'a SocketAddr,
    pub url: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub os: &'a str,
    pub arch: &'a str,
    pub upload: bool,
    pub mkdir: bool,
}
impl<'a> BaseTemplate<'a> {
    pub fn new(title: &'a str, h1: &'a str, parent: &'a str, client: &'a SocketAddr, upload: bool, mkdir: bool) -> Self {
        BaseTemplate {
            title,
            h1,
            parent,
            client,
            upload,
            mkdir,
            css: include_str!(concat!(env!("OUT_DIR"), "/fht2p.css")),
            server: consts::SERVER_ADDR.get(),
            url: consts::URL,
            name: consts::NAME,
            version: env!("CARGO_PKG_VERSION"),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        }
    }
}

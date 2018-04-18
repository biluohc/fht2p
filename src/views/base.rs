use askama::Template;

use consts;

use std::net::SocketAddr;
use std;

#[derive(Template)]
// #[template(path = "base.html", print = "code", escape= "none")]
#[template(path = "base.html", print = "none", escape= "none")]
pub struct BaseTemplate<'a> {
    css: &'a str,
    title: &'a str,
    h1: &'a str,
    parent: &'a str,
    client: &'a SocketAddr,
    server: &'a SocketAddr,
    url: &'a str,
    name: &'a str,
    version: &'a str,
    os: &'a str,
    arch: &'a str,
}
impl<'a> BaseTemplate<'a>{
    pub fn new(title: &'a str, h1: &'a str, parent: &'a str, client: &'a SocketAddr ) -> Self {
        BaseTemplate {
            title,
            h1,
            parent,
            client,
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
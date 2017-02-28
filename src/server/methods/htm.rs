use super::consts::*;
use super::html5::{HTML, TagSingle, TagDouble};
use std::env;

// <head><meta charset=UTF-8"><title>/cargo/doc/date/</title><link rel="shortcut icon" type="image/x-icon" href="/favicon.ico"><link rel="stylesheet" type="text/css" href="/style.css"></head>
pub fn head() -> TagDouble {
    HTML::head().push(TagSingle::new("link").add_attrs(vec![("rel", "stylesheet"), ("type", "text/css"), ("href", CSS_PATH)].iter()))
}

// <address><a href="https://github.com/biluohc/fht2p">fht2p</a>/0.6.1 (linux/x86_64) Server at <a href="http://127.0.0.1:8080">127.0.0.1:8080</a></address>
pub fn address(server_addr: &str) -> TagDouble {
    TagDouble::new("address")
    .push(TagDouble::new("a").add_attr("href",URL).push(NAME))
    .push(format!("/{} ({}/{}) server at ",VERSION,env::consts::OS,env::consts::ARCH))
    //注意泛型是一个，如果匹配到一个str和一个String,后者会错误,Fucking。html5的两个S得拆为两个泛型。
    .push(TagDouble::new("a").add_attr("href",format!("http://{}",server_addr)).push(server_addr))
    .push(TagDouble::new("script").add_attrs(vec![("type", "text/javascript"), ("src", JS_PATH)].iter()))
}


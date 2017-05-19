use super::statics::*;
use client::status_code::StatusCode;
use super::path_info::PathInfo;
use super::HTTP_PROTOCOL;

use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;
use std::env::consts::*;

use maud::{DOCTYPE, PreEscaped};

/// `<head>...</head>`
///```html
/// <head>
///<meta charset="UTF-8"><link rel="shortcut icon" type="image/x-ico"
///href="/favicon.ico"><link rel="stylesheet" type="text/css" href="/fht2p.css">
///<title>/</title>
///</head>
///```
pub fn head<T>(title: &T) -> PreEscaped<String>
    where T: Display
{
    html!{
     head {
            meta  charset="UTF-8" /
            link rel="shortcut icon" type="image/x-ico" href="/favicon.ico" /
            link rel="stylesheet" type="text/css" href="/fht2p.css" /
            title (title)
         }
    }
}

/// `<h1>/<span id="client">127.0.0.1:59594</span><p></h1>`
pub fn h1<T>(title: &T, client_addr: &SocketAddr) -> PreEscaped<String>
    where T: Display
{
    html!{
        h1 {
            (title)
            span id="client" (client_addr)
            p {}
        }
    }
}
/// `address><a href="https://github.com/biluohc/fht2p">fht2p</a>/0.7.0 (linux/x86_64) server at <a href="http://127.0.0.1:8080">127.0.0.1:8080</a></address>`
pub fn address(server_addr: &SocketAddr) -> PreEscaped<String> {
    html! {
    address {
        a href={(URL)} (NAME)
        "/"(VERSION) "("(OS)"/"(ARCH)") server at " 
        a  href={(HTTP_PROTOCOL)"://"(server_addr)} (server_addr) 
    }
 }
}
/// `<script type="text/javascript" src="/fht2p.js"></script>`
///
/// But `html! { script  type="text/javascript" src=(JS_PATH) / }` -> `<script ...>` is wrong
pub fn script() -> PreEscaped<String> {
    html ! {
       (PreEscaped(format!(r#"<script type="text/javascript" src="{}"></script>"#,JS_PATH)))
    }
}
/// `code(404,...)` to `html`
pub fn code(code: &StatusCode, client_addr: &SocketAddr, server_addr: &SocketAddr) -> PreEscaped<String> {
    let title_h1 = format!("{}  {}", code.code(), code.desc());
    html!{
        (DOCTYPE)
        html {
             (head(&title_h1))
            body {
               (h1(&title_h1,client_addr))
            }
            (address(server_addr))
            (script())
    }
 }
}

#[derive(Debug)]
pub enum Class {
    Dir,
    File,
    Link,
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Class::Dir => f.write_str("dir"),
            Class::File => f.write_str("file"),
            Class::Link => f.write_str("link"),
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub class: Class,
    pub link: String,
    pub name: String,
    pub info: PathInfo,
}
impl Entry {
    pub fn new<S1, S2>(class: Class, link: S1, name: S2, info: PathInfo) -> Self
        where S1: Into<String>,
              S2: Into<String>
    {
        Entry {
            class: class,
            link: link.into(),
            name: name.into(),
            info: info,
        }
    }
    pub fn class(&self) -> &Class {
        &self.class
    }
    pub fn link(&self) -> &str {
        &self.link
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn info(&self) -> &PathInfo {
        &self.info
    }
}
pub struct Dir {
    head: PreEscaped<String>, //title
    h1: PreEscaped<String>, //h1, client_addr
    address: PreEscaped<String>, // server_addr
    entries: Vec<Entry>,
}

impl Dir {
    pub fn new<T>(title_h1: T, client_addr: &SocketAddr, server_addr: &SocketAddr) -> Self
        where T: Display
    {
        Dir {
            head: head(&title_h1),
            h1: h1(&title_h1, client_addr),
            address: address(server_addr),
            entries: Vec::new(),
        }
    }
    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
    pub fn into_maud(self) -> PreEscaped<String> {
        html!{
        (DOCTYPE)
        html {
             (self.head)
            body {
               (self.h1)
               table id="table" {
                   thead {
                       tr style="border-bottom: 0.1px solid #000080;" {
                           th {
                               button onclick="sort_by(0)" "Name"
                           }
                           th {
                               button onclick="sort_by(1)" "Last_modified"
                           }
                            th {
                               button onclick="sort_by(2)" "Size"
                           }                  
                       }
                   }
                    tbody {
                            @for entry in self.entries {
                        tr {
                                td  class=(entry.class()) {
                                    a  href=(entry.link()) (entry.name())
                                }
                                td data=(entry.info().modified()) (entry.info().modified())
                                td data=(entry.info.len()) (entry.info().len())   
                            }
                        }
                    }
               }
               hr /
            }
            (self.address)
            (script())
        }
    }
    }
}

//  cargo t p html
#[test]
fn html() {
    let html = html!{
        (DOCTYPE)
        html {
             (head(&"HEAD"))
            body {
               (h1(&"H1", &"127.0.0.1:59594".parse().unwrap()))
            }
            (address(&"127.0.0.1:8080".parse().unwrap()))
            (script())
    }
    };
    errln!("{}", html.into_string());
}

#[test]
fn html_code() {
    errln!("{}",
           code(&StatusCode::new(404_u16),
                &"127.0.0.1:59594".parse().unwrap(),
                &"127.0.0.1:80".parse().unwrap())
                   .into_string());
}

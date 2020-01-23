use askama::Template;

use super::base::*;

use std::net::SocketAddr;

#[derive(Template)]
// #[template(path = "error.html", print = "code", escape= "none")]
#[template(path = "error.html", print = "none", escape = "none")]
pub struct ErrorTemplate<'a> {
    _parent: BaseTemplate<'a>,
}

impl<'a> ErrorTemplate<'a> {
    pub fn new(title: &'a str, h1: &'a str, parent: &'a str, client: &'a SocketAddr) -> Self {
        ErrorTemplate {
            _parent: BaseTemplate::new(title, h1, parent, client, false, false),
        }
    }
}

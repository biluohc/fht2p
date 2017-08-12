use super::*;
use self::statics::*;
/// `GET` and `HEAD` Methods
pub mod get;
use self::get::*;
/// Get `Path`'s `Info`
pub mod path_info;
/// `HTML` depends on `Maud`
pub mod html;
use server::content_type::ContentType;

use std::io::{self, Write};
use std::error::Error;
use std::fs::File;
use std::mem;

/// `HTTP/1.1 200 OK`
#[derive(Debug,Clone)]
pub struct StatusLine {
    pub protocol: String,
    pub version: String,
    pub code: StatusCode,
}

impl StatusLine {
    pub fn protocol(&self) -> &str {
        &self.protocol
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn code(&self) -> &StatusCode {
        &self.code
    }
}

impl Default for StatusLine {
    fn default() -> Self {
        Self {
            protocol: HTTP_PROTOCOL.to_owned(),
            version: HTTP_VERSION.to_owned(),
            code: StatusCode::default(),
        }
    }
}
/// HTTP/1.1 200 OK\r\n`Headers`\r\n\r\n`Body`
#[derive(Debug)]
pub struct Response {
    pub line: StatusLine,
    pub header: Map<String, String>,
    /// file/dir/static_file
    pub content: Content,
}

impl Response {
    pub fn code(&self) -> u16 {
        self.line.code.code()
    }
    pub fn code_set<C: Into<u16>>(&mut self, code: C) {
        self.line.code.set(code);
    }
    pub fn line(&self) -> &StatusLine {
        &self.line
    }
    pub fn header(&self) -> &Map<String, String> {
        &self.header
    }
    pub fn header_insert<S1: Into<String>, S2: Into<String>>(&mut self, key: S1, value: S2) {
        self.header.insert(key.into(), value.into());
    }
    pub fn content_type_insert(&mut self, content_type: ContentType) {
        self.header
            .insert("Content-Type".to_owned(), content_type.to_string());
    }
    pub fn content_length_insert(&mut self) {
        self.header
            .insert("Content-Length".to_owned(),
                    format!("{}", self.content.len()));
    }
    pub fn get(&mut self, req: &Request) {
        get(self, req)
    }

    pub fn write(self, mut stream: &mut TcpStream, req: &Request) -> io::Result<()> {
        write!(&mut stream,
               "{}/{} {} {}\r\n",
               self.line.protocol(),
               self.line.version(),
               self.line.code().code(),
               self.line.code().desc())?;
        for (k, v) in self.header() {
            write!(&mut stream, "{}: {}\r\n", k, v)?;
        }
        write!(&mut stream, "\r\n")?;
        if req.line().method() == "HEAD" {
            return Ok(());
        }
        self.content.write(&mut stream)
    }
}

impl Default for Response {
    fn default() -> Self {
        let mut header: Map<String, String> = Map::new();
        header.insert("Server".to_owned(), format!("{}/{}", NAME, VERSION));
        if let Some(s) = http_timeout_sec() {
            header.insert("Connection".to_owned(), "keep-alive".to_owned());
            header.insert("keep-alive".to_owned(), format!("{}", s));
        } else {
            header.insert("Connection".to_owned(), "close".to_owned());
        }
        Response {
            line: StatusLine::default(),
            header: header,
            content: Content::default(),
        }
    }
}
/// `Response`'s `Content`
#[derive(Debug)]
pub enum Content {
    /// dir,oher status -> `String` -> `Vec<u8>`
    Str(String),
    /// file
    File(File),
    /// static file -> `&[u8]`
    Sf(&'static [u8]),
}

#[allow(unknown_lints,len_without_is_empty)]
impl Content {
    pub fn update(&mut self, mut other: Content) {
        mem::swap(self, &mut other);
    }
    pub fn len(&self) -> u64 {
        fn file_lenth(file: &File) -> u64 {
            file.metadata()
                .map(|l| l.len())
                .map_err(|e| {
                             (errln!("{}_Warning@file_lenth(): {}@{:?}",
                                     NAME,
                                     e.description(),
                                     file))
                         })
                .unwrap()
        }
        match *self {
            Content::Str(ref x) => x.len() as u64,
            Content::File(ref y) => file_lenth(y),
            Content::Sf(z) => z.len() as u64,
        }
    }
    pub fn write(self, mut stream: &mut TcpStream) -> io::Result<()> {
        match self {
            Content::Str(x) => {
                stream.write_all(x.as_bytes())?;
            }
            Content::File(mut y) => {
                io::copy(&mut y, stream)?;
            }
            Content::Sf(z) => {
                stream.write_all(z)?;
            }
        };
        stream.flush()?;
        Ok(())
    }
}

impl Default for Content {
    fn default() -> Self {
        Content::Str(String::with_capacity(0))
    }
}

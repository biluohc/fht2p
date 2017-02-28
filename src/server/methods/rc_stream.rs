use super::ArcConfig;
use super::Date;
use super::consts::{NAME, BUFFER_SIZE, TIME_OUT};
use super::stderr::Loger;

use std::net::TcpStream;
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::error::Error;

#[derive(Debug,Clone)]
pub struct RcStream {
    pub arc_config: Arc<ArcConfig>,
    time: Date,
    keep_alive_time: SystemTime,
    server_addr: String,
    client_addr: String,
}

impl RcStream {
    pub fn new(arc_config: Arc<ArcConfig>, server_addr: String, client_addr: String) -> RcStream {
        RcStream {
            arc_config: arc_config,
            time: Date::now(),
            keep_alive_time: SystemTime::now() + Duration::new(TIME_OUT, 0),
            server_addr: server_addr,
            client_addr: client_addr,
        }
    }
    #[inline]
    pub fn keep_alive(&self) -> bool {
        self.arc().keep_alive()
    }
    #[inline]
    pub fn time_out(&self) -> bool {
        self.keep_alive_time > SystemTime::now()
    }
    #[inline]
    pub fn arc(&self) -> &Arc<ArcConfig> {
        &self.arc_config
    }
    #[inline]
    pub fn time(&self) -> &Date {
        &self.time
    }
    #[inline]
    pub fn server_addr(&self) -> &str {
        self.server_addr.as_ref()
    }
    #[inline]
    pub fn client_addr(&self) -> &str {
        self.client_addr.as_ref()
    }
}

#[derive(Debug)]
pub struct Line {
    pub method: String,
    pub protocol: String,
    pub version: String,
}
#[derive(Debug)]
pub struct Request {
    pub line: Line,
    pub header: HashMap<String, String>,

    pub path_raw: String, // 原始未解码,未补全的path.
    pub path_vp: String, // 补全了，解码了的虚拟路径。
    pub path_rp: String, // 对应的真实路径。如果没有就是String::new()
    pub status: u32, // 状态码，默认200.
}
impl Request {
    pub fn new(method: String,
               protocol: String,
               version: String,
               header: HashMap<String, String>,
               path_raw: String,
               path_vp: String,
               path_rp: String,
               status: u32)
               -> Self {
        Request {
            line: Line {
                method: method,
                protocol: protocol,
                version: version,
            },
            header: header,
            path_raw: path_raw,
            path_vp: path_vp,
            path_rp: path_rp,
            status: status,
        }
    }
    pub fn method(&self) -> String {
        self.line.method.clone()
    }
    pub fn protocol(&self) -> String {
        self.line.protocol.clone()
    }
    pub fn version(&self) -> String {
        self.line.version.clone()
    }
    pub fn path_raw(&self) -> &String {
        &self.path_raw
    }
    pub fn path_vp(&self) -> &String {
        &self.path_vp
    }
    pub fn path_rp(&self) -> &String {
        &self.path_rp
    }
    pub fn status(&self) -> &u32 {
        &self.status
    }
    pub fn status_set(&mut self, status: u32) {
        self.status = status;
    }
    pub fn header(&self) -> &HashMap<String, String> {
        &self.header
    }
}
#[derive(Debug)]
pub struct Status {
    pub protocol: String,
    pub version: String,
    pub code: u32,
    pub name: String,
}

#[derive(Debug)]
pub struct Response {
    pub status: Status,
    pub header: HashMap<String, String>,
    pub content: Content, /* file/dir/satic --其它状态吗是其它的 match code,由函数+HashMap生成html。 */
}
// response_type: String,
// content_type: String,
// content_lenth: u64,
impl Response {
    pub fn new(protocol: String, version: String, code: u32, name: String, header: HashMap<String, String>, content: Content) -> Self {
        Response {
            status: Status {
                protocol: protocol,
                version: version,
                code: code,
                name: name,
            },
            header: header,
            content: content,
        }
    }
    pub fn write_response(self, mut stream: &mut TcpStream) {
        fn write_response(msg: Response, mut stream: &mut TcpStream) -> Result<(), io::Error> {
            write!(&mut stream,
                   "{}/{} {} {}\r\n",
                   msg.status.protocol,
                   msg.status.version,
                   msg.status.code,
                   msg.status.name)?;
            for (k, v) in &msg.header {
                write!(&mut stream, "{}: {}\r\n", k, v)?;
            }
            write!(&mut stream, "\r\n")?;
            msg.content.write_content(&mut stream);
            Ok(())
        }
        if let Err(e) = write_response(self, &mut stream) {
            dbstln!("{}_Warning@{}::write_response(): {}",
                    NAME,
                    module_path!(),
                    e.description());
        };
    }
}

#[derive(Debug)]
pub enum Content {
    Str(Vec<u8>), // dir,oher status->String->Vec<u8>
    File(fs::File), // file 变为reader<file>
    Sf(&'static [u8]), // static file->&[u8]
}


impl Content {
    pub fn len(&self) -> u64 {
        fn file_lenth(file: &fs::File) -> u64 {
            match file.metadata() {
                Ok(ok) => ok.len(),
                Err(e) => {
                    dbstln!("{}_Warning@{}::file_lenth(): {}@{:?}",
                            NAME,
                            module_path!(),
                            e.description(),
                            file);
                    unreachable!();
                }
            }
        }
        match *self {
            Content::Str(ref x) => x.len() as u64,
            Content::File(ref y) => file_lenth(y),
            Content::Sf(z) => z.len() as u64,
        }
    }
    pub fn write_content(self, mut stream: &mut TcpStream) {
        if let Err(e) = write_content_result(self, &mut stream) {
            dbstln!("{}_Warning@{}::write_content(): {}",
                    NAME,
                    module_path!(),
                    e.description());
        };
    }
}

fn write_content_result(content: Content, mut stream: &mut TcpStream) -> Result<(), io::Error> {
    match content {
        Content::Str(x) => {
            let _ = stream.write(&x)?;
        }
        Content::File(mut y) => {
            file_write_to_tcpstream(&mut y, &mut stream)?;
        }
        Content::Sf(z) => {
            let _ = stream.write(z)?;
        }
    };
    stream.flush()?;
    Ok(())
}

fn file_write_to_tcpstream(file: &mut fs::File, stream: &mut TcpStream) -> Result<(), io::Error> {
    let mut stream = BufWriter::with_capacity(BUFFER_SIZE, stream);
    let file = BufReader::with_capacity(BUFFER_SIZE, file);
    for byte in file.bytes() {
        let byte = byte?;
        stream.write_all(&[byte])?;
    }
    stream.flush()?;
    Ok(())
}

use stderr::Loger;
use urlparse::unquote;

use super::*;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::mem;

const LINE_SEP: u8 = b'\n';
/// `GET /hello.txt HTTP/1.1`
#[derive(Debug,Clone)]
pub struct RequestLine {
    pub method: String,
    /// 原始未解码,未补全的`path`
    pub path: String,
    pub protocol: String,
    pub version: String,
}
impl RequestLine {
    pub fn new(method: &str, path: &str, protocol: &str, version: &str) -> Self {
        RequestLine {
            method: method.to_owned(),
            path: path.to_owned(),
            protocol: protocol.to_owned(),
            version: version.to_owned(),
        }
    }
    pub fn method(&self) -> &str {
        &self.method
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn protocol(&self) -> &str {
        &self.protocol
    }
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// GET /hello.txt HTTP/1.1\r\n`Headers`\r\n\r\n`Body`
///
/// `User-Agent: Mozilla/5.0\r\nAccept-Encoding: gzip, deflate\r\nConnection: keep-alive`
#[derive(Debug,Clone)]
pub struct Request {
    pub line: RequestLine,
    pub header: Map<String, String>,
    /// `path` --> `img` <--> `rel`
    pub route: Option<Route>,
    pub client_addr: SocketAddr,
    pub server_addr: SocketAddr,
    pub time: Time,
}

impl Request {
    pub fn new(line: RequestLine, header: Map<String, String>, route: Option<Route>, client_addr: SocketAddr, server_addr: SocketAddr, now: Time) -> Self {
        Request {
            line: line,
            header: header,
            route: route,
            client_addr: client_addr,
            server_addr: server_addr,
            time: now,
        }
    }
    pub fn line(&self) -> &RequestLine {
        &self.line
    }
    pub fn header(&self) -> &Map<String, String> {
        &self.header
    }
    pub fn route(&self) -> Option<&Route> {
        self.route.as_ref()
    }
    pub fn client_addr(&self) -> &SocketAddr {
        &self.client_addr
    }
    pub fn server_addr(&self) -> &SocketAddr {
        &self.server_addr
    }
    pub fn time(&self) -> &Time {
        &self.time
    }
    pub fn from_stream(stream: &mut TcpStream) -> io::Result<Self> {
        let now = Time::now();
        let (client_addr, server_addr) = (stream.peer_addr()?, stream.local_addr()?);

        let mut stream = BufReader::new(stream);
        let mut buf: Vec<u8> = Vec::new();

        // GET /hello.txt HTTP/1.1
        let _ = stream.read_until(LINE_SEP, &mut buf)?;
        let line = String::from_utf8_lossy(&buf[..]).into_owned();
        // req_line
        let req_line: Vec<&str> = line.split(' ')
            .filter(|s| !s.is_empty())
            .map(|x| x.trim())
            .collect();
        if req_line.len() != 3 {
            panic!("invalid req_line: {:?}", line); // 400
        }
        // protocol_version
        let pv: Vec<&str> = req_line[2]
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|x| x.trim())
            .collect();
        if pv.len() != 2 {
            panic!("invalid req_pv: {:?}", pv); // 400
        }

        let mut header = Map::new();
        loop {
            mem::swap(&mut buf, &mut Vec::new());
            stream.read_until(LINE_SEP, &mut buf)?;
            let line = String::from_utf8_lossy(&buf[..]).into_owned();
            if line.trim().is_empty() || !line.contains(':') {
                break;
            }
            let sep_idx = line.find(':').unwrap();
            let (key, value) = line.split_at(sep_idx);
            header.insert(key.trim().to_string(), value.trim().to_string());
        }
        dbstln!("header_Map:\n{:?}", header);

        let path_raw = req_line[1];
        dbstln!("path_raw: {:?}", path_raw);
        // 如果路径本身就存在，就不二次解码,(三次编码则会产生多余"/"等字符,不可能是真实路径。浏览器对URL只编码一次。
        let mut route = Route::parse(&path_raw);
        if route.is_none() {
            route = Route::parse(unquote(&path_raw).unwrap());
        }
        Ok(Request::new(RequestLine::new(req_line[0], path_raw, pv[0], pv[1]),
                        header,
                        route,
                        client_addr,
                        server_addr,
                        now))
    }
    pub fn into_client(self) -> Client {
        Client {
            req: self,
            resp: Response::default(),
        }
    }
}

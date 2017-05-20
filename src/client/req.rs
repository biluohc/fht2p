use stderr::Loger;
use urlparse::unquote;

use super::*;
use super::statics::BUFFER_SIZE;
use std::io::{Read, BufReader};
use std::net::SocketAddr;

/// `GET /hello.txt HTTP/1.1`
#[derive(Debug,Clone,Default)]
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
    pub is_bad: bool,
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
            is_bad: false,
        }
    }
    pub fn bad(line: RequestLine, client_addr: SocketAddr, server_addr: SocketAddr, now: Time) -> Self {
        let mut tmp = Self::new(line, Map::new(), None, client_addr, server_addr, now);
        tmp.is_bad = true;
        tmp
    }
    pub fn is_bad(&self) -> &bool {
        &self.is_bad
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
    /// Get `bytes`: `HTTP` `RequestLine` and `Header`
    pub fn read(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
        let mut reader = BufReader::new(stream);
        let mut req: Vec<u8> = Vec::new();
        let mut counter = 0;
        let mut buf = [0u8; 1];
        loop {
            let num = reader.read(&mut buf)?;
            if num == 0 {
                break;
            }
            let byte = buf[0];
            req.push(byte);
            if byte == b'\n' {
                counter += 1;
            } else if byte != b'\r' {
                counter = 0;
            }
            if counter >= 2 {
                break;
            }
        }
        Ok(req)
    }
    pub fn read_to_end(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
        let mut reader = BufReader::with_capacity(BUFFER_SIZE, stream);
        let mut req: Vec<u8> = Vec::new();
        let mut buf = [0u8; BUFFER_SIZE];
        loop {
            let num = reader.read(&mut buf)?;
            if num == 0 {
                break;
            }
            req.extend_from_slice(&buf[..num]);
        }
        Ok(req)
    }
    pub fn from_bytes(bytes: &[u8], client_addr: SocketAddr, server_addr: SocketAddr) -> Self {
        let now = Time::now();
        let str = String::from_utf8_lossy(&bytes[..]).into_owned();
        // GET /hello.txt HTTP/1.1
        // req_line
        let mut lines = str.trim().lines();
        let line = lines.next().unwrap();
        dbln!("ResquestLine_RAW: {:?}",line);
        let req_line: Vec<&str> = line.split(' ')
            .filter(|s| !s.is_empty())
            .map(|x| x.trim())
            .collect();
        if req_line.len() != 3 {
            errln!("Invalid ResquestLine: {:?}", req_line);
            return Self::bad(RequestLine::default(), client_addr, server_addr, now);
        }
        // protocol_version
        let pv: Vec<&str> = req_line[2]
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|x| x.trim())
            .collect();
        if pv.len() != 2 {
            errln!("Invalid ResquestLine's Protocol-Version: {:?}", req_line);
            let resquest_line = RequestLine::new(req_line[0], req_line[1], "", "");
            return Self::bad(resquest_line, client_addr, server_addr, now);
        }
        let mut header = Map::new();
        for line in lines {
            if !line.contains(':') {
                errln!("Invalid Header: {:?}", line);
                continue;
            }
            let sep_idx = line.find(':').unwrap();
            let (key, value) = line.split_at(sep_idx);
            if value.is_empty() {
                errln!("Invalid Header_value: {:?}", line);                
                continue;
            }
            header.insert(key.trim().to_string(), value[1..].trim().to_string());
        }
        dbstln!("Header_Map:\n{:?}", header);

        let path_raw = req_line[1];
        // 如果路径本身就存在，就不二次解码,(三次编码则会产生多余"/"等字符,不可能是真实路径。浏览器对URL只编码一次。
        let mut route = Route::parse(&path_raw);
        if route.is_none() {
            route = Route::parse(unquote(&path_raw).unwrap());
        }
        Request::new(RequestLine::new(req_line[0], path_raw, pv[0], pv[1]),
                     header,
                     route,
                     client_addr,
                     server_addr,
                     now)
    }
    pub fn into_client(self) -> Client {
        Client {
            req: self,
            resp: Response::default(),
        }
    }
}

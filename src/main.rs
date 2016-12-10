use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::io::Write;

fn main() {
    // 127.0.0.1只接收本地请求,取0.0.0.0 。
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    // Length是html文本长度，若是小于则截断，超过则"连接被重置"。
    // 注意换行符不能省略，否则也是"连接被重置"，响应头和正文之间还有一空行。
    // 注意用read_to_string和read_to_end读取浏览器无响应？Rust
    let send_head = "HTTP/1.1 200 OK\nContent-Type: text/html;charset=UTF-8\nContent-Length: ";
    let send_body = "<!DOCTYPE HTML><html><body>Hello Web!| 我 |Love Rust!</body></html>";
    let send = format!("{}{}\n\n{}", send_head, send_body.len(), send_body);

    let mut read: Vec<u8> = vec![0u8; 1024];
    let x = stream.read(&mut read);
    let y = stream.write(send.as_bytes());
    let z = stream.flush();

    let read = String::from_utf8_lossy(&read).into_owned();
    println!("{}", read.trim_right());
    println!("read: {:?} | write: {:?} | flush: {:?}\n", x, y, z);
}


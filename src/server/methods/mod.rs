use std::net::{TcpListener, TcpStream};
// use std::fs::{self, metadata, File};
use std::collections::HashMap;
use std::result::Result;
use std::time::Duration;
use std::io::prelude::*; // src/server/methods/get.rs:101::file.read_line()
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::io;

use super::*;
use self::consts::{NAME, VERSION};

use urlparse::{quote, unquote};
use poolite::Pool;

mod path; // dir/file修改时间和大小
mod htm; //html拼接
mod get;
use self::get::{get, header, other_status_code_to_resp};
mod rc_stream;
use self::rc_stream::{RcStream, Request, Response, Content};
use self::date::Date;

pub fn for_listener(listener: &TcpListener, config: Arc<ArcConfig>, pool: &Pool) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config = config.clone();
                // 一个进程一个堆，一个线程一个栈。
                // 栈大小，linux主默认8m，副线程和win一样 2m(这里给rust坑死了,一直stackover。以后要注意默认值)。
                pool.push(move || match_client(config, stream));
            }
            Err(e) => {
                errstln!("{:?}", e);
                // connection failed
            }
        };
        // errln!("empty: {}\ttasks_len: {}\tlen: {}",
        //        pool.is_empty(),
        //        pool.tasks_len(),
        //        pool.len());
    }
}

#[inline]
fn match_client(config: Arc<ArcConfig>, stream: TcpStream) {
    match deal_client(config, stream) {
        Ok(_) => {}
        Err(e) => errstln!("{}_Warning@TcpStream: {}", NAME, e.description()),
    };
}
fn deal_client(config: Arc<ArcConfig>, mut stream: TcpStream) -> Result<(), io::Error> {
    // client and server addr.
    let (server_addr, client_addr) = (format!("{}", stream.local_addr()?), format!("{}", stream.peer_addr()?));
    let time_out = Duration::new(TIME_OUT, 0);
    stream.set_read_timeout(Some(time_out))?;
    stream.set_write_timeout(Some(time_out))?;
    let rc_stream = Rc::from(RcStream::new(config, server_addr, client_addr));

    // println!("二分法 {:?}@{:?} {:?}@{:?}",
    //          rc_stream.time().as_str(),
    //          rc_stream.client_addr(),
    //          module_path!(),
    //          line!());
    loop {
        let rc_s = rc_stream.clone();
        let request_read = read_request(&mut stream)?;
        // 大小为0的请求？？？请求行都为空
        // 然后loop continue,高并发时 cpu 100%，醉了。
        if request_read.is_empty() {
            if rc_s.keep_alive() && !rc_s.time_out() {
                continue;
            } else {
                break;
            }
        }
        let mut req = to_request(&request_read[..], rc_s.clone());

        // write response.
        let resp = match req.method().as_str() {
            "GET" | "HEAD" => get(rc_s.clone(), &mut req),
            // post
            _ => {
                req.status_set(405);
                let map = header(rc_s.clone());
                other_status_code_to_resp(rc_s.clone(), &req, map)
            }
        };
        // 打印日志
        // 127.0.0.1--[2017-0129 21:11:59] 200 "GET /cargo/ HTTP/1.1" @ " "
        println!(r#"{}**[{}] {} "{} {} {}/{}" -> "{}""#,
                 rc_s.client_addr(),
                 rc_s.time().ls().trim(),
                 req.status,
                 req.method(),
                 req.path_raw(),
                 req.protocol(),
                 req.version(),
                 req.path_rp);

        resp.write_response(&mut stream, &req);
        dbstln!(); //分开各个请求，否则--log ''没法看。
        if !rc_s.keep_alive() || rc_s.time_out() {
            break;
        }
    }
    Ok(())
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    use std::io::BufReader;
    let reader = BufReader::new(stream);
    let mut req: Vec<u8> = vec![];
    let mut counter = 0;
    for byte in reader.bytes() {
        let byte = byte?;
        req.push(byte);
        if byte == 10 {
            // == 10 \n
            counter += 1;
        } else if byte != 13 {
            // !=13 \r
            counter = 0;
        }
        if counter == 2 {
            break;
        }
    }
    Ok(req)
}

fn to_request(vec: &[u8], rc_s: Rc<RcStream>) -> Request {
    // "GET /favicon.ico HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:50.0) Gecko/20100101 Firefox/50.0\r\nAccept: */*\r\nAccept-Language: en-US,en;q=0.5\r\nAccept-Encoding: gzip, deflate\r\nCookie: _ga=GA1.1.1204164507.1467299031\r\nConnection: keep-alive\r\n\r\n"
    let req_str = String::from_utf8_lossy(&vec[..]).into_owned();
    dbstln!("\n{}@{} req_raw:\n{:?}",
            module_path!(),
            rc_s.time().ls(),
            req_str);
    let mut req_str = req_str.lines();
    let line = req_str.next().unwrap();
    // 请求行
    let mut line: Vec<String> = line.split(' ').map(|x| x.to_string()).collect();
    // 协议/版本
    let mut pv: Vec<String> = line[2].split('/').map(|x| x.to_string()).collect();

    let mut header = HashMap::new();
    for ln in req_str {
        if let Some(s) = ln.find(':') {
            if ln.len() >= s + 3 {
                header.insert(ln[..s].to_string(), ln[s + 2..].to_string());
            }
        }
    }
    dbstln!("{}@{} header_HashMap:\n{:?}",
            module_path!(),
            rc_s.time().ls(),
            header);
    let url_raw = line[1].to_string();
    dbstln!("{}@{} url_raw: {:?}",
            module_path!(),
            rc_s.time().ls(),
            &line[1]);
    let url = url_handle_pre(&url_raw);
    dbstln!("{}@{} url_handle_pre(): {:?}",
            module_path!(),
            rc_s.time().ls(),
            &url);
    // 如果路径本身就存在，就不二次解码,(三次编码则会产生多余"/"等字符,不可能是真实路径。浏览器对URL只编码一次。
    // Option<(String, String)>  完全匹配才赋值，is_none() 用于是否继续寻找。
    let mut vp_rp_full_match: Option<(String, String)> = None;
    for static_path in rc_s.arc().sfs().keys() {
        // 静态文件,虚拟路径==真实路径。注意不需要解码，也就是没有空格等特殊字符。
        if url == *static_path {
            vp_rp_full_match = Some((url.clone(), url.clone()));
            break;
        }
    }
    if vp_rp_full_match.is_none() {
        url_to_vp_rp(rc_s.clone(), &url, &mut vp_rp_full_match);
    }
    let mut status = 200;
    let (vp, rp) = match vp_rp_full_match {
        Some(s) => s,
        None => {
            // 否则这样 "GET /home HTTP/1.1" 会直接后面访问到 home,TM太可怕。
            status = 404;
            (url_raw.clone(), url_raw.clone()) //没有就放原始路径。
        }
    };
    dbstln!(); //分开debug的函数/mod
    //           方法，          协议，版本，请求头。
    Request::new(line.remove(0),
                 pv.remove(0),
                 pv.remove(0),
                 header,
                 url_raw,
                 vp,
                 rp,
                 status)
}

fn url_to_vp_rp(rc_s: Rc<RcStream>, url: &str, mut vp_rp_full_match: &mut Option<(String, String)>) {
    // 不解码已经存在。
    for (vp, rp) in rc_s.arc().route().iter() {
        if url == *vp {
            // route 直接匹配了。也可以防止后面切片失败。
            *vp_rp_full_match = Some((vp.clone(), rp.clone()));
            break;
        }
        if url.starts_with(vp) {
            // Path::new("aa/bb/ccc").join("/")="/";
            // Path::new("aa/bb/ccc").join("/home")="/home";
            // vp如果是目录，则vp必然以/结尾，加上前面的url预处理，那么不可能切片以/开始还存在。
            // let mut idx = vp.len();
            // if url[vp.len()..].starts_with("/") {
            //     idx += 1;
            // }
            if url[vp.len()..].starts_with('/') {
                break;
            }
            let url_path = Path::new(rp).join(Path::new(&url[vp.len()..]));

            if url_path.exists() {
                *vp_rp_full_match = Some((url.to_string(), url_path.to_string_lossy().into_owned()));
                break;
            }
        }
    }
    if let Ok(decoded_url) = unquote(&url) {
        if vp_rp_full_match.is_none() && url != decoded_url {
            // 解码
            for (vp, rp) in rc_s.arc().route().iter() {
                if decoded_url == *vp {
                    // route 直接匹配了。也可以防止后面切片失败。
                    *vp_rp_full_match = Some((vp.clone(), rp.clone()));
                    break;
                }
                if decoded_url.starts_with(vp) {
                    if decoded_url[vp.len()..].starts_with('/') {
                        break;
                    }
                    dbstln!("{}@{}_url_sub_'/'_decoded: '{}' start_with(rp): \
                                {:?}\nrp.join(decoded_url): {:?}",
                            module_path!(),
                            rc_s.time().ls(),
                            &decoded_url,
                            vp,
                            Path::new(rp).join(Path::new(&decoded_url[vp.len()..])));
                    let url_path = Path::new(rp).join(Path::new(&decoded_url[vp.len()..]));
                    if url_path.exists() {
                        *vp_rp_full_match = Some((decoded_url, url_path.to_string_lossy().into_owned()));
                        break;
                    }
                }
            }
        }
    } else {
        dbstln!("{}_Warning@{}::url_to_vp_rp(): '{}' decode failed",
                NAME,
                module_path!(),
                url);
    }
}

/// 除去 `../` 和多余的 `/`,虽然对 `..//home/` 的处理不正确，但也够用了。
fn url_handle_pre(msg: &str) -> String {
    use std::ffi::OsStr;
    use std::path::Path;
    let mut cpts: Vec<&OsStr> = Vec::new();
    //.components()迭代出的组件会自动消去多余的/,除了/开始的保留首位的/。
    for c in Path::new(msg).components() {
        let c = c.as_os_str();
        // println!("{:?}", c);
        if c == OsStr::new("..") {
            cpts.pop();
        } else {
            cpts.push(c);
        }
    }
    let mut raw = String::new();
    // 迭代器处理不为/开始的添加两次/
    let cpts = if cpts[0] == OsStr::new("/") {
        let mut cp = cpts.into_iter();
        raw.push('/');
        cp.next();
        cp
    } else {
        cpts.into_iter()
    };
    raw = cpts.zip(vec![OsStr::new("/")].into_iter().cycle())
        .fold(raw,
              |acc, (x, y)| acc + x.to_str().unwrap() + y.to_str().unwrap());
    // 去除没有的/
    if !msg.ends_with('/') {
        raw.pop();
    }
    raw
}

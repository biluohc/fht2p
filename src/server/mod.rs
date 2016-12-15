extern crate urlparse;
use urlparse::quote_plus;
use urlparse::unquote_plus;

extern crate chrono;

use std::net::{TcpListener, TcpStream};
use std::result::Result;
use std::io;
use std::io::prelude::*;
use std::error::Error;
use std::env;
use std::thread;
use std::collections::HashMap;
use std::path::Path;
use std::fs::{self, metadata, File};
use std::time::Duration;

mod args; //命令行参数处理
mod resource; //资源性字符串/u8数组
mod path; // dir/file修改时间和大小
pub mod htm;  //html拼接

const BUFFER_SIZE: usize = 1024 * 1024 * 1; //字节1024*1024=>1m
const TIME_OUT: u64 = 6;// secs

pub fn fht2p<'a>() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let args = args::deal_args(&args[1..]);
    // println!("{:?}", args);
    match args {
        Ok(ok) => {
            match listener(&ok) {
                Ok(ok) => return Ok(ok),
                Err(e) => return Err(format!("{}:{} : {}", ok.ip, ok.port, e.description())),
            }
        }
        Err(e) => return Err(e),
    };
}
fn listener<'a>(args: &args::Args) -> Result<(), io::Error> {
    let addr = format!("{}:{}", args.ip, args.port);
    let listener = TcpListener::bind(&addr[..])?;
    println!("Fht2p/{} Serving at {} for {}",
             htm::VERSION,
             addr,
             args.dir);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = args.dir.to_string();
                let _ = thread::spawn(move || {
                    match deal_client(dir, stream) {
                        Ok(_) => {}
                        Err(e) => std_err("stream,", e.description()),
                    }
                });
            }
            Err(e) => {
                println!("{:?}", e);
                // connection failed
            }
        }
    }
    Ok(())
}


fn deal_client(dir: String, mut stream: TcpStream) -> Result<(), io::Error> {
    // // Length是下面的包括那两个换行符的html文本长度，若是小于则截断，超过则"连接被重置"。
    // // 请求一定要读取完，否则浏览器会报错？特别是UC。
    // // 注意换行符不能省略，否则也是"连接被重置"。

    let request_read = tcpstream_to_vec(&mut stream);
    let request_read_len: Result<usize, ()> = Ok(request_read.len());

    // client and server addr.
    let (server_addr, client_addr) = (format!("{}", stream.local_addr()?),
                                      format!("{}", stream.peer_addr()?));
    println!("\nclient_addr: {}\nserver_addr: {}",
             &client_addr,
             &server_addr);

    let time_out = Duration::new(TIME_OUT, 0);
    stream.set_read_timeout(Some(time_out))?;
    stream.set_write_timeout(Some(time_out))?;

    // println!("read_time_out: {:?}\nwrite_time_out: {:?}",
    //          stream.read_timeout(),
    //          stream.write_timeout());

    println!("read: {:?}", request_read_len);
    // 大小为0的请求？？？请求行都为空
    if request_read_len.unwrap() == 0 {
        return Ok(());
    }
    let request_read_ = request_read.clone();

    println!("RSQ0: {}",
             String::from_utf8(request_read_).unwrap().lines().nth(0).unwrap());

    let request = to_request(request_read, &dir);

    // print request line
    print_request(&request);

    // write response.
    to_response(server_addr, client_addr, dir, request, stream);
    Ok(())
}


fn tcpstream_to_vec(mut stream: &mut TcpStream) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![];
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let read_len = stream.read(&mut buffer).unwrap();
        if read_len < BUFFER_SIZE {
            vec.append(&mut buffer[..read_len].to_vec());
            break;
        }
        vec.append(&mut buffer.to_vec());
    }
    vec
}

fn print_request(request: &Request) {
    // println!("{:?}\n", request);
    println!("RSQ1: {} {} {}/{}",
             request.line.method,
             request.line.path,
             request.line.prtocol,
             request.line.version);
}
// 格式化标准错误输出
#[allow(dead_code)]
fn std_err(path_str: &str, msg: &str) {
    match writeln!(io::stderr(), "Warning: \"{}\" {}", path_str, msg) {    
        Ok(..) => {}
        Err(e) => panic!("{:?}", e),
    };
}

#[derive(Debug)]
struct Line {
    method: String,
    path: String,
    prtocol: String,
    version: f64,
}

#[derive(Debug)]
struct Request {
    line: Line,
    header: HashMap<String, String>,
    data: Vec<u8>,
}


fn to_request(vec8: Vec<u8>, dir: &str) -> Request {
    let msg;
    let data;
    match String::from_utf8(vec8) {
        Ok(ok) => {
            msg = ok;
            data = vec![]
        }
        Err(e) => {
            let valid_up_to = e.utf8_error().valid_up_to();
            let origin_bytes = e.into_bytes();
            match String::from_utf8(origin_bytes[..valid_up_to - 1].to_vec()) {
                Ok(ok) => {
                    msg = ok;
                    data = origin_bytes[valid_up_to..].to_vec();
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    // 可能未初始化，沃日！！ 默认"."应该是args所得目录
    let (mut method, mut path, mut protocol, mut version) =
        ("GET".to_string(), ".".to_string(), "HTTP".to_string(), 1.1);
    let mut header = HashMap::new();
    for (i, line) in msg.lines().enumerate() {
        if i == 0 {
            let vs: Vec<String> = line.split(' ').map(|x| x.to_string()).collect();
            method = (&vs[0]).to_string();
            {
                let url = unquote_plus(&vs[1]);
                let url_decode = match url {
                    Ok(ok) => ok,
                    Err(e) => {
                        std_err(&format!("url_decode_error: {}", e.description()), &vs[1]);
                        "/".to_string()
                    }
                };
                // 消去多余的 "/"
                let p: Vec<String> = url_decode.split("/")
                    .filter(|x| !x.is_empty())
                    .map(|x| x.to_string())
                    .collect();
                let sep = ["/".to_string()];
                let mut pr =
                    sep.iter().cycle().zip(p).fold(String::new(), |new, (x, y)| new + &x + &y);
                if &pr == "" {
                    pr = "/".to_string();
                }
                // println!("dir: {} pr: {}", dir, pr);
                let path_str = String::new() + dir + &pr;
                let path_str_origin = &pr;
                // 如果路径本身就存在，就不二次解码,(三次编码则会产生"/"等字符,不可能是真实路径。浏览器对URL只编码一次。
                match Path::new(&path_str).exists() {
                    false => path = String::new() + dir + &unquote_plus(path_str_origin).unwrap(),
                    true => path = path_str,
                };
            }
            let vss: Vec<String> = vs[2].split('/').map(|x| x.to_string()).collect();
            protocol = (&vss[0]).to_string();
            version = vss[1].parse::<f64>().unwrap_or(1.1);
        }
        match line.find(":") {
            Some(s) => header.insert(line[..s - 1].to_string(), line[s + 1..].to_string()),
            None => continue,
        };

    }
    Request {
        line: Line {
            method: method,
            path: path,
            prtocol: protocol,
            version: version,
        },
        header: header,
        data: data,
    }
}

#[derive(Debug)]
struct Status {
    protocol: String,
    version: f64,
    // code-name->hashmap
    code: u32,
    name: String,
}

#[derive(Debug)]
struct Response {
    status: Status,
    // extension_name-content_type->hashmap
    response_type: String,
    content_type: String,
    content_lenth: u64,
}


// 主要根据request.line.path处理文件/目录，制作响应。
// 以后也可以用URL参数提供排序功能（名字，修改时间，文件大小），排序交给js（和服务器无关）。
fn to_response(server_addr: String,
               client_addr: String,
               dir: String,
               request: Request,
               stream: TcpStream) {
    let code_name: HashMap<u32, &'static str> = resource::CNS.into_iter().map(|xy| *xy).collect();
    let extname_type: HashMap<&'static str, &'static str> = resource::ETS.into_iter()
        .map(|xy| *xy)
        .collect();
    let dir_len = dir.len();
    let (_, path_no_dir_str) = request.line.path.split_at(dir_len);
    let path = Path::new(&request.line.path);
    let (mut code, mut response_type);
    let (mut content_type, mut content_lenth);
    content_lenth = 0u64;  //possibly uninitialized 。。无力吐槽
    match path.is_dir() {
        true => {
            content_type = extname_type.get("html").unwrap().to_string();
            response_type = "dir";
        }
        false => {
            match path.extension() {
                    Some(s) => 
                        // 未包含后缀？处理到* 
                        content_type = match  extname_type.get(s.to_str().unwrap()) {
                            Some(ss)=>ss.to_string(),
                            None=>extname_type.get("*").unwrap().to_string(),
                        },
                    None => content_type = extname_type.get("*").unwrap().to_string(),
                };
            response_type = "file";
        }
    };
    match path.exists() {
        true => {
            match File::open(path) {
                Ok(_) => code = 200,
                Err(e) => {
                    std_err(e.description(), &request.line.path);
                    code = 500;
                    response_type = "500";
                }
            }
        }
        false => {
            if path_no_dir_str == resource::FAVICON_ICO_PATH ||
               path_no_dir_str == resource::CSS_PATH {
                code = 200;
                response_type = "static";
            } else {
                code = 404;
                response_type = "404";
            }
        }
    };

    let mut html = None;
    match response_type {
        "static" => {
            match path_no_dir_str {
                resource::FAVICON_ICO_PATH => content_lenth = resource::FAVICON_ICO.len() as u64,
                resource::CSS_PATH => content_lenth = resource::CSS.len() as u64,
                _ => std_err(&request.line.path, "match response_type failed !"),
            }
        }
        "dir" | "file" => {
            match res_lenth(dir, path_no_dir_str, &server_addr, &client_addr, &path) {
                Some((s, opt_html)) => {
                    content_lenth = s;
                    html = opt_html;
                }
                None => {
                    response_type = "500";
                    code = 500;
                    html = Some(htm::s500(&client_addr, &server_addr));
                    content_lenth = html.as_ref().unwrap().len() as u64;
                    content_type = extname_type.get("html").unwrap().to_string();
                }
            }
        }

        "404" => {
            html = Some(htm::s404(&client_addr, &server_addr));
            content_lenth = html.as_ref().unwrap().len() as u64;
            content_type = extname_type.get("html").unwrap().to_string();
        }
        "500" => {
            html = Some(htm::s500(&client_addr, &server_addr));
            content_lenth = html.as_ref().unwrap().len() as u64;
            content_type = extname_type.get("html").unwrap().to_string();
        }
        _ => panic!("match response_type failed !"),
    };
    let name = code_name.get(&code).unwrap().to_string();
    let response = Response {
        status: Status {
            protocol: "HTTP".to_string(),
            version: 1.1,
            code: code,
            name: name,
        },
        response_type: response_type.to_string(),
        content_type: content_type,
        content_lenth: content_lenth,
    };
    println!("Content-Type: {}", response.content_type);
    println!("Content-Length: {}", response.content_lenth);
    response_write(path_no_dir_str, html, path, &response, stream);
}

fn response_write(path_no_dir_str: &str,
                  html: Option<String>,
                  path: &Path,
                  response: &Response,
                  mut stream: TcpStream) {
    let header = format!("{}/{} {} {}\n",
                         response.status.protocol,
                         response.status.version,
                         response.status.code,
                         &response.status.name);
    let _ = stream.write(header.as_bytes());
    let content = format!("{}: {}\n{}: {}\n\n",
                          "Content-Type",
                          response.content_type,
                          "Content-Length",
                          response.content_lenth);
    let _ = stream.write(content.as_bytes());
    match response.response_type.as_str() {
        "static" => {
            println!("response_write_static {}", path_no_dir_str);
            match path_no_dir_str {
                resource::FAVICON_ICO_PATH => {
                    let _ = stream.write(resource::FAVICON_ICO);
                }
                resource::CSS_PATH => {
                    let _ = stream.write(resource::CSS.as_bytes());
                }
                _ => {}
            }
        }
        "dir" => {
            println!("response_write_dir: {:?}", path);
            let _ = stream.write(html.unwrap().as_bytes());
        }
        "file" => {
            println!("response_write_file: {:?}", path);
            file_to_bytes(&path, stream);
        }

        "404" => {
            println!("response_write_404: {:?}", path);
            let _ = stream.write(html.unwrap().as_bytes());
        }
        "500" => {
            println!("response_write_500: {:?}", path);
            let _ = stream.write(html.unwrap().as_bytes());
        }
        _ => {
            std_err(path.to_string_lossy().into_owned().as_str(),
                    "match response_type failed !");
        }
    };

}



fn file_to_bytes(path: &Path, mut stream: TcpStream) {
    let path_str = path.to_string_lossy().into_owned();
    let mut file = File::open(path).unwrap();
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut stream_len: io::Result<usize> = Ok(0);
    loop {
        let read_size = file.read(&mut buffer).unwrap();
        if read_size < BUFFER_SIZE {
            let s = stream.write(&buffer[..read_size].to_vec());
            match s {
                Ok(ok) => stream_len = Ok(stream_len.unwrap() + ok),
                // Err(e) => panic!("{:?}", e),
                Err(e) => {
                    std_err(&path_str, e.description());
                    return;
                }
            }
            break;
        };
        let s = stream.write(&buffer);
        match s {
            Ok(ok) => stream_len = Ok(stream_len.unwrap() + ok),
            // Err(e) => panic!("{:?}", e),
            Err(e) => {
                std_err(&path_str, e.description());
                return;
            }
        };
    }
    let rs = stream.flush();
    println!("write_file: {:?} Result:{:?}", stream_len, rs);
}

fn res_lenth(dir: String,
             path_no_dir_str: &str,
             server_addr: &String,
             client_addr: &String,
             path: &Path)
             -> Option<(u64, Option<String>)> {
    match path.is_dir() {
        true => dir_lenth(dir, path_no_dir_str, server_addr, client_addr, path),
        false => file_lenth(path),
    }
}

fn file_lenth(path: &Path) -> Option<(u64, Option<String>)> {
    match metadata(path) {
        Ok(ok) => Some((ok.len(), None)),
        Err(e) => {
            // panic!("file_lenth: {:?}\n{:?}", path, e);
            std_err(&path.to_string_lossy().into_owned(), e.description());
            None
        }
    }
}

fn dir_lenth(dir: String,
             path_no_dir_str: &str,
             server_addr: &String,
             client_addr: &String,
             path: &Path)
             -> Option<(u64, Option<String>)> {
    let path_str = path.to_string_lossy().into_owned();
    // println!("dir: {}\tno_dir: {}\tpath: {}", dir, path_no_dir_str, path_str);
    let path_no_dir_str_end = path_no_dir_str.ends_with("/");

    let title = path_no_dir_str.to_string();
    let h1 = title.clone();

    let mut ul = htm::Ul::new();

    if path_no_dir_str != "/" {
        let path_parent =
            Path::new(path_no_dir_str).parent().unwrap().to_string_lossy().into_owned();
        let path_parent = match quote_plus(path_parent.clone(), b"") {
            Ok(ok) => ok,
            Err(e) => {
                std_err(&format!("url_encode_error: {}", e.description()),
                        &path_parent);
                path_parent
            }
        };
        // println!("parent: {}", &path_parent);
        let (date, size) = path::fms(&(String::new() + &dir + &path_no_dir_str));
        ul.push(htm::Li::new(path_parent, "../ Parent Directory".to_string(), date, size));
    }
    match fs::read_dir(path) {
        Ok(ok) => {
            for entry in ok {
                let entry =
                    entry.expect("unwrap Result<std::fs::DirEntry, std::io::Error> on dir_lenth()")
                        .path();
                // let entry_str = entry.to_string_lossy().into_owned();
                let entry_name = entry.file_name().unwrap().to_string_lossy().into_owned();
                let (date, size) = path::fms(&(String::new() + &path_str + "/" + &entry_name));

                // 以防 /home/viw/Downloads/cache/muut/srcmain.rs，找不到文件。
                let path_http = match path_no_dir_str_end {
                    true => String::new() + path_no_dir_str + &entry_name,
                    false => String::new() + path_no_dir_str + "/" + &entry_name,
                };
                let path_http = match quote_plus(path_http.clone(), b"") {
                    Ok(ok) => ok,
                    Err(e) => {
                        std_err(&format!("url_encode_error: {}", e.description()),
                                &path_http);
                        path_http
                    }

                };
                // println!("no_dir:{}\tentry_name: {}\tpath_http: {}",
                //          path_no_dir_str,
                //          &entry_name,
                //          path_http);
                match entry.is_dir() {
                    true => 
                            // "/" 区分目录与文件(视觉)
                              ul.push(htm::Li::new(path_http,entry_name+"/",date,size)),

                    false =>  ul.push(htm::Li::new(path_http,entry_name,date,size)),
                };
            }
            let addr = htm::Address::new(server_addr);
            let html = htm::Html::new(title,
                                      htm::H1::new(h1, client_addr.to_string()),
                                      Some(ul),
                                      addr);
            let html = format!("{}", html);
            let lenth = html.len();
            // println!("{}\nhtml_len: {}", html, lenth);
            Some((lenth as u64, Some(html)))
        }
        Err(e) => {
            std_err(&path.to_string_lossy().into_owned(), e.description());
            None
        }
    }
}

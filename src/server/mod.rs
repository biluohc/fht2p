use std::net::TcpListener;
use std::error::Error;
use std::thread;
use std::env;
use std::io;

use super::*;

mod args; //命令行参数处理
mod resource; //资源性字符串/u8数组
mod path; // dir/file修改时间和大小
mod htm;  //html拼接
mod reqs;

const BUFFER_SIZE: usize = 1024 * 1024 * 1; //字节1024*1024=>1m
const TIME_OUT: u64 = 5;// secs
// set_nonblocking 不能使用,因为读取文件会阻塞，只能set_write/read_timeout() 来断开一直阻塞的连接。

pub fn fht2p() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let args = args::deal_args(&args[1..]);
    let os_type = sys_info::os_type().unwrap_or("Unkown".to_string());
    let os_release = sys_info::os_release().unwrap_or("Unkown".to_string());
    unsafe {
        if os_type == os_release && os_release == "Unkown" {
            // default is "unkown".
        } else {
            let str_box = Box::new(format!("{}/{}", os_type, os_release));
            let str_ptr = Box::into_raw(str_box);
            htm::PLATFORM = *(str_ptr as *const &'static str);
        }

    }
    // println!("{:?}", args);
    match args {
        Ok(ok) => {
            match listener(&ok) {
                Ok(ok) => Ok(ok),
                Err(e) => Err(format!("{}:{} : {}", ok.ip, ok.port, e.description())),
            }
        }
        Err(e) => Err(e),
    }
}
fn listener(args: &args::Args) -> Result<(), io::Error> {
    let addr = format!("{}:{}", args.ip, args.port);
    let listener = TcpListener::bind(&addr[..])?;
    println!("Fht2p/{} Serving at {} for {}",
             htm::VERSION,
             addr,
             args.dir);
    println!("You can visit http://127.0.0.1:{}", args.port);

    let dir = args.dir.to_string();
    thread::Builder::new().spawn(move || {
            reqs::for_listener(listener, dir);
        })?;
    Ok(())
}

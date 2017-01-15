use std::collections::HashMap;
use std::net::TcpListener;
use std::error::Error;
use std::thread;
use std::io;

use super::*;
use chrono::{Local, DateTime};

mod args; //命令行参数处理
use self::args::Config;
mod consts; //资源性字符串/u8数组
use self::consts::*; //const 变量
mod path; // dir/file修改时间和大小
mod htm;  //html拼接
mod methods;

pub fn fht2p() -> Result<(), String> {
    let config = args::get_config()?;
    let config_ = config.clone();
    let arc_config = ArcConfig::get(config_.route, config_.keep_alive, false);
    let arc_config = Arc::from(arc_config);
    // unsafe {
    //     let str_box = Box::new(format!("{}/{}", env::consts::OS, env::consts::ARCH));
    //     let str_ptr = Box::into_raw(str_box);
    //     OS = *(str_ptr as *const &'static str);
    // }
    let (mut count, port_num) = (0, config.port.len());
    for port in &config.port {
        match listener(&config, port, arc_config.clone()) {
            Ok(ok) => return Ok(ok),
            Err(e) => {
                if count == port_num - 1 {
                    return Err(format!("{}:{:?} : {}", config.ip, config.port, e.description()));
                }
            }
        };
        count += 1;
    } //不可能错误吧，2333
    Ok(())
}
fn listener(config: &Config, port: &u32, arc_config: Arc<ArcConfig>) -> Result<(), io::Error> {
    let addr = format!("{}:{}", config.ip, port);
    let tcp_listener = TcpListener::bind(&addr[..])?;
    println!("{}/{} Serving at {} for {:?}",
             NAME,
             VERSION,
             addr,
             config.route);
    println!("You can visit http://127.0.0.1:{}", port);

    thread::Builder::new().spawn(move || {
            methods::for_listener(tcp_listener, arc_config);
        })?;
    Ok(())
}

#[derive(Debug)]
pub struct ArcConfig {
    pub route: HashMap<String, String>, // virtual path=> real path
    pub cns: HashMap<u32, &'static str>, // code number =>status strs
    pub ents: HashMap<&'static str, &'static str>, // expand's name =>Content-Types
    pub sfs: HashMap<&'static str, &'static [u8]>, // static's files
    pub uptime: DateTime<Local>, // Local::now(); static's files modified's time,etc
    pub keep_alive: bool,
    pub debug: bool,
}
impl ArcConfig {
    fn new(route: HashMap<String, String>,
           cns: HashMap<u32, &'static str>,
           ents: HashMap<&'static str, &'static str>,
           sfs: HashMap<&'static str, &'static [u8]>,
           keep_alive: bool,
           debug: bool)
           -> ArcConfig {
        ArcConfig {
            route: route,
            cns: cns,
            ents: ents,
            sfs: sfs,
            uptime: Local::now(),
            keep_alive: keep_alive,
            debug: debug,
        }
    }
    fn get(route: HashMap<String, String>, keep_alive: bool, debug: bool) -> ArcConfig {
        let cns: HashMap<u32, &'static str> = CNS.into_iter().map(|xy| *xy).collect();
        let ents: HashMap<&'static str, &'static str> = ETS.into_iter()
            .map(|xy| *xy)
            .collect();
        // 暂时保留CSS让编译可以过。
        let mut sfs: HashMap<&'static str, &'static [u8]> = HashMap::new();
        sfs.insert(CSS_PATH, &CSS8[..]);
        sfs.insert(FAVICON_ICO_PATH, &FAVICON_ICO[..]);
        ArcConfig::new(route, cns, ents, sfs, keep_alive, debug)
    }
}

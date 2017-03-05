use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::io;

use super::*;
use poolite::{Pool, IntoIOResult};

mod args; //命令行参数处理
use self::args::Config;
mod consts; //资源性字符串/u8数组
use self::consts::*; //const 变量
// mod path; // dir/file修改时间和大小
// mod htm;  //html拼接
mod methods;
mod date;
use self::date::Date;

pub fn fht2p() -> Result<(), String> {
    let config = args::get_config()?;
    dbln!("{:?}", config); //debug for getting's Config
    let config_ = config.clone();
    let arc_config = ArcConfig::get(config_.route, config_.keep_alive);
    let arc_config = Arc::from(arc_config);
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
    }
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

    let pool = Pool::new().load_limit(Pool::num_cpus() * Pool::num_cpus())
        .run()
        .into_iorst()?;
    thread::Builder::new().spawn(move || { methods::for_listener(tcp_listener, arc_config, pool); })?;
    Ok(())
}

#[derive(Debug)]
pub struct ArcConfig {
    pub route: HashMap<String, String>, // virtual path=> real path
    pub route_rpset: HashSet<String>,
    pub cns: HashMap<u32, &'static str>, // code number =>status strs
    // expand's name =>Content-Types
    pub ents_bin: HashMap<&'static str, &'static str>,
    pub ents_doc: HashMap<&'static str, &'static str>,
    pub sfs: HashMap<&'static str, &'static [u8]>, // static's files
    pub uptime: Date, // Local::now(); static's files modified's time,etc
    pub keep_alive: bool,
}
impl ArcConfig {
    fn new(route: HashMap<String, String>,
           cns: HashMap<u32, &'static str>,
           ents_bin: HashMap<&'static str, &'static str>,
           ents_doc: HashMap<&'static str, &'static str>,
           sfs: HashMap<&'static str, &'static [u8]>,
           keep_alive: bool)
           -> ArcConfig {
        let mut route_rpset: HashSet<String> = HashSet::new();
        for rp in route.values() {
            route_rpset.insert(rp.clone());
        }
        ArcConfig {
            route: route,
            route_rpset: route_rpset,
            cns: cns,
            ents_bin: ents_bin,
            ents_doc: ents_doc,
            sfs: sfs,
            uptime: Date::now(),
            keep_alive: keep_alive,
        }
    }
    fn get(mut route: HashMap<String, String>, keep_alive: bool) -> ArcConfig {
        let mut cns: HashMap<u32, &'static str> = CNS.into_iter().cloned().collect();
        // let mut cns: HashMap<u32, &'static str> = CNS.into_iter().map(|xy| *xy).collect();
        let mut ents_bin: HashMap<&'static str, &'static str> = ETS_BIN.into_iter().cloned().collect();
        let mut ents_doc: HashMap<&'static str, &'static str> = ETS_DOC.into_iter().cloned().collect();
        // 暂时保留CSS让编译可以过。
        let mut sfs: HashMap<&'static str, &'static [u8]> = HashMap::with_capacity(3);
        sfs.insert(FAVICON_ICO_PATH, &FAVICON_ICO[..]);
        sfs.insert(CSS_PATH, CSS.as_bytes());
        sfs.insert(JS_PATH, JS.as_bytes());
        // shrink_to_fit前后大小一样？fuck
        route.shrink_to_fit();
        cns.shrink_to_fit();
        ents_bin.shrink_to_fit();
        ents_doc.shrink_to_fit();
        sfs.shrink_to_fit();
        ArcConfig::new(route, cns, ents_bin, ents_doc, sfs, keep_alive)
    }
    #[inline]
    fn route(&self) -> &HashMap<String, String> {
        &self.route
    }
    #[inline]
    fn route_rpset(&self) -> &HashSet<String> {
        &self.route_rpset
    }
    #[inline]
    fn cns(&self) -> &HashMap<u32, &'static str> {
        &self.cns
    }
    #[inline]
    fn ents_bin(&self, exname: &str) -> String {
        self.ents_bin
            .get(exname)
            .unwrap_or(&self.ents_bin["*"])
            .to_owned()
            .to_owned()
    }
    #[inline]
    fn ents_doc(&self, exname: &str) -> String {
        let charset = "; charset=utf-8";
        self.ents_doc
            .get(exname)
            .unwrap_or(&self.ents_doc["*"])
            .to_owned()
            .to_owned() + charset
    }
    #[inline]
    fn sfs(&self) -> &HashMap<&'static str, &'static [u8]> {
        &self.sfs
    }
    #[inline]
    fn uptime(&self) -> &Date {
        &self.uptime
    }
    #[inline]
    fn keep_alive(&self) -> bool {
        self.keep_alive
    }
}

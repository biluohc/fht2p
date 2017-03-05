use std::collections::HashMap;
use std::process::exit;
use std::error::Error;
use std::path::Path;
use std::env;
use std;

use tini::Ini; // ini文件

use super::consts::*; // 名字,版本,作者，简介，地址
// use app;  // 命令行参数处理。
use app::{App, Opt, Flag};

pub fn get_config() -> Result<Config, String> {
    let app = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR, EMAIL)
        .address(URL_NAME, URL)
        .about(ABOUT)
        .flag(Flag::new("cp").short("cp").long("cp").help("Print the default config file"))
        .flag(Flag::new("keep_alive").short("ka").long("keep-alive").help("use keep-alive"))
        .opt(Opt::new("log").long("log").short("log").help("Print log for debug"))
        .opt(Opt::new("ip").short("i").long("ip").help("Sets listenning ip"))
        .opt(Opt::new("port").short("p").long("port").help("Sets listenning port"))
        .opt(Opt::new("config").short("cf").long("config").help("Sets a custom config file"))
        .args_name("PATHS")
        .args_default("./")
        .args_help("Sets the paths to share")
        .get();
    //-cf ,--config选项，如果有就载入该文件。
    if let Some(s) = app.get_opt("config") {
        return Config::load_from_file(&s);
    }
    // 命令行有没有参数？有就解析参数，没有就寻找配置文件，再没有就使用默认配置。
    if env::args().len() <= 1 {
        match get_config_path() {
            Some(s) => Config::load_from_file(&s),
            None => Ok(Config::default()),
        }
    } else {
        let app_ini = app.to_ini();
        Config::load_from_ini("env::args()", app_ini)
    }
}

#[derive(Debug,Clone)]
pub struct Config {
    pub ip: String,
    pub port: Vec<u32>,
    pub keep_alive: bool,
    pub route: HashMap<String, String>,
}

impl Config {
    fn new(ip: String, port: Vec<u32>, keep_alive: bool, route: HashMap<String, String>) -> Self {
        Config {
            ip: ip,
            port: port,
            keep_alive: keep_alive,
            route: route,
        }
    }
    fn load_from_file(file_name: &str) -> Result<Config, String> {
        let path = Path::new(file_name);
        let conf = Ini::from_file(path);
        if let Err(e) = conf {
            return Err(format!("config file '{}' is invalid: {}",
                               file_name,
                               e.description()));
        }
        Self::load_from_ini(file_name, conf.unwrap())
    }
    fn load_from_ini(file_name: &str, conf: Ini) -> Result<Config, String> {
        let ip: Option<String> = conf.get("Setting", "ip");
        if ip.is_none() {
            return Err(format!("'{}''s Setting's ip is a invalid value", file_name));
        }
        let port: Option<Vec<u32>> = conf.get_vec("Setting", "port");
        if ip.is_none() {
            return Err(format!("'{}''s Setting's port is a invalid value", file_name));
        }
        let keep_alive: Option<bool> = conf.get("Setting", "keep-alive");
        if keep_alive.is_none() {
            return Err(format!("'{}''s Setting's keep-alive is a invalid value", file_name));
        }
        let root: Option<String> = conf.get("Route", "/");
        if root.is_none() {
            return Err(format!("'{}''s Route's '/' is a invalid value", file_name));
        }
        let mut route: HashMap<String, String> = HashMap::new();
        for (key, val) in conf.iter_section("Route").unwrap() {
            if !Path::new(&val).exists() {
                return Err(format!("'{}''s route's {}'s {} is not exists", file_name, key, val));
            }
            route.insert(key.clone(), val.clone());
        }
        Ok(Self::new(ip.unwrap(), port.unwrap(), keep_alive.unwrap(), route))
    }
    fn load_from_str(str_name: &str, str: &str) -> Result<Config, String> {
        let conf = Ini::from_buffer(str);
        Self::load_from_ini(str_name, conf)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config::load_from_str("CONFIG-Default", CONFIG_DEFAULT).unwrap()
    }
}

// 打印默认配置文件。
fn cp() {
    println!("{}", CONFIG_DEFAULT);
    std::process::exit(0);
}

fn get_config_path() -> Option<String> {
    match std::env::home_dir() {
        // 家目录 ～/.config/fht2p/fht2p.ini
        Some(ref home) if home.as_path()
                              .join(".config/fht2p")
                              .join(CONFIG_DEFAULT_PATH)
                              .exists() => {
            Some(home.as_path()
                     .join(".config/fht2p")
                     .join(CONFIG_DEFAULT_PATH)
                     .to_string_lossy()
                     .into_owned())
        }
        // 可执行文件所在目录 path/fht2p.ini
        _ if std::env::current_exe().is_ok() &&
             std::env::current_exe()
                 .unwrap()
                 .parent()
                 .unwrap()
                 .join(CONFIG_DEFAULT_PATH)
                 .exists() => {
            Some(std::env::current_exe()
                     .unwrap()
                     .parent()
                     .unwrap()
                     .join(CONFIG_DEFAULT_PATH)
                     .to_string_lossy()
                     .into_owned())
        }
        // 当前目录 dir/fht2p.ini
        _ if std::env::current_dir().is_ok() && std::env::current_dir().unwrap().join(CONFIG_DEFAULT_PATH).exists() => {
            Some(std::env::current_dir()
                     .unwrap()
                     .join(CONFIG_DEFAULT_PATH)
                     .to_string_lossy()
                     .into_owned())
        }
        _ => None,
    }
}

trait AppToIni {
    fn to_ini(self) -> Ini;
}
impl<'app> AppToIni for App<'app> {
    fn to_ini(self) -> Ini {
        // 打印默认配置。
        if self.get_flag("cp").is_some() {
            cp();
        }
        let config_default = Config::default();
        // Setting.
        let mut conf = Ini::new().section("Setting");
        let ip = self.get_opt("ip").unwrap_or(config_default.ip);
        // Vec<u32>转为 8080 ,8000 ,1080 ,etc.
        let vec_to_str = |vec: Vec<u32>| {
            let mut str = String::new();
            for v in &vec {
                if str.is_empty() {
                    str += &format!("{}", v);
                } else {
                    str += &format!(", {}", v);
                }
            }
            str
        };
        let port = self.get_opt("port").unwrap_or(vec_to_str(config_default.port));
        let keep_alive = self.get_opt("keep_alive").unwrap_or(format!("{}", config_default.keep_alive));
        conf = conf.item("ip", &ip);
        conf = conf.item("port", &port);
        conf = conf.item("keep-alive", &keep_alive);
        // Route
        conf = conf.section("Route");
        let route = args_paths_to_route(self.get_args().unwrap());
        for (k, v) in &route {
            conf = conf.item(k.to_string(), v.to_string());
        }
        conf
    }
}

// 参数转换为Route
fn args_paths_to_route(map: Vec<String>) -> HashMap<String, String> {
    let mut route: HashMap<String, String> = HashMap::new();
    let root = map[0].to_string(); //根目录。
    route.insert("/".to_owned(), root);
    if map.len() > 1 {
        for v in &map[1..] {
            route.insert(route_name(v), v.to_string());
        }
    }
    return route;
    fn route_name(msg: &str) -> String {
        let path = Path::new(msg);
        match path.file_name() {
            Some(ok) => ok.to_string_lossy().into_owned(),
            None => {
                errln!("Path '{}' dost not have name", msg);
                exit(1);
            }
        }
    }
}

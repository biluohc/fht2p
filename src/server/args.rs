use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::env;
use std;

use tini::Ini; // ini文件

use super::consts::*; // 名字,版本,作者，简介，地址

use app::{App, Opt};

pub fn parse() -> Config {
    let mut config = Config::default();
    let mut server = config.servers.pop().unwrap();
    let mut routes: Vec<String> = Vec::new();
    let mut cp = false;
    let mut c_path: Option<String> = None;
    let mut log = String::new();

    let helper = {
        App::new(NAME)
            .version(VERSION)
            .author(AUTHOR, EMAIL)
            .addr(URL_NAME, URL)
            .desc(ABOUT)
            .opt(Opt::new("cp", &mut cp)
                     .short("cp")
                     .long("cp")
                     .help("Print the default config file"))
            .opt(Opt::new("config", &mut c_path)
                     .optional()
                     .short("c")
                     .long("config")
                     .help("Sets a custom config file"))
            .opt(Opt::new("log", &mut log)
                     .optional()
                     .long("log")
                     .short("log")
                     .help("Print log for debug"))
            .opt(Opt::new("keep_alive", &mut config.keep_alive)
                     .short("k")
                     .long("keep-alive")
                     .help("use keep-alive"))
            .opt(Opt::new("ip", &mut server.ip)
                     .short("i")
                     .long("ip")
                     .help("Sets listenning ip"))
            .opt(Opt::new("port", &mut server.port)
                     .short("p")
                     .long("port")
                     .help("Sets listenning port"))
            .args("PATHS", &mut routes)
            .args_optional()
            .args_help(r#"Sets the paths to share(default is "./")"#)
            .parse_args()
    };
    // -cp/--cp
    if cp {
        config_print();
    }
    //-c/--config选项，如果有就载入该文件。
    if let Some(s) = c_path {
        return Config::load_from_file(&s)
                   .map_err(|e| helper.help_err_exit(e, 1))
                   .unwrap();
    }
    // 命令行有没有参数？有就解析参数，没有就寻找配置文件，再没有就使用默认配置。
    if env::args().skip(1).len() == 0 {
        match get_config_path() {
            Some(s) => {
                Config::load_from_file(&s)
                    .map_err(|e| helper.help_err_exit(e, 1))
                    .unwrap()
            }
            None => Config::load_from_CONFIG(),
        }
    } else {
        config.servers.push(server);
        if !routes.is_empty() {
            config.routes = args_paths_to_route(&routes[..])
                .map_err(|e| helper.help_err_exit(e, 1))
                .unwrap();
        }
        config
    }
}

use std::net::{IpAddr, Ipv4Addr};
#[derive(Debug,Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}
impl Default for Server {
    fn default() -> Server {
        Server {
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 8080,
        }
    }
}
impl Server {
    fn new(ip: IpAddr, port: u16) -> Self {
        Server { ip: ip, port: port }
    }
}
#[derive(Debug,Clone)]
pub struct Config {
    pub keep_alive: bool,
    pub servers: Vec<Server>,
    pub routes: HashMap<String, String>,
}
impl Default for Config {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert("/".to_owned(), "./".to_owned());
        Config {
            keep_alive: false,
            servers: vec![Server::default()],
            routes: map,
        }
    }
}

impl Config {
    fn load_from_file(path: &str) -> Result<Self, String> {
        let conf = Ini::from_file(path)
            .map_err(|e| format!("config file('{}') is invalid: {}", path, e.description()))?;
        Self::load_from_ini(path, conf)
    }
    fn load_from_ini(file_name: &str, conf: Ini) -> Result<Config, String> {
        let mut config = Config::default();
        config.routes.clear();
        config.servers.clear();

        config.keep_alive = conf.get("Setting", "keep-alive")
            .ok_or_else(|| format!("'{}''s Setting's keep-alive is a invalid value", file_name))?;

        for (key, val) in conf.iter_section("Servers")
                .ok_or_else(|| format!("'{}''s Servers is a invalid value", file_name))? {
            let k = key.parse::<IpAddr>()
                .map_err(|_| {
                             format!("'{}''s `{} = {}`'s ip is a invalid value",
                                     key,
                                     val,
                                     file_name)
                         })?;
            let v = val.parse::<u16>()
                .map_err(|_| {
                             format!("'{}''s `{} = {}`'s port is a invalid value",
                                     key,
                                     val,
                                     file_name)
                         })?;
            config.servers.push(Server::new(k, v));
        }

        let _: String = conf.get("Routes", "/")
            .ok_or_else(|| format!("'{}''s 'Routes = /`'s port is a invalid value", file_name))?;

        for (key, val) in conf.iter_section("Routes")
                .ok_or_else(|| format!("'{}''s Routes is a invalid value", file_name))? {
            if !Path::new(&val).exists() {
                errln!("Warning: '{}''s routes's `{}`'s `{}` is not exists",
                       file_name,
                       key,
                       val);
            }
            if config.routes.insert(key.clone(), val.clone()).is_some() {
                return Err(format!("'{}''s routes's {} already defined", file_name, key));
            }
        }
        Ok(config)
    }
    fn load_from_str(str_name: &str, str: &str) -> Result<Config, String> {
        let conf = Ini::from_buffer(str);
        Self::load_from_ini(str_name, conf)
    }
    #[allow(non_snake_case)]
    fn load_from_CONFIG() -> Self {
        Config::load_from_str("CONFIG-Default", CONFIG_DEFAULT).unwrap()
    }
}

// 打印默认配置文件。
fn config_print() {
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
        _ if std::env::current_dir().is_ok() &&
             std::env::current_dir()
                 .unwrap()
                 .join(CONFIG_DEFAULT_PATH)
                 .exists() => {
            Some(std::env::current_dir()
                     .unwrap()
                     .join(CONFIG_DEFAULT_PATH)
                     .to_string_lossy()
                     .into_owned())
        }
        _ => None,
    }
}

// 参数转换为Route
fn args_paths_to_route(map: &[String]) -> Result<HashMap<String, String>, String> {
    let mut routes: HashMap<String, String> = HashMap::new();
    for (idx, rel) in map.iter().enumerate() {
        if !Path::new(&rel).exists() {
            errln!("Warning: {:?} is not exists", &rel);
        }
        if idx == 0 {
            routes.insert("/".to_owned(), rel.to_string());
        } else if routes
                      .insert(route_name(rel)?, rel.to_string())
                      .is_some() {
            return Err(format!("{} already defined", route_name(rel).unwrap()));
        }
    }
    fn route_name(msg: &str) -> Result<String, String> {
        let path = Path::new(msg);
        path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .ok_or_else(|| format!("Path '{}' dost not have name", msg))
    }
    Ok(routes)
}

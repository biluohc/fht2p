use app::{App, Args, Opt};
use toml;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::collections::HashMap as Map;
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::env;
use std;

use super::consts::*; // 名字,版本,作者，简介，地址

/// Get `Config` by `parse` `args`
pub fn parse() -> Config {
    let mut config = Config::default();
    let mut server = Server::default();
    let mut routes: Vec<String> = vec!["./".to_owned()];
    let mut cp = false;
    let mut c_path: Option<String> = None;

    let helper = {
        App::new(NAME)
            .version(VERSION)
            .author(AUTHOR, EMAIL)
            .addr(URL_NAME, URL)
            .desc(DESC)
            .opt(
                Opt::new("cp", &mut cp)
                    .short('C')
                    .long("cp")
                    .help("Print the default config file"),
            )
            .opt(
                Opt::new("config", &mut c_path)
                    .optional()
                    .short('c')
                    .long("config")
                    .help("Sets a custom config file"),
            )
            .opt(
                Opt::new("root", &mut config.redirect_html)
                    .short('r')
                    .long("rh")
                    .help("Redirect dir to 'index.htm[l]`, if it exists"),
            )
            .opt(
                Opt::new("secs", &mut config.keep_alive)
                    .short('k')
                    .long("ka")
                    .help("Time HTTP keep alive")
                    .optional(),
            )
            .opt(
                Opt::new("ip", &mut server.ip)
                    .short('i')
                    .long("ip")
                    .help("Sets listenning ip"),
            )
            .opt(
                Opt::new("port", &mut server.port)
                    .short('p')
                    .long("port")
                    .help("Sets listenning port"),
            )
            .args(Args::new("PATH", &mut routes).help(r#"Sets the paths to share"#))
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
            Some(s) => Config::load_from_file(&s)
                .map_err(|e| helper.help_err_exit(e, 1))
                .unwrap(),
            None => Config::load_from_STR(),
        }
    } else {
        config.addrs.clear();
        config.addrs.push(SocketAddr::new(server.ip, server.port));
        config.routes = args_paths_to_route(&routes[..])
            .map_err(|e| helper.help_err_exit(e, 1))
            .unwrap();
        config
    }
}

#[derive(Debug, Clone)]
struct Server {
    pub ip: IpAddr,
    pub port: u16,
}
impl Default for Server {
    fn default() -> Server {
        Self::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080)
    }
}
impl Server {
    fn new(ip: IpAddr, port: u16) -> Self {
        Server { ip: ip, port: port }
    }
}
// 关键是结构体的字段名，和toml的[name]对应
#[derive(Debug, Deserialize)]
struct Fht2p {
    setting: Setting,
    routes: Vec<Route>,
}

#[derive(Debug, Deserialize)]
struct Setting {
    #[serde(rename = "redirect-html")] redirect_html: bool,
    #[serde(rename = "keep-alive")] keep_alive: Option<u64>,
    addrs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Route {
    url: String,
    path: String,
}

/// `Config` for `main`
#[derive(Debug, Clone)]
pub struct Config {
    pub redirect_html: bool,
    pub keep_alive: Option<u64>,
    pub addrs: Vec<SocketAddr>,
    pub routes: Map<String, String>,
}
impl Default for Config {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert("/".to_owned(), "./".to_owned());
        Config {
            redirect_html: false,
            keep_alive: None,
            addrs: vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            ],
            routes: map,
        }
    }
}

impl Config {
    fn load_from_file(path: &str) -> Result<Self, String> {
        let mut str = String::new();
        let mut file = File::open(path).map_err(|e| format!("config file('{}') open fails: {}", path, e.description()))?;
        file.read_to_string(&mut str)
            .map_err(|e| format!("config file('{}') read fails: {}", path, e.description()))?;
        Self::load_from_str(path, &str)
    }
    fn load_from_str(file_name: &str, toml: &str) -> Result<Config, String> {
        let mut config = Self::default();
        config.routes.clear();
        config.addrs.clear();

        let toml: Fht2p = toml::from_str(toml).map_err(|e| format!("config file('{}') parse fails: {}", file_name, e))?;
        config.redirect_html = toml.setting.redirect_html;
        config.keep_alive = toml.setting.keep_alive;
        for server in toml.setting.addrs {
            let addr = server.parse::<SocketAddr>().map_err(|e| {
                format!(
                    "config file('{}')'s {} parse::<SocketAddr> fails: {}",
                    file_name,
                    server,
                    e.description()
                )
            })?;
            config.addrs.push(addr);
        }

        for route in &toml.routes {
            if !Path::new(&route.path).exists() {
                warn!(
                    "'{}''s routes({:?}: {:?}) is not exists",
                    file_name, route.url, route.path
                );
            }
            if config
                .routes
                .insert(route.url.clone(), route.path.clone())
                .is_some()
            {
                return Err(format!(
                    "'{}''s routes's {} already defined",
                    file_name, route.path
                ));
            }
        }
        if config.addrs.is_empty() {
            return Err(format!("'{}''s addrs is empty", file_name));
        }
        if config.routes.is_empty() {
            return Err(format!("'{}''s routes is empty", file_name));
        }
        Ok(config)
    }
    #[allow(non_snake_case)]
    fn load_from_STR() -> Self {
        Config::load_from_str("CONFIG-STR", CONFIG_STR).unwrap()
    }
}

// 打印默认配置文件。
fn config_print() {
    println!("{}", CONFIG_STR);
    std::process::exit(0);
}

fn get_config_path() -> Option<String> {
    match std::env::home_dir() {
        // 家目录 ～/.config/fht2p/fht2p.toml
        Some(ref home)
            if home.as_path()
                .join(".config/fht2p")
                .join(CONFIG_STR_PATH)
                .exists() =>
        {
            Some(
                home.as_path()
                    .join(".config/fht2p")
                    .join(CONFIG_STR_PATH)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        // 可执行文件所在目录 path/fht2p.toml
        _ if std::env::current_exe().is_ok()
            && std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join(CONFIG_STR_PATH)
                .exists() =>
        {
            Some(
                std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(CONFIG_STR_PATH)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        // 当前目录 dir/fht2p.toml
        _ if std::env::current_dir().is_ok()
            && std::env::current_dir()
                .unwrap()
                .join(CONFIG_STR_PATH)
                .exists() =>
        {
            Some(
                std::env::current_dir()
                    .unwrap()
                    .join(CONFIG_STR_PATH)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        _ => None,
    }
}

// 参数转换为Route url, path
fn args_paths_to_route(map: &[String]) -> Result<Map<String, String>, String> {
    let mut routes: Map<String, String> = Map::new();
    for (idx, path) in map.iter().enumerate() {
        if !Path::new(&path).exists() {
            warn!("{:?} is not exists", &path);
        }
        if idx == 0 {
            routes.insert("/".to_owned(), path.to_string());
        } else if routes.insert(route_name(path)?, path.to_string()).is_some() {
            return Err(format!("{} already defined", route_name(path).unwrap()));
        }
    }
    fn route_name(msg: &str) -> Result<String, String> {
        let path = Path::new(msg);
        path.file_name()
            .map(|s| "/".to_owned() + s.to_str().unwrap())
            .map(|mut s| {
                if Path::new(msg).is_dir() {
                    s.push('/');
                }
                s
            })
            .ok_or_else(|| format!("Path '{}' dost not have name", msg))
    }
    Ok(routes)
}

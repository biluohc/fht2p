use clap::{App, Arg};
use log::LevelFilter;

use ron;

use std;
use std::collections::BTreeMap as Map;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::{fs, process, str};

use super::consts::*; // 名字,版本,作者，简介，地址

/// Get `Config` by `parse` `args`
pub fn parse() -> (Config, LevelFilter) {
    let mut config = Config::default();
    let server = Server::default();
    let routes: Vec<String> = vec!["./".to_owned()];

    let app = {
        App::new(NAME)
            .version(VERSION)
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .about(DESC)
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .multiple(true)
                    .help("Use verbose [Debug] output (support -vv.. [Trace])."),
            )
            .arg(
                Arg::with_name("config")
                    .long("config")
                    .short("c")
                    .takes_value(true)
                    .help("Set the path to a custom config file"),
            )
            .arg(
                Arg::with_name("auth")
                    .long("auth")
                    .short("a")
                    .takes_value(true)
                    .help("Set the username:password."),
            )
            .arg(
                Arg::with_name("cert")
                    .long("cert")
                    .short("C")
                    .takes_value(true)
                    .help("Set the cert for https,  private_key_file:public_key_file."),
            )
            .arg(
                Arg::with_name("config-print")
                    .long("config-print")
                    .short("P")
                    .help("Print the default config file"),
            )
            .arg(
                Arg::with_name("redircet-html")
                    .long("redircet-html")
                    .short("r")
                    .help("Redirect dir to `index.html/htm`, if it exists"),
            )
            .arg(
                Arg::with_name("keepalive")
                    .long("keepalive")
                    .short("k")
                    .help("Close HTTP keep alive"),
            )
            .arg(
                Arg::with_name("follow-links")
                    .long("follow-links")
                    .short("f")
                    .help("Whether follow links(default not)"),
            )
            .arg(
                Arg::with_name("magic-limit")
                    .long("magic-limit")
                    .short("m")
                    .takes_value(true)
                    .help("The limit for detect file ContenType(use 0 to close)"),
            )
            .arg(
                Arg::with_name("cache-secs")
                    .long("cache-secs")
                    .short("s")
                    .takes_value(true)
                    .help("Set cache secs(use 0 to close)"),
            )
            .arg(
                Arg::with_name("ip")
                    .long("ip")
                    .short("i")
                    .takes_value(true)
                    .help("Set listenning ip"),
            )
            .arg(
                Arg::with_name("port")
                    .long("port")
                    .short("p")
                    .takes_value(true)
                    .help("Set listenning port"),
            )
            .arg(
                Arg::with_name("PATH")
                    .index(1)
                    .multiple(true)
                    .help(r#"Set the paths to share"#),
            )
    };

    let matches = app.clone().get_matches();

    // -P/--config-print
    if matches.is_present("config-print") {
        config_print();
    }

    let lvl = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    //-c/--config选项，如果有就载入该文件。
    if let Some(s) = matches.value_of("config") {
        return (
            Config::load_from_file(&s)
                .map_err(|e| {
                    eprintln!("{:?}", e);
                    process::exit(1);
                })
                .unwrap(),
            lvl,
        );
    }

    let redirect_html = matches.is_present("redirect-html");
    let follow_links = matches.is_present("follow-links");
    let authorized = matches.is_present("auth");

    // 命令行有没有参数？有就解析参数，没有就寻找配置文件，再没有就使用默认配置。
    let conf = if env::args().skip(1).len() == 0 {
        match get_config_path() {
            Some(s) => Config::load_from_file(&s)
                .map_err(|e| {
                    eprintln!("{:?}", e);
                    process::exit(1);
                })
                .unwrap(),
            None => Config::load_from_STR(),
        }
    } else {
        config.addrs.clear();
        config.addrs.push(SocketAddr::new(server.ip, server.port));
        config.routes = args_paths_to_route(&routes[..], redirect_html, follow_links, authorized)
            .map_err(|e| {
                eprintln!("{:?}", e);
                process::exit(1);
            })
            .unwrap();
        config
    };
    (conf, lvl)
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
#[serde(rename_all = "camelCase")]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Fht2p {
    setting: Setting,
    routes: Map<String, Route>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    keep_alive: bool,
    magic_limit: u64,
    cache_secs: u32,
    addrs: Vec<SocketAddr>,
    auth: Option<Auth>,
    cert: Option<Cert>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Cert {
    #[serde(rename = "pub")]
    pub pub_: String,
    pub key: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Route {
    pub path: String,

    #[serde(default)]
    pub url_components: Vec<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub follow_links: bool,
    #[serde(default)]
    pub redirect_html: bool,
    #[serde(default)]
    pub authorized: bool,
    #[serde(default)]
    pub upload: bool,
    #[serde(default)]
    pub mkdir: bool,
}

impl Route {
    fn new<S: Into<String>>(
        url: S,
        path: S,
        redirect_html: bool,
        follow_links: bool,
        authorized: bool,
    ) -> Self {
        Self {
            url_components: Vec::new(),
            url: url.into(),
            path: path.into(),
            upload: false,
            mkdir: false,
            redirect_html,
            follow_links,
            authorized,
        }
    }
}

/// `Config` for `main`
#[derive(Debug)]
pub struct Config {
    pub keep_alive: bool,
    pub cache_secs: u32,
    pub magic_limit: u64,
    pub addrs: Vec<SocketAddr>,
    pub routes: Map<String, Route>,
    pub auth: Option<Auth>,
    pub cert: Option<Cert>,
}
impl Default for Config {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert("/".to_owned(), Route::new("/", ".", false, false, false));
        Config {
            keep_alive: true,
            magic_limit: *MAGIC_LIMIT.get(),
            cache_secs: 60,
            addrs: vec![SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                8080,
            )],
            routes: map,
            auth: None,
            cert: None,
        }
    }
}

impl Config {
    fn load_from_file(path: &str) -> Result<Self, String> {
        let mut str = String::new();
        let mut file = File::open(path)
            .map_err(|e| format!("config file('{}') open fails: {}", path, e.description()))?;
        file.read_to_string(&mut str)
            .map_err(|e| format!("config file('{}') read fails: {}", path, e.description()))?;
        Self::load_from_str(path, &str)
    }
    fn load_from_str(file_name: &str, toml: &str) -> Result<Config, String> {
        let mut config = Self::default();
        config.routes.clear();
        config.addrs.clear();

        let toml: Fht2p = ron::de::from_str(toml)
            .map_err(|e| format!("config file('{}') parse fails: {}", file_name, e))?;
        config.keep_alive = toml.setting.keep_alive;
        config.magic_limit = toml.setting.magic_limit;
        config.cache_secs = toml.setting.cache_secs;
        config.addrs = toml.setting.addrs.clone();
        config.cert = toml.setting.cert.clone();
        config.auth = toml.setting.auth.clone();

        for (url, route) in &toml.routes {
            if !Path::new(&route.path).exists() {
                warn!(
                    "'{}''s routes({:?}: {:?}) is not exists",
                    file_name, url, route.path
                );
            }
            config.routes.insert(
                url.clone(),
                Route::new(
                    url.as_str(),
                    route.path.as_str(),
                    route.redirect_html,
                    route.follow_links,
                    route.authorized,
                ),
            );
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

// 家目录 ～/.config/fht2p/fht2p.toml
fn get_config_path() -> Option<String> {
    // using the home_dir function from https://crates.io/crates/dirs instead.
    #[allow(deprecated)]
    let home = std::env::home_dir()?;
    if home
        .as_path()
        .join(".config/fht2p")
        .join(CONFIG_STR_PATH)
        .exists()
    {
        Some(
            home.as_path()
                .join(".config/fht2p")
                .join(CONFIG_STR_PATH)
                .to_string_lossy()
                .into_owned(),
        )
    } else {
        None
    }
}

// 参数转换为Route url, path
fn args_paths_to_route(
    map: &[String],
    redirect_html: bool,
    follow_links: bool,
    authorized: bool,
) -> Result<Map<String, Route>, String> {
    let mut routes = Map::new();
    for (idx, path) in map.iter().enumerate() {
        if !Path::new(&path).exists() {
            warn!("{:?} is not exists", &path);
        }
        if idx == 0 {
            let route = Route::new(
                "/".to_owned(),
                path.to_string(),
                redirect_html,
                follow_links,
                authorized,
            );
            routes.insert("/".to_owned(), route);
        } else {
            let route_url = route_name(path)?;
            let route = Route::new(
                route_url.clone(),
                path.to_string(),
                redirect_html,
                follow_links,
                authorized,
            );
            if routes.insert(route_url, route).is_some() {
                return Err(format!("{} already defined", route_name(path).unwrap()));
            }
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

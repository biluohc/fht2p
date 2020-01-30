use clap::{App, Arg};
use json5;
use regex::Regex;

use std;
use std::collections::BTreeMap as Map;
use std::env;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::{process, str};

use crate::{
    config::{Auth, Cert, Config, ProxyRoute, Route},
    consts::*,
    logger::{logger_init, JoinHandle},
};

/// Get `Config` by `parse` `args`
pub fn parse() -> (Config, JoinHandle) {
    let mut config = Config::default();
    let mut server = Server::default();
    let mut routes: Vec<String> = vec!["./".to_owned()];

    let default_addr = server.ip.to_string();
    let default_port = server.port.to_string();
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
                    .help("Increases logging verbosity each use for up to 3 times(warn0_info1_debug2_trace3+)"),
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
                    .help("Set the username:password for authorization")
                    .validator(|s| {
                        s.parse::<Auth>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for auth: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("cert")
                    .long("cert")
                    .short("C")
                    .takes_value(true)
                    .help("Set the cert for https,  public_key_file:private_key_file")
                    .validator(|s| {
                        s.parse::<Cert>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for cert: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("config-print")
                    .long("config-print")
                    .help("Print the default config file"),
            )
            .arg(
                Arg::with_name("redirect-html")
                    .long("redirect-html")
                    .short("r")
                    .help("Redirect dir to `index.html` or `index.htm` if it exists"),
            )
            .arg(
                Arg::with_name("show-hider")
                    .long("show-hider")
                    .short("s")
                    .help("show entries starts with '.'"),
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
                Arg::with_name("upload")
                    .long("upload")
                    .short("u")
                    .help("Whether enable upload(default not)"),
            )
            .arg(
                Arg::with_name("mkdir")
                    .short("m")
                    .long("mkdir")
                    .help("Whether enable mkdir(default not)"),
            )
            .arg(
                Arg::with_name("magic-limit")
                    .long("magic-limit")
                    .short("M")
                    .takes_value(true)
                    .help("The limit for detect file ContenType(use 0 to close)")
                    .validator(|s| {
                        s.parse::<u64>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for magic-limit: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("cache-secs")
                    .long("cache-secs")
                    .takes_value(true)
                    .help("Set cache secs(use 0 to close)")
                    .validator(|s| {
                        s.parse::<u32>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for cache-secs: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("proxy")
                    .long("proxy")
                    .short("P")
                    // .default_value("")
                    .takes_value(true)
                    .help("Enable http tunnel proxy(CONNECT)")
                    .validator(|s| {
                        Regex::new(&s)
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for proxy: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("ip")
                    .long("ip")
                    .short("i")
                    .default_value(&default_addr)
                    .takes_value(true)
                    .help("Set listenning ip")
                    .validator(|s| {
                        s.parse::<IpAddr>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for ip: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("port")
                    .long("port")
                    .short("p")
                    .default_value(&default_port)
                    .takes_value(true)
                    .help("Set listenning port")
                    .validator(|s| {
                        s.parse::<u16>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for port: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("PATH")
                    .index(1)
                    .multiple(true)
                    .help(r#"Set the paths to share"#),
            )
    };

    let matches = app.get_matches();

    // -P/--config-print
    if matches.is_present("config-print") {
        config_print();
    }

    let join_handle = logger_init(matches.occurrences_of("verbose"));

    //-c/--config选项，如果有就载入该文件。
    if let Some(s) = matches.value_of("config") {
        let config = Config::load_from_file(&s)
            .map_err(|e| {
                error!("{:?}", e);
                process::exit(1);
            })
            .unwrap();
        return (config, join_handle);
    }

    // 命令行有没有参数？有就解析参数，没有就寻找配置文件，再没有就使用默认配置。
    let conf = if env::args().skip(1).len() == 0 {
        match get_config_path() {
            Some(s) => Config::load_from_file(&s)
                .map_err(|e| {
                    error!("{:?}", e);
                    process::exit(1);
                })
                .unwrap(),
            None => Config::load_from_STR(),
        }
    } else {
        let redirect_html = matches.is_present("redirect-html");
        let follow_links = matches.is_present("follow-links");
        let show_hider = matches.is_present("show-hider");
        let upload = matches.is_present("upload");
        let mkdir = matches.is_present("mkdir");
        let authorized = matches.is_present("auth");

        matches.value_of("ip").map(|p| server.ip = p.parse().unwrap());
        matches.value_of("port").map(|p| server.port = p.parse().unwrap());
        config.addr = SocketAddr::new(server.ip, server.port);
        config.auth = matches.value_of("auth").map(|cp| cp.parse::<Auth>().unwrap());
        config.cert = matches.value_of("cert").map(|cp| cp.parse::<Cert>().unwrap());
        config.proxy = matches.value_of("proxy").map(|s| (&ProxyRoute::new(true, s)).into());
        config.keep_alive = !matches.is_present("keepalive");
        matches.values_of_lossy("PATH").map(|args| routes = args);

        config.routes = args_paths_to_route(
            &routes[..],
            redirect_html,
            follow_links,
            show_hider,
            upload,
            mkdir,
            authorized,
        )
        .map_err(|e| {
            error!("{:?}", e);
            process::exit(1);
        })
        .unwrap();
        config
    };

    (conf, join_handle)
}

#[derive(Debug, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

impl Default for Server {
    fn default() -> Server {
        Self::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
    }
}
impl Server {
    fn new(ip: IpAddr, port: u16) -> Self {
        Server { ip: ip, port: port }
    }
    fn get_sa() -> SocketAddr {
        Self::default().into()
    }
}

impl Into<SocketAddr> for Server {
    fn into(self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

// 关键是结构体的字段名，和 json 的[name]对应
#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Serialize)]
pub struct Fht2p {
    setting: Setting,
    proxy: Option<ProxyRoute>,
    routes: Map<String, Route>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    #[serde(default = "Server::get_sa")]
    addr: SocketAddr,
    keep_alive: bool,
    magic_limit: u64,
    cache_secs: u32,
    auth: Option<Auth>,
    cert: Option<Cert>,
}

impl Config {
    fn load_from_file(path: &str) -> Result<Self, String> {
        let mut str = String::new();
        let mut file = File::open(path).map_err(|e| format!("config file('{}') open fails: {}", path, e))?;
        file.read_to_string(&mut str)
            .map_err(|e| format!("config file('{}') read fails: {}", path, e))?;
        Self::load_from_str(path, &str)
    }
    fn load_from_str(file_name: &str, json: &str) -> Result<Config, String> {
        let mut config = Self::default();
        config.routes.clear();

        let json: Fht2p = json5::from_str(json).map_err(|e| format!("config file('{}') parse fails: {}", file_name, e))?;
        config.keep_alive = json.setting.keep_alive;
        config.magic_limit = json.setting.magic_limit;
        config.cache_secs = json.setting.cache_secs;
        config.addr = json.setting.addr;
        config.cert = json.setting.cert.clone();
        config.auth = json.setting.auth.clone();
        config.proxy = json.proxy.map(|ref pc| pc.into());

        for (url, route) in &json.routes {
            if !Path::new(&route.path).exists() {
                warn!("'{}''s routes({:?}: {:?}) is not exists", file_name, url, route.path);
            }
            config.routes.insert(
                url.clone(),
                Route::new(
                    url.as_str(),
                    route.path.as_str(),
                    route.redirect_html,
                    route.follow_links,
                    route.show_hider,
                    route.upload,
                    route.mkdir,
                    route.authorized,
                ),
            );
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

// 家目录 ～/.config/fht2p/fht2p.json
fn get_config_path() -> Option<String> {
    // using the home_dir function from https://crates.io/crates/dirs instead.
    #[allow(deprecated)]
    let home = std::env::home_dir()?;
    if home.as_path().join(".config/fht2p").join(CONFIG_STR_PATH).exists() {
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
    show_hider: bool,
    upload: bool,
    mkdir: bool,
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
                show_hider,
                upload,
                mkdir,
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
                show_hider,
                upload,
                mkdir,
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

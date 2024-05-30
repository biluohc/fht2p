use clap::{App, Arg};
use json5;
use regex::Regex;

use std::collections::BTreeMap as Map;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::{env, fs, str};

use crate::{
    config::{Auth, Cert, Config, CorsConfig, ProxyRoute, Route},
    consts::*,
    logger::{logger_init, JoinHandle},
    process_exit,
};

/// Get `Config` by `parse` `args`
#[allow(clippy::option_map_unit_fn)]
pub fn parse() -> (Config, JoinHandle) {
    let mut config = Config::default();
    let mut server = Server::default();
    let mut routes: Vec<String> = vec!["./".to_owned()];

    let default_addr = server.ip.to_string();
    let default_port = server.port.to_string();
    let default_cache = config.cache_secs.to_string();
    let default_compress = config.compress_level.to_string();
    let default_magic_size = config.magic_limit.to_string();
    let app = {
        App::new(NAME)
            .version(VERSION)
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .about(DESC)
            .arg(
                Arg::with_name("qr")
                    .short("Q")
                    .long("qr-code")
                    .help("Show URL's QR code at startup"),
            )
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .multiple(true)
                    .help("Increases logging verbosity each use(warn0_info1_debug2_trace3+)"),
            )
            .arg(
                Arg::with_name("config")
                    .long("config")
                    .short("c")
                    .takes_value(true)
                    .help("Set the path to use a custom config file\ndefault path: ~/.config/fht2p/fht2p.json"),
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
                    .help("Set the cert for https,  public_cert_file:private_key_file")
                    .validator(|s| {
                        s.parse::<Cert>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for cert: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("compress")
                    .long("compress")
                    .takes_value(true)
                    .help("Set the level for index compress, should between 0~9, use 0 to close")
                    .default_value(&default_compress)
                    .validator(|s| {
                        s.parse::<u32>()
                            .map_err(|e| format!("invalid value for compress: {}", e))
                            .and_then(|x| {
                                if x <= 9 {
                                    Ok(())
                                } else {
                                    Err(format!("invalid level for compress: {}", x))
                                }
                            })
                    }),
            )
            .arg(
                Arg::with_name("config-print")
                    .short("F")
                    .long("config-print")
                    .help("Print the content of default config file"),
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
                    .help("Show entries starts with '.'"),
            )
            .arg(
                Arg::with_name("keepalive")
                    .long("keepalive")
                    .short("k")
                    .help("Close HTTP keep alive"),
            )
            .arg(
                Arg::with_name("disable-index")
                    .long("disable-index")
                    .short("d")
                    .help("Disable index(directory) view(will return 403)"),
            )
            .arg(
                Arg::with_name("follow-links")
                    .long("follow-links")
                    .short("f")
                    .help("Enable follow links"),
            )
            .arg(
                Arg::with_name("upload")
                    .long("upload")
                    .short("u")
                    .help("Enable upload function"),
            )
            .arg(Arg::with_name("mkdir").short("m").long("mkdir").help("Enable mkdir function"))
            .arg(
                Arg::with_name("magic-limit")
                    .long("magic-limit")
                    .short("M")
                    .takes_value(true)
                    .default_value(&default_magic_size)
                    .help("The size limit for detect file ContenType(use 0 to close)")
                    .validator(|s| {
                        s.parse::<u64>()
                            .map(|_| ())
                            .map_err(|e| format!("invalid value for magic-limit: {}", e))
                    }),
            )
            .arg(
                Arg::with_name("cache-secs")
                    .short("S")
                    .long("cache-secs")
                    .takes_value(true)
                    .default_value(&default_cache)
                    .help("Set the secs of cache(use 0 to close)")
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
                    .takes_value(true)
                    .help("Enable http proxy(Regular for allowed domains, empty string can allow all)")
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
                    .help("Set listenning ip address")
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
                    .help(r#"Set the paths to share [Default: "."]"#),
            )
    };

    // clap's NOTE: The first argument will be parsed as the binary name unless AppSettings::NoBinaryName is used
    let args = env::args().collect::<Vec<_>>();
    let args_is_empty = args_is_empty(args.iter());
    let matches = app.get_matches_from(args);
    let qr = matches.is_present("qr");

    // -P/--config-print
    if matches.is_present("config-print") {
        config_print();
    }

    let mut join_handle = logger_init(matches.occurrences_of("verbose"));
    let exit_with_msg = |e: String| {
        error!("{}", e);
        join_handle.join();
        process_exit(1);
    };

    //-c/--config option，use it if it exists
    if let Some(s) = matches.value_of("config") {
        let config = Config::load_from_file(&s).map_err(exit_with_msg).unwrap();
        return (config.show_qrcode(qr), join_handle);
    }

    // 命令行有没有参数？有就解析参数，没有就寻找配置文件，再没有就使用默认配置。
    let conf = if args_is_empty {
        match get_config_path() {
            Some(s) => Config::load_from_file(&s).map_err(exit_with_msg).unwrap(),
            None => Config::load_from_STRING(),
        }
    } else {
        let redirect_html = matches.is_present("redirect-html");
        let follow_links = matches.is_present("follow-links");
        let disable_index = matches.is_present("disable-index");
        let show_hider = matches.is_present("show-hider");
        let authorized = matches.is_present("auth");
        let upload = matches.is_present("upload");
        let mkdir = matches.is_present("mkdir");

        matches.value_of("ip").map(|p| server.ip = p.parse().unwrap());
        matches.value_of("port").map(|p| server.port = p.parse().unwrap());
        matches
            .value_of("cache-secs")
            .map(|cs| config.cache_secs = cs.parse::<u32>().unwrap());
        matches
            .value_of("compress")
            .map(|cp| config.compress_level = cp.parse::<u32>().unwrap());

        config.addr = SocketAddr::new(server.ip, server.port);
        config.auth = matches.value_of("auth").map(|cp| cp.parse::<Auth>().unwrap());
        config.cert = matches.value_of("cert").map(|cp| cp.parse::<Cert>().unwrap());
        config.proxy = matches.value_of("proxy").map(|s| ProxyRoute::new(true, s).into());
        config.keep_alive = !matches.is_present("keepalive");
        matches.values_of_lossy("PATH").map(|args| routes = args);

        config.routes = args_paths_to_route(&routes[..], |r| {
            r.disable_index(disable_index)
                .redirect_html(redirect_html)
                .follow_links(follow_links)
                .show_hider(show_hider)
                .authorized(authorized)
                .upload(upload)
                .mkdir(mkdir)
        })
        .map_err(exit_with_msg)
        .unwrap();
        config
    };

    (conf.show_qrcode(qr), join_handle)
}

fn config_print() {
    println!("{}", CONFIG_STRING);
    process_exit(0);
}

// not contains other than -v*, -Q, --qr-code/--verbose, but -vs ?
pub fn args_is_empty<S: AsRef<str>>(args: impl Iterator<Item = S>) -> bool {
    let regex = Regex::new(r"^((-[vQ]{1,})|(--verbose)|(--qr-code))$").unwrap();

    args.skip(1).all(|arg| {
        // arg.as_ref().starts_with("-v") || arg.as_ref().starts_with("-Q") || ["--verbose", "--qr-code"].contains(&arg.as_ref())
        regex.is_match(arg.as_ref())
    })
}

#[test]
fn args_is_empty_test() {
    assert!(args_is_empty(["", "-v"].iter()));
    assert!(args_is_empty(["", "--verbose"].iter()));
    assert!(args_is_empty(["", "-Q"].iter()));
    assert!(args_is_empty(["", "--qr-code"].iter()));
    assert!(args_is_empty(["", "-vv"].iter()));
    assert!(args_is_empty(["", "-vQ"].iter()));
    assert!(args_is_empty(["", "-QQ"].iter()));

    assert!(!args_is_empty(["", "-vs"].iter()));
    assert!(!args_is_empty(["", "-Qr"].iter()));
    assert!(!args_is_empty(["", "--verbose0"].iter()));
    assert!(!args_is_empty(["", "--qr-code "].iter()));
    assert!(!args_is_empty(["", " --qr-code "].iter()));
    assert!(!args_is_empty(["", " -v"].iter()));
    assert!(!args_is_empty(["", "-v "].iter()));
    assert!(!args_is_empty(["", " -v "].iter()));
    assert!(!args_is_empty(["", "-Q", "-i"].iter()));
    assert!(!args_is_empty(["", "-v", "--ip"].iter()));
    assert!(!args_is_empty(["", "-vvQr"].iter()));
    assert!(!args_is_empty(["", "-r"].iter()));
    assert!(!args_is_empty(["", "-ip"].iter()));
    assert!(!args_is_empty(["", "./-v"].iter()));
    assert!(!args_is_empty(["", "-"].iter()));
    assert!(!args_is_empty(["", "--"].iter()));
    assert!(!args_is_empty(["", ""].iter()));
}

#[derive(Debug, Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

impl Default for Server {
    fn default() -> Server {
        let addr = SERVER_ADDR.get();
        Self::new(addr.ip(), addr.port())
    }
}
impl Server {
    fn new(ip: IpAddr, port: u16) -> Self {
        Server { ip, port }
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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
    compress_level: u32,
    auth: Option<Auth>,
    cert: Option<Cert>,
    cors: Option<CorsConfig>,
}

impl Config {
    fn load_from_file(path: &str) -> Result<Self, String> {
        debug!("Config.load_from_file: {}", path);
        let str = fs::read_to_string(path).map_err(|e| format!("config file('{}') read failed: {}", path, e))?;
        Self::load_from_str(path, &str)
    }
    fn load_from_str(file_name: &str, json: &str) -> Result<Config, String> {
        let mut config = Self::default();

        let Fht2p { setting, proxy, routes } =
            json5::from_str(json).map_err(|e| format!("config file('{}') parse failed: {}", file_name, e))?;

        let Setting {
            addr,
            auth,
            cert,
            cors,
            keep_alive,
            magic_limit,
            cache_secs,
            compress_level,
        } = setting;

        config.addr = addr;
        config.cert = cert;
        config.auth = auth;
        config.keep_alive = keep_alive;
        config.cache_secs = cache_secs;
        config.magic_limit = magic_limit;
        config.compress_level = compress_level;
        config.proxy = proxy.map(|pc| pc.into());
        config.cors = cors.unwrap_or_default();

        config.routes = routes;
        if config.routes.is_empty() {
            return Err(format!("'{}''s routes is empty", file_name));
        } else {
            for (url, route) in &mut config.routes {
                route.url = url.clone();
                if !Path::new(&route.path).exists() {
                    warn!("'{}''s routes({:?}: {:?}) is not exists", file_name, url, route.path);
                }
            }
        }

        Ok(config)
    }
    #[allow(non_snake_case)]
    fn load_from_STRING() -> Self {
        Config::load_from_str("CONFIG-STRING", CONFIG_STRING).unwrap()
    }
}

// Home: ～/.config/fht2p/fht2p.json
fn get_config_path() -> Option<String> {
    // using the home_dir function from https://crates.io/crates/dirs instead.
    #[allow(deprecated)]
    let home = std::env::home_dir()?;
    let confp = home.as_path().join(".config/fht2p").join(CONFIG_FILE_NAME);
    if confp.exists() {
        Some(confp.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn route_name(p: &str) -> Result<String, String> {
    let path = Path::new(p);
    path.file_name()
        .map(|s| {
            let s = s.to_str().expect("route_name invalid");
            if path.is_dir() {
                format!("/{}/", s)
            } else {
                format!("/{}", s)
            }
        })
        .ok_or_else(|| format!("Path '{}' dost not have name", p))
}

fn args_paths_to_route<F>(map: &[String], f: F) -> Result<Map<String, Route>, String>
where
    F: Fn(Route) -> Route,
{
    let mut routes = Map::new();
    for (idx, path) in map.iter().enumerate() {
        if !Path::new(&path).exists() {
            warn!("{:?} is not exists", &path);
        }
        if idx == 0 {
            let route = f(Route::new("/", path));
            routes.insert("/".to_owned(), route);
        } else {
            let route_url = route_name(path)?;
            let route = f(Route::new(&route_url, path));

            if routes.insert(route_url, route).is_some() {
                return Err(format!("{} already defined", route_name(path).unwrap()));
            }
        }
    }

    Ok(routes)
}

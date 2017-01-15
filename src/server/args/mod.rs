use std::path::Path;
use std::error::Error;
use std::collections::{HashSet, HashMap};
use std;

extern crate ini;
use ini::Ini;

use super::consts::*; //名字和版本
mod app;
use self::app::{App, Opt}; //命令行参数处理。

pub fn get_config() -> Result<Config, String> {
    let app = App::new()
        .name(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .address(ADDRESS)
        .opt(Opt::new("config").short("cf").long("config").help("Sets a custom config file"))
        .opt(Opt::new("cp")
            .short("cp")
            .long("cp")
            .has_value(false)
            .values((vec!["cp"]).into_iter())
            .fun(cp)
            .help("Print the default config file"))
        .opt(Opt::new("ip").short("i").long("ip").help("Sets listenning ip"))
        .opt(Opt::new("port").short("p").long("port").help("Sets listenning port"))
        .opt(Opt::new("keep-alive")
            .short("ka")
            .long("keep-alive")
            .values(vec!["0", "false", "1", "true"].into_iter())
            .help("Whether use keep-alive"))
        .arg_name("PATHS")
        .arg("Sets paths to share")
        .get();
    let app_ini_str = &app.to_ini();
    // println!("APP(args::37):\n{}", app_ini_str);

    if let Ok(app_ini) = Ini::load_from_str(app_ini_str) {
        // 参数里config值是否为null,否则读取它。
        if let Ok(setting) = get_section(&app_ini, "Setting") {
            if let Ok(config_file) = get_value(&setting, "config") {
                if config_file.to_lowercase() != "null" {
                    return Config::load_from_file(&config_file);
                }
            }
        }
        // 如果参数长度为 1 ,读取默认配置文件
        if app.len == 1 {
            if let Some(config_path) = get_config_path() {
                return Config::load_from_file(&config_path);
            } else {
                return Config::load_from_str(CONFIG);
            }
        }
        // 读取命令行参数，转为Config返回。
        let mut config = Config::load_from_str(CONFIG)?;
        let setting = get_section(&app_ini, "Setting")?;
        if get_value(setting, "ip").unwrap().to_lowercase() != "null" {
            config.ip = get_value(setting, "ip").unwrap();
        };
        if get_value(setting, "port").unwrap().to_lowercase() != "null" {
            config.port = vec![get_value(setting, "port").unwrap().trim().parse().unwrap()];
        };
        if get_value(setting, "keep-alive").unwrap().to_lowercase() != "null" {
            config.keep_alive = match get_value(setting, "keep-alive").unwrap().as_ref() {
                "0" | "false" => false,
                "1" | "true" => true,
                _ => unreachable!(),
            };
        };
        config.route = {
            let mut route: HashMap<String, String> = HashMap::new();
            for (k, v) in app.args.iter() {
                if *k == 0 {
                    let _ = route.insert("/".to_string(), v.to_string());
                    continue;
                }
                use std::path::Path;
                let path = Path::new(v);
                let pathp = match path.file_name() {
                    Some(s) => s.to_string_lossy().into_owned(),
                    None => {
                        err!("The path: {:?}  has't name, can't to confirm route's name except \
                              it is the first elements",
                             v);
                        std::process::exit(1);
                    }
                };
                if route.insert(pathp.clone(), v.to_string()).is_some() {
                    err!("route's name repeat(dir's name): {}", v);
                    std::process::exit(1);
                }
            }
            route
        };
        return Ok(config);
    }
    return Config::load_from_str(CONFIG);
}

#[derive(Debug,Clone)]
pub struct Config {
    pub ip: String,
    pub port: Vec<u32>,
    pub keep_alive: bool,
    pub route: HashMap<String, String>,
}

impl Config {
    fn load_from_file(file_name: &str) -> Result<Config, String> {
        use std::io::Read;
        use std::fs::File;
        let file = File::open(file_name);
        let mut file_str = String::new();

        // if file.is_ok() && file.unwrap().read_to_string(&file_str).is_ok() {
        // } else {
        //     return Err(format!("config file '{}' is invalid.", file_name));
        // }
        match file {
            Ok(mut ok) => {
                if let Err(e) = ok.read_to_string(&mut file_str) {
                    return Err(format!("config file '{}' is invalid: {}",
                                       file_name,
                                       e.description()));
                }
            }
            Err(e) => {
                return Err(format!("config file '{}' is invalid: {}",
                                   file_name,
                                   e.description()))
            }
        };
        Config::load_from_str(&file_str)
    }
    fn load_from_str<T: AsRef<str>>(str: T) -> Result<Config, String> {
        let ini = match Ini::load_from_str(str.as_ref()) {
            Ok(ok) => ok,
            Err(e) => return Err(format!("config is not a valid ini: {}", e.description())),
        };
        let setting = get_section(&ini, "Setting")?;
        let ip = get_value(&setting, "ip")?;
        let port_str = get_value(&setting, "port")?;
        let keep_alive_str = get_value(&setting, "keep-alive")?;
        let keep_alive = match keep_alive_str.as_ref() {
            "0" | "false" => false,            
            "1" | "true" => true,
            e @ _ => {
                return Err(format!("config's Setting's keep_alive is not a valid value: {}", e))
            }
        };

        let ports = port_str.split(',')
            .filter(|x| !x.is_empty())
            .map(|y| y.parse::<u32>());
        let mut port: Vec<u32> = Vec::new();
        for res in ports {
            match res {
                Ok(ok) => port.push(ok),
                Err(e) => {
                    return Err(format!("config'port has invalid: '{}' {}.",
                                       port_str,
                                       e.description()))
                }
            };
        }

        let route_str = get_section(&ini, "Route")?;
        let _ = get_value(&route_str, "/")?;

        let mut keys = HashSet::new();
        let mut route: HashMap<String, String> = HashMap::new();
        // let route_ = route_str.clone();
        for (k, v) in route_str.iter() {
            if keys.insert(k) == false {
                return Err(format!("config's Route's key is repeated: '{}'.", k));
            }
            if !Path::new(v).exists() {
                //     if k=='/' {}
                return Err(format!("config's Route's '{}' is not exists: '{}'.", k, v));
            }
            route.insert(k.to_string(), v.to_string());
        }
        Ok(Config {
            ip: ip,
            port: port,
            keep_alive: keep_alive,
            route: route,
        })
    }
}

impl Default for Config {
    fn default() -> Config {
        Config::load_from_str(CONFIG).unwrap()
    }
}

#[test]
fn config_test() {
    let config = Config::default();
    errln!("#[test]: Config::default(CONFIG:\"fht2p.ini\")\n{:?}\n",
           config);
}

fn cp() {
    println!("{}", CONFIG);
    std::process::exit(0);
}

fn get_section<'a>(ini: &'a Ini,
                   section_name: &str)
                   -> Result<&'a HashMap<String, String>, String> {
    match ini.section(Some(section_name)) {
        Some(s) => Ok(s),
        None => Err(format!("Config doesn't has '[{}]' section.", &section_name)),
    }
}
fn get_value(map: &HashMap<String, String>, key_name: &str) -> Result<String, String> {
    match map.get(key_name) {
        Some(s) => Ok(s.to_string()),
        None => Err(format!("Config doesn't has '[{}]' key.", &key_name)),
    }
}
// pub fn file_to_string() -> String {
//     use std::io::Read;
//     use std::fs::File;
//     let file = File::open(CONFIG_PATH);
//     let mut str = CONFIG.to_string();

//     if let Ok(mut file) = file {
//         let _ = file.read_to_string(&mut str);
//     }
//     str
// }

fn get_config_path() -> Option<String> {
    match std::env::home_dir() {
        Some(ref home) if home.as_path()
            .join(".config/fht2p")
            .join(CONFIG_PATH)
            .exists() => {
            Some(home.as_path()
                .join(".config/fht2p")
                .join(CONFIG_PATH)
                .to_string_lossy()
                .into_owned())
        }
        _ if std::env::current_exe().is_ok() &&
             std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(CONFIG_PATH)
            .exists() => {
            Some(std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join(CONFIG_PATH)
                .to_string_lossy()
                .into_owned())
        }
        _ if std::env::current_dir().is_ok() &&
             std::env::current_dir().unwrap().join(CONFIG_PATH).exists() => {
            Some(std::env::current_dir()
                .unwrap()
                .join(CONFIG_PATH)
                .to_string_lossy()
                .into_owned())
        }
        _ => None,
    }
}
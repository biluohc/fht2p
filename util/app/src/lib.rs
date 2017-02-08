#![allow(stable_features)]
#![feature(repeat_str)]
#![allow(dead_code)]
use std::collections::{HashSet, HashMap};
use std::process::exit;
use std::env;

#[macro_use]
extern crate stderr;
use stderr::Loger;
// tini

fn main() {
    init!();
    app_test();
}

fn app_test() {
    let app = App::new("fht2p")
        .version("0.5.2")
        .author("Wspsxing", "biluohc@qq.com>")
        .address("Repository", "https://github.com/biluohc/fht2p")
        .about("A HTTP Server for Static File written with Rust")
        .opt(Opt::new("config").short("cf").long("config").help("Sets a custom config file"))
        .flag(Flag::new("cp")
            .short("cp")
            .long("cp")
            .help("Print the default config file"))
        .opt(Opt::new("ip").short("i").long("ip").help("Sets listenning ip"))
        .opt(Opt::new("port").short("p").long("port").help("Sets listenning port"))
        .opt(Opt::new("keep_alive")
            .short("ka")
            .long("keep-alive")
            .valid_values(vec!["0", "false", "1", "true"].into_iter())
            .help("Whether use keep-alive"))
        .args_name("PATHS")
        .args_help("Sets the paths to share")
        .get();
    println!("\nApp_Debug:\n{:?}\n", app);
}

#[derive(Debug)]
#[derive(Clone)]
pub struct App<'app> {
    // 环境信息
    pub current_exe: Result<String, String>, // 可执行文件路径
    pub current_dir: Result<String, String>, // 当前目录
    pub home_dir: Option<String>, // home目录
    pub args_len: usize, // 参数总长度
    // App信息
    pub name: &'app str,
    pub version: Option<&'app str>,
    pub author: Vec<(&'app str, &'app str)>, /* 名字-邮箱>,可以is_empty,每次调用添加一个作者+邮箱。 */
    pub address: Vec<(&'app str, &'app str)>, /* 项目地址/主页，每次调用添加名字链接。Vec顺序可控。 */
    pub about: Option<&'app str>, // 简介
    // 参数信息。
    pub opts: HashMap<String, Opt<'app>>, // options
    pub flags: HashMap<String, Flag<'app>>, // flags
    pub args_name: Option<&'app str>, // 其余参数名字，None表示不收集，默认不收集。
    pub args: HashMap<usize, String>, // 其余参数及其编号。
    pub args_default: Option<&'app str>,
    pub args_must: bool, // 是否必须。
    pub args_help: Option<&'app str>,
}

#[derive(Debug,Clone)]
pub struct Flag<'app> {
    flag_name: &'app str,
    short: Option<&'app str>, // -h
    long: Option<&'app str>, // --help
    value: Option<bool>, // 是否出现,default方法直接改值。
    must: bool, // default is false
    help: Option<&'app str>,
}
impl<'app> Flag<'app> {
    pub fn new<'s: 'app>(flag_name: &'s str) -> Flag<'app> {
        Flag {
            flag_name: flag_name,
            short: None,
            long: None,
            value: None,
            must: false,
            help: None,
        }
    }
    pub fn short<'s: 'app>(mut self, short: &'s str) -> Self {
        self.short = Some(short);
        self
    }
    pub fn long<'s: 'app>(mut self, long: &'s str) -> Self {
        self.long = Some(long);
        self
    }
    pub fn default<'s: 'app>(mut self, default: bool) -> Self {
        self.value = Some(default);
        self
    }
    pub fn must<'s: 'app>(mut self, must: bool) -> Self {
        self.must = must;
        self
    }
    pub fn help<'s: 'app>(mut self, help: &'s str) -> Self {
        self.help = Some(help);
        self
    }
    pub fn value<'s: 'app>(&self) -> Option<bool> {
        self.value.clone()
    }
}
#[derive(Debug,Clone)]
pub struct Opt<'app> {
    opt_name: &'app str, // key
    short: Option<&'app str>, // -n
    long: Option<&'app str>, // --name
    value: Option<String>, // 是否有值（出现）,如果有的话，是否合法。
    valid_values: HashSet<String>, // 合法值集合。
    default: Option<&'app str>, // default value
    must: bool, // default is false
    help: Option<&'app str>,
}
impl<'app> Opt<'app> {
    pub fn new<'s: 'app>(opt_name: &'s str) -> Opt<'app> {
        Opt {
            opt_name: opt_name,
            short: None,
            long: None,
            value: None,
            valid_values: HashSet::new(),
            default: None,
            must: false,
            help: None,
        }
    }
    pub fn short<'s: 'app>(mut self, short: &'s str) -> Self {
        self.short = Some(short);
        self
    }
    pub fn long<'s: 'app>(mut self, long: &'s str) -> Self {
        self.long = Some(long);
        self
    }
    pub fn default<'s: 'app>(mut self, default: &'s str) -> Self {
        self.default = Some(default);
        self
    }
    pub fn valid_values<'s: 'app, T: Iterator>(mut self, values: T) -> Self
        where <T as std::iter::Iterator>::Item: std::fmt::Display
    {
        for value_valid in values {
            self.valid_values.insert(format!("{}", value_valid));
        }
        self
    }
    pub fn must<'s: 'app>(mut self, must: bool) -> Self {
        self.must = must;
        self
    }
    pub fn help<'s: 'app>(mut self, help: &'s str) -> Self {
        self.help = Some(help);
        self
    }
    pub fn value<'s: 'app>(&self) -> Option<String> {
        self.value.clone()
    }
}

impl<'app> App<'app> {
    pub fn new<'s: 'app>(name: &'s str) -> App<'app> {
        use std::path::PathBuf;
        use std::error::Error;
        let res = |p: std::io::Result<PathBuf>| match p {
            Err(e) => Err(e.description().to_owned()),
            Ok(ok) => Ok(ok.to_string_lossy().into_owned()),
        };
        let app = App {
            current_exe: res(env::current_exe()),
            current_dir: res(env::current_dir()),
            home_dir: env::home_dir().map(|x| x.to_string_lossy().into_owned()),
            args_len: env::args().len(),

            name: name,
            version: None,
            author: Vec::new(),
            address: Vec::new(),
            about: None,

            opts: HashMap::new(),
            flags: HashMap::new(),
            args_name: None,
            args: HashMap::new(),
            args_default: None,
            args_must: false,
            args_help: None,
        };
        let app = app.flag(Flag::new("help")
            .short("h")
            .long("help")
            .help("Print this help message"));
        app.flag(Flag::new("version")
            .short("V")
            .long("version")
            .help("Print the version"))
    }
    pub fn version<'s: 'app>(mut self, version: &'s str) -> Self {
        self.version = Some(version);
        self
    }
    pub fn author<'s: 'app>(mut self, name: &'s str, email: &'s str) -> Self {
        self.author.push((name, email));
        self
    }
    pub fn address<'s: 'app>(mut self, address_name: &'s str, address: &'s str) -> Self {
        self.address.push((address_name, address));
        self
    }
    pub fn about<'s: 'app>(mut self, about: &'s str) -> Self {
        self.about = Some(about);
        self
    }
    pub fn args_name<'s: 'app>(mut self, args_name: &'s str) -> Self {
        self.args_name = Some(args_name);
        self
    }
    pub fn args_default<'s: 'app>(mut self, default: &'s str) -> Self {
        self.args_default = Some(default);
        self
    }
    pub fn args_must<'s: 'app>(mut self, must: bool) -> Self {
        self.args_must = must;
        self
    }
    pub fn args_help<'s: 'app>(mut self, help: &'s str) -> Self {
        self.args_help = Some(help);
        self
    }
    pub fn opt(mut self, b: Opt<'app>) -> Self {
        if b.short == None && b.long == None {
            err!("short and long can't be empty all: {:?}", b);
            std::process::exit(1);
        }
        if let Some(s) = self.opts.insert(b.opt_name.to_owned(), b) {
            err!("option's name is repeated: {:?}", s);
            std::process::exit(1);
        }
        self
    }
    pub fn flag(mut self, b: Flag<'app>) -> Self {
        if b.short == None && b.long == None {
            err!("short and long can't be empty all: {:?}", b);
            std::process::exit(1);
        }
        if let Some(s) = self.flags.insert(b.flag_name.to_owned(), b) {
            err!("flag's name is repeated: {:?}", s);
            std::process::exit(1);
        };
        self
    }
    pub fn flags(&'app self) -> &'app HashMap<String, Flag<'app>> {
        &self.flags
    }
    pub fn get(mut self) -> Self {
        let mut args: Vec<String> = Vec::new();
        for (i, arg) in env::args().enumerate() {
            if i == 0 {
                // 消耗第一个。
                continue;
            }
            if arg == "-h" || arg == "--help" {
                self.help();
                unreachable!();
            }
            if arg == "-V" || arg == "--version" {
                self.ver();
                unreachable!();
            }
            args.push(arg);
        }
        // 收集选项到名字的映射。
        let mut short_to_name: HashMap<&'app str, String> = HashMap::new();
        let mut long_to_name: HashMap<&'app str, String> = HashMap::new();
        for (k, v) in self.flags.iter() {
            if let Some(s) = v.short {
                if let Some(s) = short_to_name.insert(s, k.clone()) {
                    errln!("The flag is repeated： {:?}", s);
                    exit(1);
                }
            }
            if let Some(s) = v.long {
                if let Some(s) = long_to_name.insert(s, k.clone()) {
                    errln!("The flag is repeated： {:?}", s);
                    exit(1);
                }
            }
        }
        for (k, v) in self.opts.iter() {
            if let Some(s) = v.short {
                if let Some(s) = short_to_name.insert(s, k.clone()) {
                    errln!("The flag is repeated： {:?}", s);
                    exit(1);
                }
            }
            if let Some(s) = v.long {
                if let Some(s) = long_to_name.insert(s, k.clone()) {
                    errln!("The flag is repeated： {:?}", s);
                    exit(1);
                }
            }
        }
        // option的值是否合法。
        fn value_valid(value: &str, opt: Option<&Opt>) -> bool {
            if let Some(opt) = opt {
                if opt.valid_values.is_empty() {
                    return true;
                }
                for valid_value in &opt.valid_values {
                    if &value == valid_value {
                        return true;
                    }
                }
            }
            false
        }
        let mut i = 0;
        for _ in 0..args.len() {
            if i >= args.len() {
                break;
            }
            let arg = &args[i];

            if arg.starts_with("--") && arg.len() > 2 {
                let opt_name = match long_to_name.get(&arg[2..]) {
                    Some(s) => s,
                    None => {
                        err!("Don't have the option: {:?}", arg);
                        exit(1);
                    }
                };
                match &arg[2..] {
                    _ if self.flags.get(opt_name).is_some() => {
                        let opt = self.flags.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            opt.value = Some(true);
                        }
                        i += 1;
                        continue;
                    }
                    _ if self.opts.get(opt_name).is_some() => {
                        if args.len() <= i + 1 {
                            err!("Null value for the option: {:?}", arg);
                            exit(1);
                        }
                        let opt = self.opts.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            if opt.valid_values.is_empty() ||
                               opt.valid_values.contains(&args[i + 1]) {
                                opt.value = Some((&args[i + 1]).to_string());
                            } else {
                                err!("Invalid value '{}' for the option: {:?}", &args[i + 1], arg);
                                exit(1);
                            }
                        }
                        i += 2;
                        continue;
                    }
                    _ => unreachable!(),
                };
            } else if arg.starts_with("-") && arg.len() > 1 {
                let opt_name = match short_to_name.get(&arg[1..]) {
                    Some(s) => s,
                    None => {
                        err!("Don't have the option: {:?}", arg);
                        exit(1);
                    }
                };
                let opt_name = &opt_name.to_string();
                match &arg[1..] {
                    _ if self.flags.get(opt_name).is_some() => {
                        let opt = self.flags.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            opt.value = Some(true);
                        }
                        i += 1;
                        continue;
                    }
                    _ if self.opts.get(opt_name).is_some() => {
                        if args.len() <= i + 1 {
                            err!("Null value for the option: {:?}", arg);
                            exit(1);
                        }
                        let opt = self.opts.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            if opt.valid_values.is_empty() ||
                               opt.valid_values.contains(&args[i + 1]) {
                                opt.value = Some((&args[i + 1]).to_string());
                            } else {
                                err!("Invalid value '{}' for the option: {:?}", &args[i + 1], arg);
                                exit(1);
                            }
                        }
                        i += 2;
                        continue;
                    }
                    _ => unreachable!(),
                };
            } else {
                let args_len = self.args.len();
                self.args.insert(args_len, arg.to_string());
                i += 1;
            }
        }
        // 处理default和must
        for v in self.opts.values_mut() {
            if v.value.is_none() && v.default.is_some() {
                v.value = Some(v.default.as_ref().unwrap().to_string());
            }
            if v.value.is_none() && v.must {
                err!("option: {}(-{:?}/--{:?}) is must",
                     v.opt_name,
                     v.short,
                     v.long);
                exit(1);
            }
        }
        for v in self.flags.values_mut() {
            if v.value.is_none() && v.must {
                err!("option: {}(-{:?}/--{:?}) is must",
                     v.flag_name,
                     v.short,
                     v.long);
                exit(1);
            }
        }
        if self.args.is_empty() && self.args_default.is_some() {
            self.args.insert(0, self.args_default.unwrap().to_string());
        }
        if self.args.is_empty() && self.args_must {
            err!("args: '{}' is must", self.args_name.unwrap());
            exit(1);
        }
        self
    }
    fn help(self) {
        if self.version.is_some() {
            println!("{}  v{}", self.name, self.version.unwrap());
        } else {
            println!("{}", self.name);
        }
        for (author, email) in self.author {
            println!("{}  <{}>", author, email);
        }
        for (name, addr) in self.address {
            println!("{}:  {}", name, addr);
        }
        if let Some(s) = self.about {
            println!("{}\n", s);
        }
        println!("USAGE:\n");
        if self.args_name.is_some() && !self.opts.is_empty() {
            println!("\t{} [{}] [{}]",
                     self.name,
                     "OPTIONS",
                     self.args_name.unwrap());
        }
        println!("\t{} {}\n", self.name, "FLAG");
        // 打印FLAGS
        println!("FLAGS:");
        let mut vec_flag: Vec<Vec<String>> = Vec::new();
        for flag in self.flags.values() {
            let mut tmp = Vec::new();
            let mut str0 = String::new();
            if let Some(s) = flag.short {
                str0 = str0 + &format!("-{}", s);
            }
            if let Some(s) = flag.long {
                if !str0.is_empty() {
                    str0 += ", ";
                }
                str0 = str0 + &format!("--{}", s);
            }
            if let Some(help) = flag.help {
                tmp.push(str0);
                tmp.push(help.to_string());
            } else {
                tmp.push(str0);
            }
            vec_flag.push(tmp);
        }
        let mut len_max = 0;
        for str in vec_flag.iter() {
            if str[0].len() > len_max {
                len_max = str[0].len();
            }
        }
        let blanks = |msg: &String, len| {
            let blank_num = len - msg.len();
            String::new() + &msg + &" ".repeat(blank_num)
        };
        for strs in vec_flag.iter() {
            if strs.len() > 1 {
                println!("\t{}{}", blanks(&strs[0], len_max + 4), strs[1]);
            } else {
                println!("\t{}", blanks(&strs[0], len_max + 4));
            }
        }

        // 打印OPTIONS
        println!("\nOPTIONS:");
        let mut vec_opts: Vec<Vec<String>> = Vec::new();
        for opt in self.opts.values() {
            let mut tmp = Vec::new();
            let mut str0 = String::new();
            if let Some(s) = opt.short {
                str0 = str0 + &format!("-{}", s);
            }
            if let Some(s) = opt.long {
                if !str0.is_empty() {
                    str0 += ", ";
                }
                str0 = str0 + &format!("--{}", s);
            }
            str0 += &format!(" <{}>", opt.opt_name);
            if let Some(help) = opt.help {
                tmp.push(str0);
                tmp.push(help.to_string());
            } else {
                tmp.push(str0);
            }
            vec_opts.push(tmp);
        }
        let mut len_max = 0;
        for str in vec_opts.iter() {
            if str[0].len() > len_max {
                len_max = str[0].len();
            }
        }
        for strs in vec_opts.iter() {
            if strs.len() > 1 {
                println!("\t{}{}", blanks(&strs[0], len_max + 4), strs[1]);
            } else {
                println!("\t{}", blanks(&strs[0], len_max + 4));
            }
        }
        if let Some(s) = self.args_name {
            println!("\nARGS");
            println!("<{}>\t\t\t{}", s, self.args_help.unwrap());
        }
        exit(0);
    }

    fn ver(&self) {
        println!("{}  v{}", self.name, self.version.unwrap());
        exit(0);
    }

    pub fn get_opt(&self, key: &str) -> Option<String> {
        if let Some(s) = self.opts.get(key) {
            if let Some(s) = s.value.as_ref() {
                Some(s.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn get_flag(&self, key: &str) -> Option<bool> {
        if let Some(s) = self.flags.get(key) {
            if let Some(s) = s.value.as_ref() {
                Some(s.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn get_args(&self) -> Option<HashMap<usize, String>> {
        if self.args_name.is_some() {
            Some(self.args.clone())
        } else {
            None
        }
    }
}

 #![allow(dead_code)]
use std::collections::{HashSet, HashMap, BTreeMap, BTreeSet};
use std;

fn main() {
    app_test();
}

fn app_test() {
    let app = App::new()
        .name("fht2p")
        .version("0.5.2")
        .author("Wspsxing <biluohc@qq.com>")
        .opt(Opt::new("config").short("cf").long("config").help("Sets a custom config file"))
        .opt(Opt::new("cp")
            .short("cp")
            .long("cp")
            .has_value(false)
            .values((vec!["cp"]).into_iter())
            .help("Print the default config file"))
        .opt(Opt::new("ip").short("i").long("ip").help("Sets listenning ip"))
        .opt(Opt::new("port").short("p").long("port").help("Sets listenning port"))
        .opt(Opt::new("keep_alive")
            .short("ka")
            .long("keep_alive")
            .values(vec!["0", "false", "1", "true"].into_iter())
            .help("Whether use keep-alive"))
        .get();
    println!("\nApp:\n{:?}\n", app);

    println!("{}", &app.to_ini());
}

// #[derive(Debug)]
#[derive(Clone)]
pub struct App<'app> {
    pub path: String,
    pub len: usize,
    pub name: Option<&'app str>,
    pub version: Option<&'app str>,
    pub author: Option<&'app str>,
    pub about: Option<&'app str>,
    pub address: Option<&'app str>,
    pub opts: BTreeMap<String, Opt<'app>>,
    pub arg_name: Option<&'app str>,
    pub arg: Option<&'app str>,
    pub args: BTreeMap<usize, String>,
}

#[derive(Debug,Clone)]
pub struct Opt<'app> {
    opt_name: &'app str, // key
    short: Option<&'app str>, // -n
    long: Option<&'app str>, // --name
    has_value: bool, // default is true(find --name value args VS rm -rf args)
    value: Option<String>,
    default: Option<&'app str>, // default value
    values: HashSet<String>, // ivalid values
    no_values: BTreeSet<String>, // all opts will store in it while has_value is false.
    // is_valid: Option<Box<FnBox(&str) -> bool + 'static>>, how to ignore a member in struct?
    fun: Option<fn()>,
    must: bool, // default is false
    // count: u32, // 出现几次。
    sl_name: HashMap<String, String>, // short/long->opt_name
    help: Option<&'app str>,
}
impl<'app> Opt<'app> {
    pub fn new<'s: 'app>(opt_name: &'s str) -> Opt<'app> {
        Opt {
            opt_name: opt_name,
            short: None,
            long: None,
            has_value: true,
            value: None,
            default: None,
            values: HashSet::new(),
            no_values: BTreeSet::new(),
            fun: None,
            must: false,
            // count: 0,
            sl_name: HashMap::new(),
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
    pub fn has_value<'s: 'app>(mut self, has_value: bool) -> Self {
        self.has_value = has_value;
        self
    }
    pub fn value<'s: 'app>(mut self, value: &'s str) -> Self {
        self.value = Some(value.to_owned());
        self
    }
    pub fn default<'s: 'app>(mut self, default: &'s str) -> Self {
        self.default = Some(default);
        self
    }
    pub fn values<'s: 'app, T: Iterator>(mut self, values: T) -> Self
        where <T as std::iter::Iterator>::Item: std::fmt::Display
    {
        for value_valid in values {
            self.values.insert(format!("{}", value_valid));
        }
        self
    }
    pub fn fun(mut self, fun: fn()) -> Self {
        self.fun = Some(fun);
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
}

impl<'app> App<'app> {
    pub fn new() -> App<'app> {
        let app = App {
            len: 0,
            path: String::new(),
            name: None,
            version: None,
            author: None,
            address: None,
            about: None,
            opts: BTreeMap::new(),
            arg_name: None,
            arg: None,
            args: BTreeMap::new(),
        };
        let app = app.opt(Opt::new("help")
            .short("h")
            .long("help")
            .has_value(false)
            .help("Print this help message"));
        app.opt(Opt::new("version")
            .short("V")
            .long("version")
            .has_value(false)
            .help("Print the version"))
    }
    pub fn name<'s: 'app>(mut self, name: &'s str) -> Self {
        self.name = Some(name);
        self
    }
    pub fn version<'s: 'app>(mut self, version: &'s str) -> Self {
        self.version = Some(version);
        self
    }
    pub fn author<'s: 'app>(mut self, author: &'s str) -> Self {
        self.author = Some(author);
        self
    }
    pub fn about<'s: 'app>(mut self, about: &'s str) -> Self {
        self.about = Some(about);
        self
    }
    pub fn address<'s: 'app>(mut self, address: &'s str) -> Self {
        self.address = Some(address);
        self
    }
    pub fn arg_name<'s: 'app>(mut self, arg_name: &'s str) -> Self {
        self.arg_name = Some(arg_name);
        self
    }
    pub fn arg<'s: 'app>(mut self, arg: &'s str) -> Self {
        self.arg = Some(arg);
        self
    }
    pub fn opt(mut self, b: Opt<'app>) -> Self {
        if b.short == None && b.long == None && b.has_value == true {
            err!("short and long can't be empty at the same time except it has'not value: {:?}",
                 b);
            std::process::exit(1);
        }
        self.opts.insert(b.opt_name.to_owned(), b);
        self
    }
    pub fn get(mut self) -> Self {
        let mut btset_short: BTreeSet<&str> = BTreeSet::new();
        let mut btset_long: BTreeSet<&str> = BTreeSet::new();
        for opt in self.opts.values() {
            if let Some(short) = opt.short {
                if btset_short.insert(short) == false {
                    err!("The option is repeated： {:?}", short);
                    std::process::exit(1);
                }
            }
            if let Some(long) = opt.long {
                if btset_long.insert(long) == false {
                    err!("The option is repeated： {:?}", long);
                    std::process::exit(1);
                }
            }
        }
        let mut sl_name: HashMap<String, String> = HashMap::new();
        for (k, v) in self.opts.clone() {
            sl_name.insert(v.short.unwrap().to_string(), k.to_string());
            sl_name.insert(v.long.unwrap().to_string(), k.to_string());
        }

        self.len = std::env::args().len();
        let mut args: Vec<String> = Vec::new();
        for (i, arg) in std::env::args().enumerate() {
            if i == 0 {
                self.path = arg;
                continue;
            }
            if arg == "-h" || arg == "--help" {
                self.help();
            }
            if arg == "-V" || arg == "--version" {
                self.ver();
            }
            args.push(arg);
        }

        fn value_valid(value: &str, opt: Option<&Opt>) -> bool {
            if let Some(opt) = opt {
                if opt.values.is_empty() {
                    return true;
                }
                for valid_value in &opt.values {
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
                let opt_name = match sl_name.get(&arg[2..]) {
                    Some(s) => s,
                    None => {
                        err!("Don't have the option: {:?}", arg);
                        std::process::exit(1);
                    }
                };
                match &arg[2..] {
                    x if !self.opts.get(opt_name).unwrap().has_value => {
                        let opt = self.opts.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            if opt.values.contains(x) {
                                opt.no_values.insert(x.to_string());
                            } else {
                                err!("Don't have the option: {:?}", arg);
                                std::process::exit(1);
                            }
                        }
                        i += 1;
                        continue;
                    }
                    _ if args.len() <= i + 1 ||
                         !value_valid(&args[i + 1], self.opts.get(opt_name)) => {
                        if args.len() <= i + 1 {
                            err!("Null value for the option: {:?}", arg);
                        } else {
                            err!("Invalid value '{}' for the option: {:?}", &args[i + 1], arg);
                        }
                        std::process::exit(1);
                    }
                    _ => {
                        let opt = self.opts.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            opt.value = Some((&args[i + 1]).to_string());
                        }
                        i += 2;
                    }
                };
            } else if arg.starts_with("-") && arg.len() > 1 {
                let opt_name = match sl_name.get(&arg[1..]) {
                    Some(s) => s,
                    None => {
                        err!("Don't have the option: {:?}", arg);
                        std::process::exit(1);
                    }
                };
                let opt_name = &opt_name.to_string();
                match &arg[1..] {
                    x if !self.opts.get(opt_name).unwrap().has_value => {
                        let opt = self.opts.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            if opt.values.contains(x) {
                                opt.no_values.insert(x.to_string());
                            } else {
                                err!("Don't have the option: {:?}", arg);
                                std::process::exit(1);
                            }
                        }
                        i += 1;
                        continue;
                    }
                    _ if args.len() <= i + 1 ||
                         !value_valid(&args[i + 1], self.opts.get(opt_name)) => {
                        if args.len() <= i + 1 {
                            err!("Null value for the option: {:?}", arg);
                        } else {
                            err!("Invalid value '{}' for the option: {:?}", &args[i + 1], arg);
                        }
                        std::process::exit(1);
                    }
                    _ => {
                        let opt = self.opts.get_mut(opt_name);
                        if let Some(mut opt) = opt {
                            opt.value = Some((&args[i + 1]).to_string());
                        }
                        i += 2;
                    }
                };
            } else {
                let args_len = self.args.len();
                self.args.insert(args_len, arg.to_string());
                i += 1;
            }
        }
        for opt_value in self.opts.values() {
            if opt_value.must == true && opt_value.value.is_none() &&
               opt_value.no_values.is_empty() {
                err!("The option is must： {:?}", opt_value.opt_name);
                std::process::exit(1);
            }
            if !opt_value.no_values.is_empty() {
                if let Some(fun) = opt_value.fun {
                    fun();
                }
            }
        }
        self
    }
    // pub fn get_value(&self, key: &str) -> Option<String> {
    //     if self.opts.get(key).is_some() && self.opts.get(key).unwrap().value.is_some() {
    //         return Some(self.opts.get(key).as_ref().unwrap().value.unwrap().to_string());
    //     } else if self.opts.get(key).is_some() &&
    //               !self.opts.get(key).unwrap().no_values.is_empty() {
    //         let mut str = String::new();
    //         for value in self.opts.get(key).unwrap().no_values.iter() {
    //             str += &format!("{}, ", value);
    //         }
    //         return Some(str);
    //     } else {
    //         None
    //     }
    // }
    pub fn to_ini(&self) -> String {
        let mut str = String::new();
        str += "encoding = utf-8\n\n";
        str += "; fht2p.ini \n[fht2p.ini]\n";
        str += &format!("name = {}\nverson = {}\nauthor = {}\naddress{}\n\n",
                        self.name.unwrap_or("null"),
                        self.version.unwrap_or("null"),
                        self.author.unwrap_or("null"),
                        self.address.unwrap_or("null"));
        str += "[Setting]\n";
        for v in self.opts.values() {
            if v.has_value == true {
                let tmp = format!("{} = {}\n; {}",
                                  v.opt_name,
                                  v.value.as_ref().unwrap_or(&"null".to_string()),
                                  v.help.unwrap_or("null"));
                str += &tmp;
                str += "\n\n";
            }
        }
        str += "[Route]\n";
        let mut route = HashSet::new();
        for (k, v) in self.args.iter() {
            if *k == 0 {
                let _ = route.insert(v.to_string());
                str += &format!("/ = {}\n", v);
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
            if route.insert(pathp.clone()) == false {
                err!("route's name repeat(dir's name): {}", v);
                std::process::exit(1);
            }
            str += &format!("{} = {}\n", pathp, v);
        }
        str
    }
    pub fn has_config(&self) -> Option<(String, String)> {
        if let Some(config) = self.opts.get("config") {
            Some(("config".to_owned(),
                  format!("{}", config.value.as_ref().unwrap_or(&"null".to_string()))))
        } else {
            None
        }
    }
    pub fn ver(&self) {
        println!("{}", self.version.unwrap_or(""));
        std::process::exit(0);
    }
    pub fn help(&self) {
        let mut str = String::new();
        str += &format!("{} {}\n{}\n{}\n{}\n\n",
                        self.name.unwrap_or(""),
                        self.version.unwrap_or(""),
                        self.author.unwrap_or(""),
                        self.about.unwrap_or(""),
                        self.address.unwrap_or(""));
        str += &match self.arg_name.is_none() {
            true => format!("USAGE:\n\t{} [OPTIONS]\n\n", self.name.unwrap_or("")),
            false => {
                format!("USAGE:\n\t{} [OPTIONS] [{}]\n\n",
                        self.name.unwrap_or(""),
                        self.arg_name.unwrap_or(""))
            }
        };

        let (mut flags, mut options) = (String::from("FLAGS:\n"), String::from("OPTIONS:\n"));
        for (k, v) in self.opts.iter() {
            match v.has_value {
                true => {
                    options += &format!("\t-{}, --{} <{}>\t\t{}\n",
                                        v.short.unwrap_or(""),
                                        v.long.unwrap_or(""),
                                        k,
                                        v.help.unwrap_or(""));
                }
                false => {
                    flags += &format!("\t-{}, --{} \t\t{}\n",
                                      v.short.unwrap_or(""),
                                      v.long.unwrap_or(""),
                                      v.help.unwrap_or(""));
                }
            };

        }
        str += &format!("{}\n{}\n", flags, options);
        if self.arg_name.is_some() {
            str += &format!("ARGS:\n\t<{}>\t\t{}\n\n",
                            self.arg_name.unwrap(),
                            self.arg.unwrap_or(""));
        }
        println!("{}", &str);
        std::process::exit(0);
    }
}

impl<'app> std::fmt::Debug for App<'app> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut opts_str = String::new();
        for (k, v) in self.opts.iter() {
            let tmp_str = &format!("{{opt_name: {:?}, short: {:?} ,long: {:?} ,has_value: {:?} \
                                    ,value: {:?} ,default: {:?} ,values: {:?} ,no_values: {:?}, \
                                    must: {:?}\nhelp: {:?}}}",
                                   v.opt_name,
                                   v.short,
                                   v.long,
                                   v.has_value,
                                   v.value,
                                   v.default,
                                   v.values,
                                   v.no_values,
                                   v.must,
                                   v.help);
            opts_str += &format!("{:?}: {:?}\n", k, tmp_str);
        }
        write!(f,
               "path: {:?}\nname: {:?}\nversion: {:?}\nauthor: {:?}\nopts:\n{}args:\n{:?}",
               self.path,
               self.name,
               self.version,
               self.author,
               opts_str,
               self.args)
    }
}

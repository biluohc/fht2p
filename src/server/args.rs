use std::path::Path;

use super::htm;
#[derive(Debug)]
pub struct Args<'a> {
    pub ip: &'a str,
    pub port: u32,
    pub dir: &'a str,
}

// fht2p -i0.0.0.0 -p8080 dir/
pub fn deal_args<'a>(args: &'a [String]) -> Result<Args<'a>, String> {
    let (mut ip, mut port, mut dir) = ("0.0.0.0", 8080u32, ".");
    for arg in args {
        if arg == "-h" || arg == "--help" {
            help();
            return Err("".to_string());
        }
        if arg == "-V" || arg == "--version" {
            version();
            return Err("".to_string());
        }
        if arg.starts_with("-i") {
            if arg.len() > 2 {
                ip = &arg[2..];
            } else {
                // panic!("{:?} is invalid", arg);
                return Err(format!("\"{}\" is invalid", arg));
            }

        }
        if arg.starts_with("-p") {
            if arg.len() > 2 {
                port = arg[2..].parse().unwrap();
            } else {
                // panic!("{:?} is invalid", arg);
                return Err(format!("\"{}\" is invalid", arg));
            }
        }
        if !arg.starts_with("-") {
            // 还有文件，应该可以，但路径不能访问？应该报错？
            if Path::new(&arg).exists() {
                dir = &arg;
            } else {
                // panic!("{:?} is not exists", arg);
                return Err(format!("\"{}\" is invalid", arg));
            }
        }
    }
    Ok(Args {
        ip: ip,
        port: port,
        dir: dir,
    })
}

fn help() {
    println!("{}/{}: A HTTP Servers for Static File Serving.\n",
             htm::NAME,
             htm::VERSION);
    let explain = r#"    
    ./fht2p  -i0.0.0.0 -p8080  dir/
     path    ip         port   dir

     default:
    ./fht2p  -i0.0.0.0 -p8080  ./

     ./fht2p -h  or  ./fht2p --help
     print this "help"

Author: biluohc@outlook.com
Github: https://github.com/biluohc/fht2p
    "#;
    println!("Using: {}", explain);
}

fn version() {
    println!("{} {}", htm::NAME, htm::VERSION);
}
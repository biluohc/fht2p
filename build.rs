use std::process::Command;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    // Cargo build --release 把build.rs的输出当参数(link搜索路径,-vv可以看到)。
    // Cargo build --release -vv 可以看到println的内容。。
    if cfg!(target_os = "windows") {
        Command::new("windres")
            .arg("config/favicon.rc")
            .arg("-o")
            .arg(&format!("{}/favicon.o", out_dir))
            .status()
            .unwrap();
    }
}

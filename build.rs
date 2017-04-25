fn main() {
    platform::main();
}

#[cfg(unix)]
mod platform {
    pub fn main() {}
}

#[cfg(windows)]
mod platform {
    use std::process::Command;
    use std::env;

    pub fn main() {
        let out_dir = env::var("OUT_DIR").unwrap();
        println!("cargo:rustc-link-search=native={}", out_dir);
        // Cargo build --release 把build.rs的输出当参数(link搜索路径,-vv可以看到)。
        // Cargo build --release -vv 可以看到println的内容。。
        Command::new("windres")
            .arg("config/favicon.rc")
            .arg("--target")
            .arg(target())
            .arg("-o")
            .arg(&format!("{}/favicon.o", out_dir))
            .status()
            .unwrap();
    }

    // build can't use if cfg!(target-point-width="64"), instead by env::var()
    fn target() -> &'static str {
        if env::var("TARGET").unwrap().contains("x86_64") {
            "pe-x86-64"
        } else {
            "pe-i386"
        }
    }
}
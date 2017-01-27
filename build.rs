#[allow(unused_imports)]
use std::process::Command;

fn main() {
    #[cfg(windows)]    {
        Command::new("windres").args(&["favicon.rc", "-o", "favicon.o"]).status().unwrap();
    }
}

// windres favicon.rc favicon.o
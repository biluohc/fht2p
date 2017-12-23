// crate's info
pub const NAME: &str = env!("CARGO_PKG_NAME");
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
include!(concat!(env!("OUT_DIR"), "/fht2p.txt"));
pub const AUTHOR: &str = "Wspsxing";
pub const EMAIL: &str = "biluohc@qq.com";
pub const DESC: &str = env!("CARGO_PKG_DESCRIPTION");
pub const URL_NAME: &str = "Github";
pub const URL: &str = "https://github.com/biluohc/fht2p";

// config file
pub const CONFIG_STR_PATH: &str = "fht2p.toml";
pub const CONFIG_STR: &str = include_str!("../fht2p.toml");

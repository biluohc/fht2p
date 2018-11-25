use std::collections::BTreeMap as Map;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use consts::*;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Cert {
    #[serde(rename = "pub")]
    pub pub_: String,
    pub key: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Route {
    pub path: String,
    #[serde(default)]
    #[serde(skip)]
    pub url_components: Vec<String>,
    #[serde(default)]
    #[serde(skip)]
    pub url: String,
    #[serde(default)]
    #[serde(skip)]
    pub follow_links: bool,
    #[serde(default)]
    pub redirect_html: bool,
    #[serde(default)]
    pub authorized: bool,
    #[serde(default)]
    pub upload: bool,
    #[serde(default)]
    pub mkdir: bool,
}

impl Route {
    pub fn new<S: Into<String>>(url: S, path: S, redirect_html: bool, follow_links: bool, authorized: bool) -> Self {
        Self {
            url_components: Vec::new(),
            url: url.into(),
            path: path.into(),
            upload: false,
            mkdir: false,
            redirect_html,
            follow_links,
            authorized,
        }
    }
}

/// `Config` for `main`
#[derive(Debug, Clone)]
pub struct Config {
    pub keep_alive: bool,
    pub cache_secs: u32,
    pub magic_limit: u64,
    pub addrs: Vec<SocketAddr>,
    pub routes: Map<String, Route>,
    pub auth: Option<Auth>,
    pub cert: Option<Cert>,
}
impl Default for Config {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert("/".to_owned(), Route::new("/", ".", false, false, false));
        Config {
            keep_alive: true,
            magic_limit: *MAGIC_LIMIT.get(),
            cache_secs: 60,
            addrs: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)],
            routes: map,
            auth: None,
            cert: None,
        }
    }
}

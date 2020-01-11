use tokio_rustls::rustls::{self, internal::pemfile};
pub use tokio_rustls::TlsAcceptor;

use std::{collections::BTreeMap as Map, fs, io, net::SocketAddr, sync::Arc};

use crate::{args::Server, consts::*, how::Result};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Cert {
    #[serde(rename = "pub")]
    pub pub_: String,
    pub key: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProxyRoute {
    #[serde(default)]
    authorized: bool,
    // regex string
    path: String,
}

impl Into<Route> for &ProxyRoute {
    fn into(self) -> Route {
        let mut new = Route::default();
        new.authorized = self.authorized;
        new.path = self.path.clone();
        new
    }
}

pub fn load_certs(path: &str) -> Result<Vec<rustls::Certificate>> {
    let certfile = fs::File::open(path).map_err(|e| format_err!("open certificate file({}) failed: {:?}", path, e))?;
    let mut reader = io::BufReader::new(certfile);
    pemfile::certs(&mut reader).map_err(|e| format_err!("load certificate({}) failed: {:?}", path, e))
}

pub fn load_private_key(path: &str) -> Result<rustls::PrivateKey> {
    let keyfile = fs::File::open(path).map_err(|e| format_err!("open private key file({}) failed: {:?}", path, e))?;
    let mut reader = io::BufReader::new(keyfile);
    let mut keys =
        pemfile::rsa_private_keys(&mut reader).map_err(|e| format_err!("load private key({}) failed: {:?}", path, e))?;
    assert!(keys.len() == 1);
    Ok(keys.remove(0))
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Default, Deserialize, PartialEq, Serialize, Clone)]
pub struct Route {
    pub path: String,
    #[serde(default)]
    #[serde(skip)]
    pub urlcs: usize,
    #[serde(default)]
    #[serde(skip)]
    pub url: String,
    #[serde(default)]
    #[serde(skip)]
    pub follow_links: bool,
    #[serde(default)]
    pub redirect_html: bool,
    #[serde(default)]
    pub show_hider: bool,
    #[serde(default)]
    pub authorized: bool,
    #[serde(default)]
    pub upload: bool,
    #[serde(default)]
    pub mkdir: bool,
}

impl Route {
    pub fn new<S: Into<String>>(
        url: S,
        path: S,
        redirect_html: bool,
        follow_links: bool,
        show_hider: bool,
        authorized: bool,
    ) -> Self {
        Self {
            urlcs: 0,
            url: url.into(),
            path: path.into(),
            upload: false,
            mkdir: false,
            redirect_html,
            follow_links,
            show_hider,
            authorized,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert("/".to_owned(), Route::new("/", ".", false, false, false, false));
        Config {
            addr: Server::default().into(),
            magic_limit: *MAGIC_LIMIT.get(),
            keep_alive: true,
            cache_secs: 60,
            proxy: None,
            routes: map,
            auth: None,
            cert: None,
        }
    }
}

/// `Config` for `main`
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub keep_alive: bool,
    pub cache_secs: u32,
    pub magic_limit: u64,
    pub addr: SocketAddr,
    pub routes: Map<String, Route>,
    pub auth: Option<Auth>,
    pub cert: Option<Cert>,
    pub proxy: Option<Route>,
}

impl Config {
    pub fn load_cert(&self) -> Result<Option<TlsAcceptor>> {
        if let Some(cert) = &self.cert {
            let certs = load_certs(&cert.pub_)?;
            let key = load_private_key(&cert.key)?;
            let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());

            cfg.set_single_cert(certs, key)
                .map_err(|e| format_err!("set single cert failed: {:?}", e))?;

            return Ok(Some(TlsAcceptor::from(Arc::new(cfg))));
        }

        Ok(None)
    }
}

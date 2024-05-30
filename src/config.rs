use tokio_rustls::rustls::{self, internal::pemfile};
pub use tokio_rustls::TlsAcceptor;

use std::{collections::BTreeMap as Map, fs, io, io::Seek, net::SocketAddr, str::FromStr, sync::Arc};

use crate::{
    args::Server,
    consts::*,
    how::{Error, Result},
};

fn kv_parser(input: &str) -> Result<(&str, &str)> {
    use nom::{
        bytes::complete::{is_not, tag},
        error::ErrorKind,
        sequence::separated_pair,
    };

    separated_pair(is_not(":"), tag(":"), is_not(":"))(input)
        .map_err(|e: nom::Err<(&str, ErrorKind)>| format_err!("kv-parse failed: {:?}", e))
        .map(|(_remains, (k, v))| (k.trim(), v.trim()))
        .and_then(|(k, v)| {
            if !k.is_empty() && !v.is_empty() {
                Ok((k, v))
            } else {
                Err(format_err!("empty key or value"))
            }
        })
}

#[test]
fn kv_parser_test() {
    assert!(kv_parser(":").is_err());
    assert!(kv_parser("ab").is_err());
    assert!(kv_parser("ab:").is_err());
    assert!(kv_parser(":cd").is_err());
    assert_eq!(kv_parser("a:b").unwrap(), ("a", "b"));
    assert_eq!(kv_parser("a : b").unwrap(), ("a", "b"));
    assert_eq!(kv_parser("a/b/c:/d/e/f").unwrap(), ("a/b/c", "/d/e/f"));
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

impl FromStr for Auth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (username, password) = kv_parser(s).map(|(k, v)| (k.to_owned(), v.to_owned()))?;
        Ok(Self { username, password })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Cert {
    #[serde(rename = "pub")]
    pub pub_: String,
    pub key: String,
    #[serde(default)]
    pub hsts: Option<StrictTransportSecurity>,
}

impl FromStr for Cert {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (pub_, key) = kv_parser(s).map(|(k, v)| (k.to_owned(), v.to_owned()))?;
        Ok(Self { pub_, key, hsts: None })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrictTransportSecurity {
    max_age: u32,
    include_sub_domains: bool,
    preload: bool,
}

// HSTS: HTTP Strict Transport Security
// https://developer.mozilla.org/zh-CN/docs/Security/HTTP_Strict_Transport_Security
impl StrictTransportSecurity {
    pub fn to_header(&self) -> Option<String> {
        if self.max_age > 0 {
            let mut args = format!("max-age={}", self.max_age);
            if self.include_sub_domains {
                args.push_str("; includeSubDomains");
            }
            if self.preload {
                args.push_str("; preload");
            }
            Some(args)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProxyRoute {
    #[serde(default)]
    authorized: bool,
    // regex string
    path: String,
}

impl ProxyRoute {
    pub fn new<S: Into<String>>(authorized: bool, path: S) -> Self {
        Self {
            authorized,
            path: path.into(),
        }
    }
}

impl Into<Route> for ProxyRoute {
    fn into(self) -> Route {
        Route::new("proxy", self.path).authorized(self.authorized)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CorsConfig {
    // null => 'self'
    pub allow_referers: Option<String>,
    // null => 'self'
    pub allow_origins: Option<String>,
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
        pemfile::rsa_private_keys(&mut reader).map_err(|e| format_err!("load private key(rsa: {}) failed: {:?}", path, e))?;

    if keys.is_empty() {
        reader.seek(io::SeekFrom::Start(0))?;
        keys = pemfile::pkcs8_private_keys(&mut reader)
            .map_err(|e| format_err!("load private key(pkcs8: {}) failed: {:?}", path, e))?;
    }

    assert_eq!(keys.len(), 1);
    Ok(keys.remove(0))
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub path: String,
    #[serde(default)]
    #[serde(skip)]
    pub urlcs: usize,
    #[serde(default)]
    #[serde(skip)]
    pub url: String,
    #[serde(default)]
    pub follow_links: bool,
    #[serde(default)]
    pub redirect_html: bool,
    #[serde(default)]
    pub show_hider: bool,
    #[serde(default)]
    pub disable_index: bool,
    #[serde(default)]
    pub authorized: bool,
    #[serde(default)]
    pub upload: bool,
    #[serde(default)]
    pub mkdir: bool,
}

macro_rules! route_builder {
    ($name: ident) => {
        #[inline]
        pub fn $name(mut self, b: bool) -> Self {
            self.$name = b;
            self
        }
    };
    ($($name: ident,)*) => {
        $(route_builder!{$name})*
    }
}

impl Route {
    pub fn new<U, P>(url: U, path: P) -> Self
    where
        U: Into<String>,
        P: Into<String>,
    {
        Self {
            urlcs: 0,
            url: url.into(),
            path: path.into(),
            ..Default::default()
        }
    }

    route_builder! {
        disable_index,
        redirect_html,
        follow_links,
        show_hider,
        authorized,
        upload,
        mkdir,
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert("/".to_owned(), Route::new("/", "."));
        Config {
            addr: Server::default().into(),
            magic_limit: *MAGIC_LIMIT.get(),
            compress_level: COMPRESS_LEVEL,
            cors: Default::default(),
            show_qrcode: false,
            keep_alive: true,
            cache_secs: 60,
            proxy: None,
            routes: map,
            auth: None,
            cert: None,
            hsts_header: None,
        }
    }
}

/// `Config` for `main`
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub show_qrcode: bool,
    pub keep_alive: bool,
    pub cache_secs: u32,
    pub magic_limit: u64,
    pub addr: SocketAddr,
    pub routes: Map<String, Route>,
    pub auth: Option<Auth>,
    pub cert: Option<Cert>,
    pub proxy: Option<Route>,
    pub compress_level: u32,
    pub cors: CorsConfig,
    pub hsts_header: Option<String>,
}

impl Config {
    pub fn load_cert(&self) -> Result<Option<TlsAcceptor>> {
        if let Some(cert) = &self.cert {
            let certs = load_certs(&cert.pub_)?;
            let key = load_private_key(&cert.key)?;
            let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());

            cfg.set_single_cert(certs, key)
                .map_err(|e| format_err!("set single cert failed: {:?}", e))?;

            // Configure ALPN to accept HTTP/2, HTTP/1.1, etc
            // Have to modify the proxy module because of the request's path from hyper has domain name all when h2 is turned on
            // values: h2, http/1.1
            cfg.set_protocols(&[b"http/1.1".to_vec()]);

            HSTS_HEADER.set(cert.hsts.as_ref().and_then(|c| c.to_header()));

            return Ok(Some(TlsAcceptor::from(Arc::new(cfg))));
        }

        Ok(None)
    }

    pub fn show_qrcode(mut self, b: bool) -> Self {
        self.show_qrcode = b;
        self
    }
}

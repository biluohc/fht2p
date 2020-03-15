use hyper::{header, Uri};
use regex::Regex;

use std::net::SocketAddr;

use crate::{
    base::{
        ctx::{ctxs, Ctx},
        http::uri::InvalidUri,
        middleware::MiddleWare,
        HeaderGetStr, Request, Response,
    },
    config::CorsConfig,
    handlers::exception::exception_handler_sync,
};

#[derive(Debug, Default, Clone)]
pub struct CorsController {
    allow_referers: Option<Regex>,
    allow_origins: Option<Regex>,
    // simple request(get, head and post) not need
    // allow_credentials: bool,
}

impl CorsController {
    pub fn new(config: &CorsConfig) -> crate::how::Result<Self> {
        let f = |config: Option<&String>| -> crate::how::Result<_> {
            Ok(if let Some(s) = config { Some(Regex::new(&s)?) } else { None })
        };

        Ok(Self {
            allow_referers: f(config.allow_referers.as_ref())?,
            allow_origins: f(config.allow_origins.as_ref())?,
        })
    }
}

// https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Access_control_CORS
impl MiddleWare for CorsController {
    fn before(&self, req: &Request, addr: &SocketAddr, ctx: &mut Ctx) -> Result<(), Response> {
        let host = req.headers().get_str(header::HOST);

        let mut f = |headkey, reg: &Option<Regex>, kind| {
            if let Some(headv) = req.headers().get_str_option(headkey) {
                debug!("addr: {}, host: {}, {}: {}", addr, host, kind, headv);

                let headv_host = match uri_to_host(headv) {
                    Ok(h) => h,
                    Err(e) => {
                        warn!("{}'s request Header has wrong {}({}): {:?}", addr, kind, headv, e);
                        return Err(exception_handler_sync(400, Some("Invalid request Header"), req, addr).unwrap());
                    }
                };

                // igrore the case
                if headv_host.len() == host.len()
                    && headv_host.chars().zip(host.chars()).all(|(a, b)| a.eq_ignore_ascii_case(&b))
                    || reg.as_ref().map(|reg| reg.is_match(headv)).unwrap_or_default()
                {
                    // Access-Control-Allow-Origin: * or scheme://$host ?
                    if kind == "Origin" {
                        ctx.insert(true);
                    }
                } else {
                    return Err(exception_handler_sync(403, Some(&format!("Invalid {}", kind)), req, addr).unwrap());
                }
            }
            Ok(())
        };

        f(header::ORIGIN, &self.allow_origins, "Origin")?;
        f(header::REFERER, &self.allow_referers, "Referer")
    }
    fn after(&self, resp: &mut Response, _addr: &SocketAddr, ctx: &mut Ctx) {
        let cors = ctx.get::<ctxs::Cors>().copied().unwrap_or_default();
        if cors {
            let value = "*".parse().expect("Access-Control-Allow-Origin: *");
            resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        }
    }
}

/// Referer to HOST
pub fn uri_to_host(uri: &str) -> Result<String, InvalidUri> {
    uri.parse::<Uri>().map(|uri| match (uri.host(), uri.port()) {
        (Some(h), Some(p)) => format!("{}:{}", h, p),
        (Some(h), None) => h.to_owned(),
        _ => Default::default(),
    })
}

/// Regex is more than double slower with or without unicode enabled
pub fn uri_to_host_regex(uri: &str) -> Option<&str> {
    lazy_static! {
        static ref HOST: Regex = Regex::new(r#"\w+://(?P<host>([\w\.:\[\]]+))/"#).expect("uri_to_host_fast's Regex");
        // static ref HOST: Regex = Regex::new(r#"[[:word:]]+://(?P<host>([[[:word:]]\.:\[\]]+))/"#).expect("uri_to_host_fast's Regex");
    }
    HOST.captures(uri).and_then(|cs| cs.name("host").map(|m| m.as_str()))
}

// cargo tr  uri_to  -- --nocapture
#[test]
fn uri_to_host_test() {
    let tests = vec![
        ("http://192.168.0.29:9000/cargo/", "192.168.0.29:9000"),
        ("http://[192.168.0.29]:9000/cargo/", "[192.168.0.29]:9000"),
        ("https://crates.io/crates/nom", "crates.io"),
        ("https://2crates.io/crates/nom", "2crates.io"),
        ("https://two.crates.io/crates/nom", "two.crates.io"),
        ("https://t.io/crates/nom", "t.io"),
        ("https://t.i/crates/nom", "t.i"),
        ("https://t/crates/nom", "t"),
        ("https://./crates/nom", "."),
        ("https://[::1]/crates/nom", "[::1]"),
        ("https://[::1]:80/crates/nom", "[::1]:80"),
        // ("https://中文.cn/crates/nom", "中文.cn"), // Unicode: hyper is not supported now
    ];

    let errors = vec!["https:///crates/nom", "https:/crates/nom", "https:/crates/nom"];

    let f = |s| {
        let m = uri_to_host(s);
        println!("{:?}", m);
        m.ok()
    };

    let freg = |s| {
        let m = uri_to_host_regex(s);
        println!("{:?}", m);
        m
    };

    fn bench<F: Fn(u32)>(tag: &str, times: u32, f: F) {
        use std::time::Instant;

        let now = Instant::now();
        (0..times).into_iter().for_each(|c| f(c));
        let costed = now.elapsed();
        println!("{} {} times costed {:?}, avg time: {:?}", tag, times, costed, costed / times)
    }

    bench("regex", 100, |_| {
        tests.iter().for_each(|(i, _)| assert!(uri_to_host_regex(i).is_some()))
    });
    bench("hyper", 100, |_| {
        tests.iter().for_each(|(i, _)| assert!(uri_to_host(i).ok().is_some()))
    });

    tests.into_iter().for_each(|(i, o)| {
        assert_eq!(f(i).unwrap(), o);
        assert_eq!(freg(i).unwrap(), o);
    });

    errors.into_iter().for_each(|i| {
        assert_eq!(f(i), None);
        assert_eq!(freg(i), None);
    });
}

use futures_cpupool::{Builder, CpuPool};
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpListener;
use futures::{future, Future, Stream};
use hyper::server::{Http, Request, Response, Service};
use hyper::{header, Error, Method, StatusCode};
use url::percent_encoding::percent_decode;
use time;

use hyper_fs::{Exception, ExceptionHandlerServiceAsync};
use hyper_fs::{Config as FsConfig, FutureObject};

mod local {
    use hyper_fs::static_file::*;
    use exception::ExceptionHandler;
    static_file!(StaticFile, ExceptionHandler);
}

pub use self::local::StaticFile;
use exception::ExceptionHandler;
use content_type::headers_maker;

use index::StaticIndex;
use args::{Config, Route};
use consts;

use std::path::PathBuf;
use std::sync::Arc;
use std::rc::Rc;
use std::io;

/// main function
pub fn run(config: Config) -> io::Result<()> {
    let pool = Builder::new().pool_size(3).name_prefix("hyper-fs").create();
    let fsconfig = FsConfig::new()
        .cache_secs(config.cache_secs)
        .follow_links(config.follow_links)
        .show_index(true);
    debug!("{:?}", fsconfig);

    let mut core = Core::new()?;
    let handle = core.handle();
    let tcp = config.addrs[1..].iter().fold(
        TcpListener::bind(&config.addrs[0], &handle),
        |tcp, &addr| {
            if tcp.is_err() {
                TcpListener::bind(&addr, &handle)
            } else {
                tcp
            }
        },
    )?;
    consts::MAGIC_LIMIT.set(config.magic_limit);
    let keep_alive = config.keep_alive;

    let server = Rc::new(Server::new(handle.clone(), pool, config, fsconfig));
    let addr = tcp.local_addr()?;
    consts::SERVER_ADDR.set(addr);

    let mut http = Http::new();
    http.keep_alive(keep_alive);
    let http_server = tcp.incoming().for_each(|(socket, addr)| {
        http.bind_connection(&handle, socket, addr, server.clone());
        Ok(())
    });

    let mut info = format!(
        "{}/{} Serving at {}:{} for:\n",
        consts::NAME,
        env!("CARGO_PKG_VERSION"),
        addr.ip(),
        addr.port()
    );
    server
        .routes
        .iter()
        .for_each(|r| info.push_str(&format!("   {:?} -> {:?}\n", r.url, r.path)));
    info.push_str(&format!(
        "You can visit http://{}:{}",
        addr.ip(),
        addr.port()
    ));
    println!("{}", info);

    core.run(http_server)
}

/// `Server`
pub struct Server {
    handle: Handle,
    pool: CpuPool,
    fsconfig: Arc<FsConfig>,
    routes: Vec<Route>,
}

impl Server {
    pub fn new(handle: Handle, pool: CpuPool, config: Config, fsconfig: FsConfig) -> Self {
        let mut routes = config
            .routes
            .into_iter()
            .map(|(_, mut r)| {
                r.url_components = r.url
                    .split('/')
                    .filter(|c| !c.is_empty())
                    .map(|c| c.to_owned())
                    .collect::<Vec<_>>();
                r
            })
            .collect::<Vec<_>>();
        routes.sort_by(|a, b| a.url_components.len().cmp(&b.url_components.len()));
        Self {
            handle: handle,
            pool: pool,
            fsconfig: Arc::new(fsconfig),
            routes: routes,
        }
    }
    pub fn call2(&self, req_path: &str, req: Request) -> FutureObject {
        debug!("{:?} {:?}?{:?}", req.method(), req.path(), req.query());
        match *req.method() {
            Method::Head | Method::Get => {}
            _ => return ExceptionHandler::call_async(Exception::Method, req),
        }
        let (req_path, mut fspath, redirect_html) = match router(&self.routes, req_path) {
            Some(s) => s,
            None => return ExceptionHandler::call_async(Exception::not_found(), req),
        };
        debug!("({:?} , {:?})", req_path, fspath);

        if req_path.ends_with('/') && redirect_html {
            let mut _301 = "";
            fspath.push("index.html");
            if fspath.exists() {
                _301 = "index.html";
            } else {
                fspath.pop();
                fspath.push("index.htm");
                if fspath.exists() {
                    _301 = "index.htm";
                } else {
                    fspath.pop();
                }
            }
            if !_301.is_empty() {
                let mut new_path = req.path().to_owned();
                new_path.push_str(_301);
                if let Some(query) = req.query() {
                    new_path.push('?');
                    new_path.push_str(query);
                }
                return Box::new(future::ok(
                    Response::new()
                        .with_status(StatusCode::MovedPermanently)
                        .with_header(header::Location::new(new_path)),
                ));
            }
        }
        let metadata = if self.fsconfig.get_follow_links() {
            fspath.metadata()
        } else {
            fspath.symlink_metadata()
        };
        match metadata {
            Ok(md) => {
                let config = self.fsconfig.clone();
                if md.is_file() {
                    let mut file_server = StaticFile::new(self.handle.clone(), self.pool.clone(), fspath, config);
                    file_server.headers_maker(headers_maker);
                    file_server.call(&self.pool, req)
                } else if md.is_dir() {
                    let mut index_server = StaticIndex::new(req_path, fspath, config.clone());
                    *index_server.headers_mut() = Some(consts::HTML_HEADERS.get().clone());
                    index_server.call(&self.pool, req)
                } else {
                    ExceptionHandler::call_async(Exception::Typo, req)
                }
            }
            Err(e) => ExceptionHandler::call_async(e, req),
        }
    }
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = FutureObject;
    fn call(&self, req: Request) -> Self::Future {
        match req.remote_addr() {
            Some(addr) => {
                let mut info = (
                    addr,
                    req.method().clone(),
                    percent_decode(req.path().as_bytes())
                        .decode_utf8()
                        .unwrap()
                        .into_owned()
                        .to_owned(),
                );
                let query = req.query().map(|q| format!("?{}", q));
                let object = self.call2(&info.2, req);

                query.map(|q| info.2.push_str(&q));
                Box::new(object.inspect(move |res| {
                    let datatime = time::now();
                    println!(
                        "[{}:{}{}] {} {} {}",
                        info.0.ip(),
                        info.0.port(),
                        datatime.strftime("**%Y-%m%d/%I:%M:%S").unwrap(),
                        res.status().as_u16(),
                        info.1,
                        info.2
                    );
                }))
            }
            None => unreachable!("Request.remote_addr() is None"),
        }
    }
}

pub fn router(routes: &Vec<Route>, req_path: &str) -> Option<(String, PathBuf, bool)> {
    let mut reqpath_components = req_path
        .split('/')
        .filter(|c| !c.is_empty() && c != &".")
        .collect::<Vec<_>>();

    let parent_count = (0..reqpath_components.len())
        .into_iter()
        .rev()
        .fold(0, |count, idx| match (idx < reqpath_components.len(), idx > 0) {
            (true, true) => match (reqpath_components[idx] == "..", reqpath_components[idx - 1] == "..") {
                (false, _) => count,
                (true, false) => {
                    reqpath_components.remove(idx);
                    reqpath_components.remove(idx - 1);
                    count
                }
                (true, true) => {
                    reqpath_components.remove(idx);
                    count + 1
                }
            },
            (false, _) => count,
            (true, false) => {
                if count >= reqpath_components.len() {
                    reqpath_components.clear();
                    count
                } else {
                    let new_len = reqpath_components.len() - count;
                    reqpath_components.truncate(new_len);
                    reqpath_components.first().map(|f| *f == "..").map(|b| {
                        if b {
                            reqpath_components.clear()
                        }
                    });
                    count
                }
            }
        });
    debug!("{}: {:?}", parent_count, reqpath_components);

    let mut components = (0..routes.len())
        .into_iter()
        .fold(Vec::with_capacity(routes.len()), |mut rs, idx| {
            if routes[idx].url_components.len() <= reqpath_components.len() {
                rs.push(&routes[idx]);
            }
            rs
        });

    #[allow(unknown_lints, needless_range_loop)]
    for idx in 0..reqpath_components.len() {
        let rpc = reqpath_components[idx];
        let mut cs_idx = components.len();
        while cs_idx > 0 {
            let cs_idx_tmp = cs_idx - 1;
            if components[cs_idx_tmp].url_components.len() > idx && components[cs_idx_tmp].url_components[idx] != rpc {
                components.remove(cs_idx_tmp);
            }
            cs_idx -= 1;
        }
    }
    for r in components.into_iter().rev() {
        let mut extract_path = reqpath_components[r.url_components.len()..]
            .iter()
            .fold(PathBuf::from(&r.path), |mut p, &c| {
                p.push(c);
                p
            });
        if extract_path.exists() {
            let mut path = reqpath_components
                .into_iter()
                .fold(String::with_capacity(req_path.len()), |mut p, c| {
                    p.push('/');
                    p.push_str(c);
                    p
                });
            if req_path.ends_with('/') {
                path.push('/');
            }
            return Some((path, extract_path, r.redirect_html));
        }
    }
    None
}

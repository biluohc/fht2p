use futures_cpupool::{Builder, CpuPool};
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpListener;
use futures::{future, Future, Stream};
use hyper::server::{Http, Request, Response, Service};
use hyper::{header, Error, Method, StatusCode};
use chrono::{DateTime, Local};

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

use tools::url_path_decode;
use args::{Config, Route};
use index::StaticIndex;
use router;
use consts;
use stat;

use std::time::Instant;
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

    stat::print(&addr, server.routes.as_slice());

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
        let (req_path, mut fspath, redirect_html) = match router::find(&self.routes, req_path) {
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
                    ExceptionHandler::call_async(
                        // symlink, socket, ...
                        Exception::Io(io::Error::from(io::ErrorKind::PermissionDenied)),
                        req,
                    )
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
        let timer = Instant::now();
        match req.remote_addr() {
            Some(addr) => {
                let mut info = (
                    addr,
                    req.method().clone(),
                    url_path_decode(req.uri().path())
                        .into_owned()
                        .to_owned(),
                );
                let query = req.query().map(|q| format!("?{}", q));
                let object = self.call2(&info.2, req);

                query.map(|q| info.2.push_str(&q));
                Box::new(object.inspect(move |res| {
                    let datatime: DateTime<Local> = Local::now();
                    let duration = timer.elapsed();
                    println!(
                        "[{} {}ms {}:{}] {} {} {}",
                        datatime.format("%Y-%m%d/%H:%M:%S"),
                        duration.as_secs() * 1000 + duration.subsec_nanos() as u64 / 1000_000,
                        info.0.ip(),
                        info.0.port(),
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

use futures_cpupool::{Builder, CpuPool};
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpListener;
use futures::{Future, Stream};
use hyper::server::{Http, Request, Response, Service};
use hyper::{Error, Method};
use url::percent_encoding::percent_decode;

use hyper_fs::{Exception, ExceptionHandlerServiceAsync};
use hyper_fs::{Config as FsConfig, FutureObject};

mod local {
    use hyper_fs::static_file::*;
    use exception::ExceptionHandler;
    static_file!(StaticFile, ExceptionHandler);
}
pub use self::local::StaticFile;
use exception::ExceptionHandler;

use index::StaticIndex;
use args::Config;
use consts;

#[allow(unused_imports)]
use std::io::{self, ErrorKind as IoErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::rc::Rc;

/// main function
pub fn run(config: Config) -> io::Result<()> {
    let pool = Builder::new().pool_size(3).name_prefix("hyper-fs").create();
    let fsconfig = FsConfig::new()
        .cache_secs(60)
        .follow_links(true)
        .show_index(true);

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
    let server = Rc::new(Server::new(handle.clone(), pool, config, fsconfig));
    let addr = tcp.local_addr()?;

    let http = Http::new();
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
        .for_each(|r| info.push_str(&format!("   {:?} -> {:?}\n", r.1, r.2)));
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
    routes: Vec<(Vec<String>, String, String)>,
    redirect_html: bool,
}

impl Server {
    pub fn new(handle: Handle, pool: CpuPool, config: Config, fsconfig: FsConfig) -> Self {
        let redirect_html = config.redirect_html;
        let mut routes = config
            .routes
            .into_iter()
            .map(|(u, p)| {
                (
                    u.split('/')
                        .filter(|c| !c.is_empty())
                        .map(|c| c.to_owned())
                        .collect::<Vec<_>>(),
                    u,
                    p,
                )
            })
            .collect::<Vec<_>>();
        routes.sort_by(|a, b| a.0.len().cmp(&b.0.len()));
        Self {
            handle: handle,
            pool: pool,
            fsconfig: Arc::new(fsconfig),
            routes: routes,
            redirect_html: redirect_html,
        }
    }
    pub fn router(&self, req_path: &str) -> Option<(String, PathBuf)> {
        let mut reqpath_components = req_path
            .split('/')
            .filter(|c| !c.is_empty() && c != &".")
            .collect::<Vec<_>>();
        (0..reqpath_components.len())
            .into_iter()
            .rev()
            .for_each(|idx| {
                if idx < reqpath_components.len() && reqpath_components[idx] == ".." {
                    reqpath_components.remove(idx);
                    if idx > 0 {
                        reqpath_components.remove(idx - 1);
                    }
                }
            });
        let mut components = (0..self.routes.len())
            .into_iter()
            .fold(Vec::with_capacity(self.routes.len()), |mut rs, idx| {
                if self.routes[idx].0.len() <= reqpath_components.len() {
                    rs.push(&self.routes[idx]);
                }
                rs
            });
        for idx in 0..reqpath_components.len() {
            let rpc = reqpath_components[idx];
            let mut cs_idx = components.len();
            while cs_idx > 0 {
                let cs_idx_tmp = cs_idx - 1;
                if components[cs_idx_tmp].0.len() > idx && components[cs_idx_tmp].0[idx] != rpc {
                    components.remove(cs_idx_tmp);
                }
                cs_idx -= 1;
            }
        }
        for r in components.into_iter().rev() {
            let extract_path = reqpath_components[r.0.len()..]
                .iter()
                .fold(PathBuf::from(&r.2), |mut p, &c| {
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
                return Some((path, extract_path));
            }
        }
        None
    }
    pub fn call2(&self, req_path: &str, req: Request) -> FutureObject {
        match *req.method() {
            Method::Head | Method::Get => {}
            _ => return ExceptionHandler::call_async(Exception::Method, req),
        }
        let (req_path, fspath) = match self.router(req_path) {
            Some(s) => s,
            None => return ExceptionHandler::call_async(Exception::not_found(), req),
        };
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
                    file_server.call(&self.pool, req)
                } else if md.is_dir() {
                    let mut index_server = StaticIndex::new(req_path, fspath, config.clone());
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
                let info = (
                    addr,
                    req.method().clone(),
                    percent_decode(req.path().as_bytes())
                        .decode_utf8()
                        .unwrap()
                        .into_owned()
                        .to_owned(),
                );
                let object = self.call2(&info.2, req);
                Box::new(object.inspect(move |res| {
                    println!(
                        "[{}:{}] {} {} {}",
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

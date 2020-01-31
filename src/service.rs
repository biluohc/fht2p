use futures::{
    future::{select, Either},
    Future, FutureExt,
};
use hyper::server::conn::Http;
use tokio::{
    runtime::{Builder, Runtime},
    signal::ctrl_c,
    task::{JoinHandle, LocalSet},
};

use crate::{
    base::{Router, Server},
    config::{Config, TlsAcceptor},
    how::Result,
    stat::stat_print,
};

pub struct State {
    tls: Option<TlsAcceptor>,
    config: Config,
    runtime: Runtime,
    router: Router,
    http: Http,
}

pub static mut STATE: Option<State> = None;

pub type GlobalState = &'static State;

impl State {
    pub fn new(config: Config) -> Result<Self> {
        let tls = config.load_cert()?;
        let mut http = Http::new();
        http.keep_alive(config.keep_alive);
        let router = Router::new(&config);
        let runtime = Builder::new()
            .threaded_scheduler()
            .core_threads(num_cpus::get() * 2 + 1)
            .thread_name("tok")
            .enable_all()
            .build()?;

        Ok(Self {
            config,
            runtime,
            router,
            tls,
            http,
        })
    }
    pub fn into_global(self) -> GlobalState {
        unsafe {
            STATE = Some(self);
            assert_eq!(STATE.as_ref().unwrap().config, Self::global().config);
            STATE.as_ref().unwrap()
        }
    }
    pub fn global() -> GlobalState {
        unsafe { STATE.as_ref().expect("Global State is None") }
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn tls(&self) -> &Option<TlsAcceptor> {
        &self.tls
    }
    pub fn http(&self) -> &Http {
        &self.http
    }
    pub fn router(&self) -> &Router {
        &self.router
    }
}

pub fn run(config: Config) -> Result<()> {
    stat_print(&config.addr, config.cert.is_some(), config.routes.values());

    let state = State::new(config)?.into_global();
    let mut rt = Builder::new().basic_scheduler().enable_all().build()?;
    let local = LocalSet::new();

    local.block_on(&mut rt, async move {
        let http = Server::run(state);
        let ctrlc = ctrl_c();

        match select(http.boxed(), ctrlc.boxed()).await {
            Either::Left((http, _)) => {
                error!("http listen failed: {:?}", http);
            }
            Either::Right((ctrlc, _)) => {
                warn!("ctrlc catched: {:?}, will exit", ctrlc);
            }
        }

        Ok(())
    })
}

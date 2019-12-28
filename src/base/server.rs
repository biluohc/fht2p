use futures::{future, FutureExt};
use tokio::{
    net::{TcpListener, TcpStream},
    time::{delay_for, timeout},
};

use std::{io, net::SocketAddr, time::Duration};

use crate::{base::Service, service::GlobalState, Error, Result};

pub struct Server;

impl Server {
    pub async fn run(state: GlobalState) -> Result<()> {
        let mut tcp = TcpListener::bind(state.config().addr).await?;

        loop {
            match tcp.accept().await.and_then(|(s, sa)| s.set_nodelay(true).map(|_| (s, sa))) {
                Ok((socket, addr)) => {
                    state.spawn(serve_socket(socket, addr, state).then(move |rest| {
                        if let Err(e) = rest {
                            error!("socket {}: {}", addr, e.description());
                        }
                        future::ready(())
                    }));
                }
                Err(e) => {
                    let is_server_error = is_server_error(&e);
                    error!("http's tcp-listener error(is_server_error: {:5}): {:?}", is_server_error, e);

                    if is_server_error {
                        delay_for(Duration::from_millis(10)).await;
                        error!("http's tcp-listener error, sleep 10 ms ok");
                    }
                }
            }
        }
    }
}

pub async fn serve_socket(socket: TcpStream, addr: SocketAddr, state: GlobalState) -> Result<()> {
    let service = Service::new(addr, state);

    if let Some(tls) = state.tls() {
        let socket = timeout(Duration::from_secs(10), tls.accept(socket))
            .await
            .map_err(|_| format_err!("tls handshake timeout"))
            .and_then(|res| res.map_err(Error::from))?;
        state.http().serve_connection(socket, service).await?;
    } else {
        state.http().serve_connection(socket, service).await?;
    }

    Ok(())
}

pub fn is_server_error(e: &io::Error) -> bool {
    if let Some(c) = e.raw_os_error() {
        match c {
            // 11: Resource temporarily unavailable
            11 |
            // 23: Too many open files in system
            23 |
            // 24: Too many open files
            24 => return true,
            _=> {}
        }
    }
    false
}

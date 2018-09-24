use bytesize::ByteSize;
use failure::Error;
use futures::{future, Future, Sink, Stream};
use tokio::codec::Decoder;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub mod auth;
pub mod http;
pub mod proxy;

use self::auth::Auth;
use self::http::Http;
use self::proxy::BytesCodec;

pub fn process_socket<S>(addr: SocketAddr, socket: S) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    S: AsyncRead + AsyncWrite + Send + 'static,
{
    let addr_clone = addr.clone();
    let (w, r) = Http.framed(socket).split();

    let socket_auth_remote = r
        .into_future()
        .map_err(|(e, _rh)| Error::from(e))
        .and_then(|(req, rh)| req.ok_or(format_err!("read or parse Request error")).map(|req| (req, rh)))
        .map_err(move |e| error!("handle connect method for {} failed: {:?}", addr_clone, e))
        .and_then(move |(req, rh)| {
            info!("{:?}", req);
            auth::handle(addr, req).map_err(|_| ()).and_then(|(ok, resp)| {
                future::result(ok.ok_or(format_err!("Auth failed")))
                    .and_then(|auth| {
                        info!("{:?}", auth);
                        let uri = auth.uri.to_string();
                        // todo: handle dns async and safely
                        uri.to_socket_addrs().map_err(Error::from).and_then(|mut iter| {
                            iter.nth(0)
                                .ok_or(format_err!("resolvered DNS failed"))
                                .map(|sokcet_addr| (auth, sokcet_addr))
                        })
                    }).and_then(|(auth, socket_addr)| {
                        let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(3);
                        Retry::spawn(retry_strategy, move || TcpStream::connect(&socket_addr))
                            .map_err(Error::from)
                            .map(|socket| (auth, socket))
                    }).then(move |rest| match rest {
                        Ok((auth, remote_socket)) => {
                            let send = w
                                .send(resp)
                                .map_err(|e| error!("send resp for connect method failed: {:?}", e))
                                .and_then(move |wh| {
                                    rh.reunite(wh)
                                        .map(move |socket| (socket.into_inner(), auth, remote_socket))
                                        .map_err(|e| error!("merge socket failed: {:?}", e))
                                });
                            Box::new(send) as Box<Future<Item = (S, Auth, TcpStream), Error = ()> + Send + 'static>
                        }
                        Err(e) => {
                            error!("handle connect method failed: {:?}", e);

                            let _ = w.send(resp).map_err(|e| {
                                error!("send resp for connect method failed: {:?}", e);
                            });

                            Box::new(future::err(())) as _
                        }
                    })
            })
        });

    socket_auth_remote.and_then(|(socket, auth, remote)| {
        let up_count = Arc::from(AtomicUsize::new(0));
        let down_count = Arc::from(AtomicUsize::new(0));
        let (up_count_clone, down_count_clone) = (up_count.clone(), down_count.clone());

        let (cw, cr) = BytesCodec::new(up_count_clone).framed(socket).split();
        let (rw, rr) = BytesCodec::new(down_count_clone).framed(remote).split();

        let up = cr.fold(rw, |rw, bytes| rw.send(bytes.freeze())).map(|_| ());

        let down = rr.fold(cw, |cw, bytes| cw.send(bytes.freeze())).map(|_| ());

        up.select(down).then(move |rest| {
            let up = up_count.load(Ordering::SeqCst);
            let down = down_count.load(Ordering::SeqCst);
            let all = up + down;

            let (all, up, down) = (ByteSize::b(all as _), ByteSize::b(up as _), ByteSize::b(down as _));

            match rest {
                Ok(_) => {
                    info!(
                        "transfer for {} with {} success: all={:8}, up={:8}, down={:8}",
                        auth.addr, auth.uri, all, up, down
                    );
                }
                Err((e, _)) => {
                    error!(
                        "transfer for {} with {} failed:  all={:8}, up={:8}, down={:8}, error={:?}",
                        auth.addr, auth.uri, all, up, down, e
                    );
                }
            };
            future::ok(())
        })
    })
}

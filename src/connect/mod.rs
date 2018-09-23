use bytesize::ByteSize;
use failure::Error;
use futures::{future, Future, Sink, Stream};
use tokio::codec::Decoder;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

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
        .and_then(|(req, rh)| req.ok_or(format_err!("read or parse Request failed")).map(|req| (req, rh)))
        .and_then(move |(req, rh)| {
            info!("{:?}", req);
            auth::handle(addr, req).and_then(|(ok, resp)| {
                if let Some(auth) = ok {
                    info!("{:?}", auth);
                    let uri = auth.uri.to_string();
                    // todo: handle dns async and safely
                    let socket_addr = uri.to_socket_addrs().unwrap().nth(0).unwrap();

                    Box::new(
                        TcpStream::connect(&socket_addr)
                            .and_then(|remote| w.send(resp).map(move |wh| (wh, remote)))
                            .map_err(|e| Error::from(e))
                            .and_then(|(wh, remote)| {
                                rh.reunite(wh)
                                    .map(move |socket| (socket.into_inner(), auth, remote))
                                    .map_err(|e| format_err!("merge socket failed: {:?}", e))
                            }),
                    ) as Box<Future<Item = (S, Auth, TcpStream), Error = Error> + Send + 'static>
                } else {
                    Box::new(future::err(format_err!("Auth failed"))) as _
                }
            })
        }).map_err(move |e| error!("handle connect request for {} failed: {:?}", addr_clone, e));

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

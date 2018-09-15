#[cfg(unix)]
use net2::unix::UnixTcpBuilderExt;
use net2::TcpBuilder;
use tokio::net::TcpListener;
use tokio::reactor;

use std::io::Result;
use std::net::SocketAddr;

pub fn reuse_address(addr: &SocketAddr) -> Result<TcpListener> {
    let builder = if addr.is_ipv4() { TcpBuilder::new_v4() } else { TcpBuilder::new_v6() }?;

    // reuse have to before bind
    builder.reuse_port(true)?;
    builder.reuse_address(true)?;
    builder.bind(addr)?;
    // 半连接队列长度
    builder.listen(128).and_then(|b| TcpListener::from_std(b, &reactor::Handle::current()))
}

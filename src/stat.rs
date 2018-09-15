use systemstat::data::IpAddr as StatIpAddr;
use systemstat::{Platform, System};

use config::Route;
use consts;

use std::io;
use std::net::{IpAddr, SocketAddr};

const TIP: &str = "You can visit:";

pub fn print(addr: &SocketAddr, proto: &str, routes: Vec<Route>) {
    println!(
        "{}/{} Serving at {}:{} for:",
        consts::NAME,
        env!("CARGO_PKG_VERSION"),
        addr.ip(),
        addr.port()
    );

    routes.iter().for_each(|r| println!("   {:?} -> {:?}", r.url, r.path));

    println!("{}", TIP);

    print_addrs(addr)
        .map_err(|e| error!("print_addrs faield: {:?}", e))
        .unwrap_or_else(|_| println!("{}{}://{}:{}", " ".repeat(TIP.len()), proto, addr.ip(), addr.port()))
}

fn print_addrs(addr: &SocketAddr) -> io::Result<()> {
    let netifs = System::new().networks()?;

    let mut adrs = netifs
        .values()
        .flat_map(|netif| {
            trace!("{}: {:?}", netif.name, netif.addrs);
            netif.addrs.iter().filter_map(|a| match a.addr {
                StatIpAddr::V4(ipv4) if addr.is_ipv4() => Some(IpAddr::V4(ipv4)),
                StatIpAddr::V6(ipv6) if addr.is_ipv6() => Some(IpAddr::V6(ipv6)),
                _ => None,
            })
        }).collect::<Vec<_>>();

    adrs.sort();
    adrs.iter()
        .for_each(|adr| println!("{}http://{}:{}", " ".repeat(TIP.len()), adr, addr.port()));
    Ok(())
}

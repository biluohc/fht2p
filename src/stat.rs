use systemstat::data::IpAddr as StatIpAddr;
use systemstat::{Platform, System};

use crate::config::Route;
use crate::consts;

use std::cmp::Ordering::*;
use std::io;
use std::net::{IpAddr, SocketAddr};

const TIP: &str = "You can visit:";

pub fn stat_print<'a>(addr: &SocketAddr, tls: bool, routes: impl Iterator<Item = &'a Route>) {
    println!(
        "{}/{} Serving at {}:{} for:",
        consts::NAME,
        env!("CARGO_PKG_VERSION"),
        addr.ip(),
        addr.port()
    );

    routes.for_each(|r| println!("   {:?} -> {:?}", r.url, r.path));

    println!("{}", TIP);

    let proto = if tls { "https" } else { "http" };
    print_addrs(addr, proto)
        .map_err(|e| error!("print_addrs faield: {:?}", e))
        .unwrap_or_else(|_| println!("{}{}://{}:{}", " ".repeat(TIP.len()), proto, addr.ip(), addr.port()))
}

fn print_addrs(addr: &SocketAddr, proto: &str) -> io::Result<()> {
    let netifs = System::new().networks()?;

    let mut adrs = netifs
        .values()
        .flat_map(|netif| {
            trace!("{}: {:?}", netif.name, netif.addrs);
            netif
                .addrs
                .iter()
                .filter_map(|a| match a.addr {
                    StatIpAddr::V4(ipv4) if addr.is_ipv4() => Some(ipv4.into()),
                    StatIpAddr::V6(ipv6) if addr.is_ipv6() => Some(ipv6.into()),
                    _ => None,
                })
                .filter(|ip: &IpAddr| {
                    let addr_ip = addr.ip();

                    if addr_ip.is_unspecified() {
                        true
                    } else if addr_ip.is_loopback() {
                        ip.is_loopback()
                    } else {
                        *ip == addr_ip
                    }
                })
        })
        .collect::<Vec<_>>();

    adrs.sort_by(|a, b| match (a.is_loopback(), b.is_loopback()) {
        (true, true) => Equal,
        (true, false) => Less,
        (false, true) => Greater,
        _ => a.cmp(b),
    });

    adrs.iter().for_each(|adr| {
        // curl  http://::1:9000
        // curl: (3) IPv6 numerical address used in URL without brackets
        if adr.is_ipv6() {
            println!("{}{}://[{}]:{}", " ".repeat(TIP.len()), proto, adr, addr.port())
        } else {
            println!("{}{}://{}:{}", " ".repeat(TIP.len()), proto, adr, addr.port())
        }
    });

    Ok(())
}

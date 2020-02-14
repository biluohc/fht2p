use systemstat::data::IpAddr as StatIpAddr;
use systemstat::{Platform, System};

use crate::config::Route;
use crate::consts;

use std::cmp::Ordering::*;
use std::io;
use std::net::{IpAddr, SocketAddr};

const TIP: &str = "You can visit:";

pub fn stat_print<'a>(addr: &SocketAddr, tls: bool, routes: impl Iterator<Item = &'a Route>, show_qrcode: bool) {
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
    print_addrs(addr, proto, show_qrcode)
        .map_err(|e| error!("print_addrs faield: {:?}", e))
        .unwrap_or_else(|_| print_addr(&addr.ip(), addr.port(), proto, show_qrcode))
}

fn print_addr(adr: &IpAddr, port: u16, proto: &str, show_qrcode: bool) {
    // curl  http://::1:9000
    // curl: (3) IPv6 numerical address used in URL without brackets
    let adr_str = if adr.is_ipv6() {
        format!("{}://[{}]:{}", proto, adr, port)
    } else {
        format!("{}://{}:{}", proto, adr, port)
    };

    println!("{}{}", " ".repeat(TIP.len()), adr_str);

    if show_qrcode {
        qr2term::print_qr(&adr_str)
            .map(|_| println!(""))
            .map_err(|e| error!("print qr code failed: {}", e))
            .ok();
    }
}

fn print_addrs(addr: &SocketAddr, proto: &str, show_qrcode: bool) -> io::Result<()> {
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

    adrs.iter().for_each(|adr| print_addr(adr, addr.port(), proto, show_qrcode));

    Ok(())
}

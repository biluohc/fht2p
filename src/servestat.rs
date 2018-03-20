use systemstat::{Platform, System};
use systemstat::data::IpAddr as StatIpAddr;

use args::Route;
use consts;

use std::net::{IpAddr, SocketAddr};
use std::io;

const TIP: &str = "You can visit:";

pub fn print(addr: &SocketAddr, routes: &[Route]) {
    println!(
        "{}/{} Serving at {}:{} for:",
        consts::NAME,
        env!("CARGO_PKG_VERSION"),
        addr.ip(),
        addr.port()
    );

    routes
        .iter()
        .for_each(|r| println!("   {:?} -> {:?}", r.url, r.path));

    println!("{}", TIP);

    print_addrs(&addr).unwrap_or_else(|e| {
        // systemstat is unsupported NetworkInterface on windows and opendsb now.
        if !cfg!(windows) && !cfg!(openbsd) {
            error!("Networks: {:?}", e)
        }
        println!(
            "{}http://{}:{}",
            " ".repeat(TIP.len()),
            addr.ip(),
            addr.port()
        )
    })
}
fn print_addrs(addr: &SocketAddr) -> io::Result<()> {
    let netifs = System::new().networks()?;

    let mut adrs = netifs
        .values()
        .flat_map(|netif| {
            trace!("{}: {:?}", netif.name, netif.addrs);
            netif.addrs.iter().filter_map(|a| match a.addr {
                StatIpAddr::V4(ipv4) => if addr.is_ipv4() && addr.ip() != ipv4 {
                    Some(IpAddr::V4(ipv4))
                } else {
                    None
                },
                StatIpAddr::V6(ipv6) => if addr.is_ipv6() && addr.ip() != ipv6 {
                    Some(IpAddr::V6(ipv6))
                } else {
                    None
                },
                StatIpAddr::Empty | StatIpAddr::Unsupported => None,
            })
        })
        .collect::<Vec<_>>();

    adrs.sort();
    adrs.iter()
        .for_each(|adr| println!("{}http://{}:{}", " ".repeat(TIP.len()), adr, addr.port()));
    Ok(())
    // Err(io::Error::new(io::ErrorKind::Other, "Not windows or openbsd!"))
}

#[macro_use]
pub extern crate nonblock_logger;
pub extern crate fht2plib;

use fht2plib::{args, process_exit, service::Service};

fn main() {
    let (config, mut handle) = args::parse();

    trace!("{:?}\n", &config);

    if let Err(e) = config.startup() {
        error!("{}", e);
        handle.join();
        process_exit(1)
    }
}

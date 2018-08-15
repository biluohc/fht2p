use chrono;
use fern;
use log::LevelFilter;

use fern::colors::{Color, ColoredLevelConfig};

use std;

pub fn set(lvl: LevelFilter) -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::White)
        .trace(Color::White);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {:5}#{}:{}->{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.module_path().unwrap_or("<none>"),
                // record.file().unwrap_or("<none>"),
                record.line().unwrap_or(0),
                current_thread_name(),
                message
            ))
        }).level(lvl)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn current_thread_name() -> &'static str {
    thread_local!(static TNAME: String = std::thread::current()
        .name()
        .map(|s| s.to_owned())
        .unwrap_or("<none>".to_owned()));
    TNAME.with(|tname| unsafe { std::mem::transmute::<&str, &'static str>(tname.as_str()) })
}

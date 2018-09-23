use chrono;
use fern;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::{io, mem, thread};

pub fn set(warn0_info1_debug2_trace3: u64) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match warn0_info1_debug2_trace3 {
        0 => base_config.level(LevelFilter::Warn),
        1 => base_config.level(LevelFilter::Info),
        2 => base_config.level(LevelFilter::Debug),
        _3_or_more => base_config.level(LevelFilter::Trace),
    };

    let filter_targets = vec![
        "mio",
        "tokio_reactor",
        "tokio_core",
        "tokio",
        "tokio_threadpool",
        // "hyper",
        "want",
        "tokio_io",
    ];
    // 开发阶段通过日志多熟悉 tokio*
    // let filter_targets: Vec<&str> = vec![];

    for target in filter_targets {
        base_config = base_config.level_for(target, LevelFilter::Info);
    }

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::White)
        .trace(Color::White);

    base_config
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {:5}#{}:{}.{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.module_path().unwrap_or("*"),
                // record.file().unwrap_or("*"),
                record.line().unwrap_or(0),
                current_thread_name(),
                message
            ))
        }).chain(io::stdout())
        .apply()?;
    Ok(())
}

fn current_thread_name() -> &'static str {
    thread_local!(static TNAME: String = thread::current()
        .name()
        .map(|s| format!("{}.{}", thread_id(), s))
        .unwrap_or_else(||format!("{}.<uname-{:2}>", thread_id(), uname_count())));
    TNAME.with(|tname| unsafe { mem::transmute::<&str, &'static str>(tname.as_str()) })
}

// https://doc.rust-lang.org/nightly/src/std/thread/mod.rs.html#938
// pub struct ThreadId(u64);
fn thread_id() -> u64 {
    unsafe { mem::transmute::<thread::ThreadId, _>(thread::current().id()) }
}

fn uname_count() -> usize {
    static COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
    COUNT.fetch_add(1, Ordering::SeqCst)
}

pub use nonblock_logger::{current_thread_name, JoinHandle};
use nonblock_logger::{
    log::{log_enabled, Level::Info, LevelFilter, Record},
    BaseFilter, BaseFormater, FixedLevel, NonblockLogger,
};

pub fn format(base: &BaseFormater, record: &Record) -> String {
    let level = FixedLevel::with_color(record.level(), base.color_get())
        .length(base.level_get())
        .into_colored()
        .into_coloredfg();

    format!(
        "{} {} [{} {}:{}] {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
        level,
        current_thread_name(),
        record.file().unwrap_or("*"),
        record.line().unwrap_or(0),
        record.args()
    )
}

pub fn log_enabled_info(target: &str) -> bool {
    log_enabled!(target: target, Info)
}

pub fn logger_init(verbose: u64) -> JoinHandle {
    let pkg = crate::consts::NAME;
    let log = match verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _more => LevelFilter::Trace,
    };

    if verbose > 2 {
        println!("logger_init: pkg: {}, level: {:?}", pkg, log)
    };

    let formater = BaseFormater::new().local(true).color(true).level(4).formater(format);
    let filter = BaseFilter::new()
        .max_level(log)
        .starts_with(true)
        .notfound(true)
        .chain(pkg, log)
        .chain("tokio", LevelFilter::Info)
        .chain("hyper", LevelFilter::Info)
        .chain("mio", LevelFilter::Info);

    NonblockLogger::new()
        .formater(formater)
        .filter(filter)
        .expect("add filiter failed")
        .log_to_stdout()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap()
}

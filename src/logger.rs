use nonblock_logger::{
    current_thread_name,
    log::{LevelFilter, Record},
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

pub fn fun<F>(f: F)
where
    F: FnOnce(),
{
    let pkg = env!("CARGO_PKG_NAME");
    let log = LevelFilter::Debug;
    println!("{}: {:?}", pkg, log);

    let formater = BaseFormater::new().local(true).color(true).level(4).formater(format);
    let filter = BaseFilter::new()
        .max_level(log)
        .starts_with(true)
        .notfound(true)
        .chain(pkg, log)
        .chain("tokio", LevelFilter::Info)
        .chain("hyper", LevelFilter::Info)
        .chain("mio", LevelFilter::Info);

    let _handle = NonblockLogger::new()
        .formater(formater)
        .filter(filter)
        .expect("add filiter failed")
        .log_to_stdout()
        .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
        .unwrap();
    f()
}

use log::{max_level, Level, Metadata, Record};

/// 初始化日志系统
pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIX-ME: Configure the logger
    // log::set_max_level(log::LevelFilter::Info);
    log::set_max_level(log::LevelFilter::Info);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= max_level()
    }


    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // FIX-ME: Implement the logger with serial output
        let level_str = match record.level() {
            Level::Error => "ERROR",
            Level::Warn => "WARN ",
            Level::Info => "INFO ",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        };
        
        let module = record.module_path().unwrap_or("unknown");
        println!(
            "[{}] {}: {}",
            level_str,
            module,
            record.args()
        );
    }


    fn flush(&self) {}
}

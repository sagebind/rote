use log;
use std::io;
use std::io::Write;

pub use log::LogLevelFilter as Filter;


struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= log::LogLevel::Info
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            if record.level() <= log::LogLevel::Warn {
                writeln!(&mut io::stderr(), "rote[{}]: {}", record.level(), record.args())
                    .expect("Unable to write to stderr!");
            } else {
                println!("rote[{}]: {}", record.level(), record.args());
            }
        }
    }
}

pub fn init(level: log::LogLevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(level);
        Box::new(Logger)
    })
}

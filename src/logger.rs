use log::*;
use term;

pub use log::LogLevelFilter as Filter;


/// Writes log messages to standard error.
///
/// The enabled filter level can be customized by passing in a specific filter.
struct Logger(LogLevelFilter);

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mut err = term::stderr().expect("failed to open stderr");

            // Print with colors matching the level.
            match record.level() {
                LogLevel::Error => {
                    err.attr(term::Attr::Bold).ok();
                    err.fg(term::color::BRIGHT_RED).ok();
                    write!(err, "error: ").unwrap();

                    err.fg(term::color::BRIGHT_WHITE).ok();
                    writeln!(err, "{}", record.args()).unwrap();
                    err.reset().ok();
                }
                LogLevel::Warn => {
                    err.fg(term::color::BRIGHT_YELLOW).ok();
                    write!(err, "warn: ").unwrap();
                    err.reset().ok();

                    err.attr(term::Attr::Bold).ok();
                    err.fg(term::color::BRIGHT_WHITE).ok();
                    writeln!(err, "{}", record.args()).unwrap();
                    err.reset().ok();
                }
                LogLevel::Info => {
                    err.fg(term::color::BRIGHT_GREEN).ok();
                    write!(err, "info: ").unwrap();

                    err.fg(term::color::BRIGHT_WHITE).ok();
                    writeln!(err, "{}", record.args()).unwrap();
                    err.reset().ok();
                }
                LogLevel::Debug => {
                    err.fg(term::color::BRIGHT_BLUE).ok();
                    write!(err, "debug: ").unwrap();
                    err.reset().ok();

                    err.fg(term::color::BRIGHT_WHITE).ok();
                    writeln!(err, "{}", record.args()).unwrap();
                    err.reset().ok();
                }
                LogLevel::Trace => {
                    err.fg(term::color::BRIGHT_WHITE).ok();
                    writeln!(err, "trace: {}", record.args()).unwrap();
                    err.reset().ok();
                }
            }

            err.flush().unwrap();
        }
    }
}

/// Initializes the global logger with a given level filter.
pub fn init(level: LogLevelFilter) -> Result<(), SetLoggerError> {
    set_logger(|max_log_level| {
        max_log_level.set(level);
        Box::new(Logger(level))
    })
}

use log::*;
use term;

pub use log::LogLevelFilter as Filter;


/// Writes log messages to standard output and standard error.
///
/// The enabled filter level can be customized by passing in a specific filter.
struct Logger(LogLevelFilter);

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            // Print with colors matching the level.
            match record.level() {
                LogLevel::Error => {
                    let mut t = term::stderr().unwrap();

                    t.attr(term::Attr::Bold).unwrap();
                    t.fg(term::color::BRIGHT_RED).unwrap();
                    write!(t, "error: ").unwrap();
                    t.fg(term::color::BRIGHT_WHITE).unwrap();
                    writeln!(t, "{}", record.args()).unwrap();
                    t.reset().unwrap();
                }
                LogLevel::Warn => {
                    let mut t = term::stderr().unwrap();

                    t.attr(term::Attr::Bold).unwrap();
                    t.fg(term::color::BRIGHT_YELLOW).unwrap();
                    write!(t, "warn: ").unwrap();
                    t.fg(term::color::BRIGHT_WHITE).unwrap();
                    writeln!(t, "{}", record.args()).unwrap();
                    t.reset().unwrap();
                }
                LogLevel::Info => {
                    let mut t = term::stdout().unwrap();

                    t.attr(term::Attr::Bold).unwrap();
                    t.fg(term::color::BRIGHT_GREEN).unwrap();
                    write!(t, "info: ").unwrap();
                    t.fg(term::color::BRIGHT_WHITE).unwrap();
                    writeln!(t, "{}", record.args()).unwrap();
                    t.reset().unwrap();
                }
                LogLevel::Debug => {
                    let mut t = term::stdout().unwrap();

                    t.fg(term::color::BRIGHT_BLUE).unwrap();
                    write!(t, "debug: ").unwrap();
                    t.reset().unwrap();
                    writeln!(t, "{}", record.args()).unwrap();
                }
                LogLevel::Trace => {
                    let mut t = term::stdout().unwrap();

                    t.attr(term::Attr::Bold).unwrap();
                    t.fg(term::color::BRIGHT_WHITE).unwrap();
                    write!(t, "trace: ").unwrap();
                    t.reset().unwrap();
                    writeln!(t, "{}", record.args()).unwrap();
                }
            }
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

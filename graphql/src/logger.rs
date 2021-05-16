use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde::Deserialize;

/// A stdio logger.
pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

impl Logger {
    /// Initialize logger.
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        log::set_max_level(level);
        log::set_logger(&Logger)
    }
}

#[derive(Debug, Deserialize)]
#[serde(remote = "log::LevelFilter", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogLevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

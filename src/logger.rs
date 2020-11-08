use anyhow::{anyhow, Result};
use log::{Record, Level, Metadata, LevelFilter};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

// Init logger and set max log level
pub fn init_log() -> Result<()> {
    let logger = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug));

    match logger {
            Ok(r) => Ok(r),
            Err(e) => Err(anyhow!("Cannot init logger: {}", e)),
        }
}
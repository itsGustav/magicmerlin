//! Structured logging setup and helpers for MagicMerlin services.

mod error;
mod level;
mod rotate;

use std::path::{Path, PathBuf};

pub use error::LoggingError;
pub use level::LogLevel;
use rotate::RotatingMakeWriter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

/// Settings for logging initialization.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Minimum accepted log level.
    pub level: LogLevel,
    /// Enables ANSI color output for console logs.
    pub color: bool,
    /// Directory where `gateway.log` and `gateway.err.log` are created.
    pub log_dir: Option<PathBuf>,
    /// Maximum size of a single log file before rotation.
    pub rotate_size_bytes: u64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            color: true,
            log_dir: None,
            rotate_size_bytes: 10 * 1024 * 1024,
        }
    }
}

/// Initializes global structured logging with console and optional file sinks.
pub fn init(config: LoggingConfig) -> Result<(), LoggingError> {
    let level_filter = config.level.as_level_filter();
    let console_layer = fmt::layer()
        .with_ansi(config.color)
        .with_target(true)
        .with_filter(level_filter);

    if let Some(log_dir) = config.log_dir {
        let logs =
            RotatingMakeWriter::new(log_dir.join("gateway.log"), config.rotate_size_bytes, 5)?;
        let errs =
            RotatingMakeWriter::new(log_dir.join("gateway.err.log"), config.rotate_size_bytes, 5)?;

        tracing_subscriber::registry()
            .with(console_layer)
            .with(
                fmt::layer()
                    .with_ansi(false)
                    .with_target(true)
                    .with_writer(logs)
                    .with_filter(level_filter),
            )
            .with(
                fmt::layer()
                    .with_ansi(false)
                    .with_target(true)
                    .with_writer(errs)
                    .with_filter(LevelFilter::ERROR),
            )
            .try_init()
            .map_err(LoggingError::SetGlobalDefault)?;
    } else {
        tracing_subscriber::registry()
            .with(console_layer)
            .try_init()
            .map_err(LoggingError::SetGlobalDefault)?;
    }

    Ok(())
}

/// Initializes logging with `level`, `color`, and optional log directory.
pub fn init_with(level: LogLevel, color: bool, log_dir: Option<&Path>) -> Result<(), LoggingError> {
    let config = LoggingConfig {
        level,
        color,
        log_dir: log_dir.map(Path::to_path_buf),
        ..LoggingConfig::default()
    };
    init(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_parse_variants() {
        assert_eq!(
            "silent".parse::<LogLevel>().expect("parse"),
            LogLevel::Silent
        );
        assert_eq!("fatal".parse::<LogLevel>().expect("parse"), LogLevel::Fatal);
        assert!("nope".parse::<LogLevel>().is_err());
    }
}

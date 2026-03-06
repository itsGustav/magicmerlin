//! Log level parsing and conversion helpers.

use std::str::FromStr;

use tracing_subscriber::filter::LevelFilter;

/// User-configurable logging levels for MagicMerlin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Disable all logging.
    Silent,
    /// Fatal-only logging (mapped to ERROR in tracing).
    Fatal,
    /// Error and above.
    Error,
    /// Warning and above.
    Warn,
    /// Informational and above.
    Info,
    /// Debug and above.
    Debug,
    /// Trace and above.
    Trace,
}

impl LogLevel {
    /// Converts this level into a `tracing_subscriber` level filter.
    pub fn as_level_filter(self) -> LevelFilter {
        match self {
            Self::Silent => LevelFilter::OFF,
            Self::Fatal => LevelFilter::ERROR,
            Self::Error => LevelFilter::ERROR,
            Self::Warn => LevelFilter::WARN,
            Self::Info => LevelFilter::INFO,
            Self::Debug => LevelFilter::DEBUG,
            Self::Trace => LevelFilter::TRACE,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl FromStr for LogLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("silent") {
            return Ok(Self::Silent);
        }
        if s.eq_ignore_ascii_case("fatal") {
            return Ok(Self::Fatal);
        }
        if s.eq_ignore_ascii_case("error") {
            return Ok(Self::Error);
        }
        if s.eq_ignore_ascii_case("warn") || s.eq_ignore_ascii_case("warning") {
            return Ok(Self::Warn);
        }
        if s.eq_ignore_ascii_case("info") {
            return Ok(Self::Info);
        }
        if s.eq_ignore_ascii_case("debug") {
            return Ok(Self::Debug);
        }
        if s.eq_ignore_ascii_case("trace") {
            return Ok(Self::Trace);
        }

        Err("invalid log level")
    }
}

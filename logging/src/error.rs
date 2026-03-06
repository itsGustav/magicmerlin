//! Error types for logging initialization and file output.

use std::path::PathBuf;

/// Errors produced by the logging crate.
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    /// Creating log directories or files failed.
    #[error("failed to create log file at {path}: {source}")]
    CreateLogFile {
        /// Path of file that failed to create.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// A filesystem operation for rotation failed.
    #[error("failed to rotate logs for {path}: {source}")]
    Rotate {
        /// Base path of rotated log file.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Failed to set global tracing subscriber.
    #[error("failed to set global tracing subscriber: {0}")]
    SetGlobalDefault(#[source] tracing_subscriber::util::TryInitError),
}

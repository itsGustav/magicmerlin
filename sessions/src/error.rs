use std::path::PathBuf;

use thiserror::Error;

/// Result type for sessions crate operations.
pub type Result<T> = std::result::Result<T, SessionsError>;

/// Error type for sessions lifecycle and persistence operations.
#[derive(Debug, Error)]
pub enum SessionsError {
    /// Wrapper for storage crate errors.
    #[error(transparent)]
    Storage(#[from] magicmerlin_storage::StorageError),

    /// Wrapper for sqlite errors.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    /// Missing session in metadata store.
    #[error("session not found: {0}")]
    MissingSession(String),

    /// IO failure while working with paths.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Source io error.
        source: std::io::Error,
    },
}

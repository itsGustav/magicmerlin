//! Error types for storage operations.

use std::path::PathBuf;

/// Error returned by storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Generic filesystem I/O failure.
    #[error("io error at {path}: {source}")]
    Io {
        /// File path involved in the operation.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// JSON parse or serialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// SQL operation failure.
    #[error("sqlite error: {0}")]
    Sql(#[from] rusqlite::Error),

    /// Connection pool failure.
    #[error("pool error: {0}")]
    Pool(#[from] r2d2::Error),

    /// Lock acquisition timed out.
    #[error("timed out acquiring lock for {0}")]
    LockTimeout(PathBuf),

    /// Invalid lock file contents.
    #[error("invalid lock file: {0}")]
    InvalidLock(String),
}

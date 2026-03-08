//! Error types for agent runtime.

use std::path::PathBuf;
use std::time::Duration;

/// Result alias for agent runtime operations.
pub type Result<T> = std::result::Result<T, AgentError>;

/// Errors raised by agent runtime components.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// I/O error while reading/writing workspace/state files.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path involved.
        path: PathBuf,
        /// Underlying source error.
        source: std::io::Error,
    },
    /// Storage-layer error.
    #[error("storage error: {0}")]
    Storage(#[from] magicmerlin_storage::StorageError),
    /// Provider-layer error.
    #[error("provider error: {0}")]
    Provider(#[from] magicmerlin_providers::ProviderError),
    /// JSON parse error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// Generic invariant violation.
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// Operation exceeded timeout.
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
    /// Operation was cancelled.
    #[error("operation cancelled: {0}")]
    Cancelled(String),
    /// External command error.
    #[error("tool execution failed: {0}")]
    ToolExecution(String),
}

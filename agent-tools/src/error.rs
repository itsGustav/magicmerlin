//! Error types for agent tool execution.

use std::path::PathBuf;

/// Result alias for tool operations.
pub type Result<T> = std::result::Result<T, ToolError>;

/// Errors raised by tool execution and validation.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Tool name is missing from registry.
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    /// Parameters do not satisfy expected shape.
    #[error("invalid params for {tool}: {message}")]
    InvalidParams {
        /// Tool name.
        tool: String,
        /// Validation details.
        message: String,
    },
    /// I/O failure.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path for I/O operation.
        path: PathBuf,
        /// Underlying source error.
        source: std::io::Error,
    },
    /// JSON parsing/serialization failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Process management failure.
    #[error("process error: {0}")]
    Process(String),
    /// Tool execution denied by policy.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// Storage interaction failure.
    #[error("storage error: {0}")]
    Storage(#[from] magicmerlin_storage::StorageError),
    /// Generic runtime failure.
    #[error("execution failed: {0}")]
    Execution(String),
}

//! Error types for infrastructure utility helpers.

/// Errors returned by infra helpers.
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    /// Wrapper over HTTP transport errors.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON encoding or decoding failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Base64 decode failed.
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Invalid timezone offset representation.
    #[error("invalid timezone offset: {0}")]
    InvalidTimezoneOffset(String),
}

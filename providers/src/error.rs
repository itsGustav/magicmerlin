//! Error types for provider routing.

use std::time::Duration;

use thiserror::Error;

/// Result alias used by the providers crate.
pub type Result<T> = std::result::Result<T, ProviderError>;

/// Errors raised while dispatching model completions.
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Generic HTTP transport failure.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// JSON parse/serialize failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Generic infrastructural failure from shared utilities.
    #[error("infra error: {0}")]
    Infra(#[from] magicmerlin_infra::InfraError),
    /// I/O failure when loading auth/model metadata.
    #[error("io error at {path}: {source}")]
    Io {
        /// I/O path.
        path: std::path::PathBuf,
        /// Source error.
        source: std::io::Error,
    },
    /// A request failed with an API status and payload.
    #[error("api error status {status}: {body}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },
    /// Model identifier is invalid or not found.
    #[error("model resolution failed: {0}")]
    Model(String),
    /// Provider is not registered.
    #[error("provider not registered: {0}")]
    ProviderNotFound(String),
    /// Auth credentials are missing.
    #[error("missing auth for provider: {0}")]
    MissingAuth(String),
    /// Auth refresh failed.
    #[error("oauth refresh failed for provider {provider}: {message}")]
    OAuthRefresh {
        /// Provider name.
        provider: String,
        /// Failure details.
        message: String,
    },
    /// Timeout waiting for completion.
    #[error("timeout after {0:?}")]
    Timeout(Duration),
    /// All fallbacks were exhausted.
    #[error("all providers exhausted: {0}")]
    Exhausted(String),
}

impl ProviderError {
    /// Returns true when this error should trigger retry/failover.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout(_) => true,
            Self::Http(err) => err.is_timeout() || err.is_connect(),
            Self::Api { status, .. } => matches!(*status, 401 | 429 | 500..=599),
            _ => false,
        }
    }

    /// Returns `Retry-After` duration parsed from API response headers/body when available.
    pub fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            Self::Api { status: 429, .. } => Some(Duration::from_secs(1)),
            _ => None,
        }
    }
}

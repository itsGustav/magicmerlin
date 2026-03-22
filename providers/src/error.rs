//! Error types for provider routing.

use std::time::Duration;

use thiserror::Error;

/// Result alias used by the providers crate.
pub type Result<T> = std::result::Result<T, ProviderError>;

/// Structured retry hint extracted from provider error headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryHint {
    /// Backoff duration.
    pub after: Duration,
    /// Whether rotating API keys is recommended.
    pub rotate_key: bool,
}

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
        /// Optional Retry-After value in seconds.
        retry_after_seconds: Option<u64>,
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
    /// Stream protocol parse failure.
    #[error("stream parse error: {0}")]
    StreamProtocol(String),
    /// Invalid request for provider.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// Circuit breaker is open for provider.
    #[error("circuit breaker open for provider {provider} ({remaining_ms}ms remaining)")]
    CircuitOpen {
        /// Provider name.
        provider: String,
        /// Remaining cooldown in milliseconds.
        remaining_ms: u64,
    },
}

impl ProviderError {
    /// Creates an API error with optional retry hint extracted from headers.
    pub fn api(status: u16, body: String, retry_after_seconds: Option<u64>) -> Self {
        Self::Api {
            status,
            body,
            retry_after_seconds,
        }
    }

    /// Returns true when this error should trigger retry/failover.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout(_) => true,
            Self::Http(err) => err.is_timeout() || err.is_connect() || err.is_request(),
            Self::Api { status, .. } => matches!(*status, 401 | 408 | 409 | 425 | 429 | 500..=599),
            Self::CircuitOpen { .. } => true,
            _ => false,
        }
    }

    /// Returns parsed retry hint when available.
    pub fn retry_hint(&self) -> Option<RetryHint> {
        match self {
            Self::Api {
                status: 429,
                retry_after_seconds,
                ..
            } => Some(RetryHint {
                after: Duration::from_secs(retry_after_seconds.unwrap_or(1).max(1)),
                rotate_key: true,
            }),
            Self::Api {
                status: 529,
                retry_after_seconds,
                ..
            }
            | Self::Api {
                status: 503,
                retry_after_seconds,
                ..
            }
            | Self::Api {
                status: 502,
                retry_after_seconds,
                ..
            }
            | Self::Api {
                status: 500,
                retry_after_seconds,
                ..
            } => Some(RetryHint {
                after: Duration::from_secs(retry_after_seconds.unwrap_or(2).max(1)),
                rotate_key: false,
            }),
            Self::Timeout(_) => Some(RetryHint {
                after: Duration::from_millis(500),
                rotate_key: false,
            }),
            _ => None,
        }
    }

    /// Returns `Retry-After` duration parsed from API response hints when available.
    pub fn retry_after_hint(&self) -> Option<Duration> {
        self.retry_hint().map(|hint| hint.after)
    }

    /// Returns true when provider auth rotation may help recover.
    pub fn should_rotate_auth(&self) -> bool {
        self.retry_hint()
            .map(|hint| hint.rotate_key)
            .unwrap_or(false)
    }

    /// Returns status code if this is an API error.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_hint_for_rate_limit() {
        let err = ProviderError::api(429, "too many".to_string(), Some(3));
        let hint = err.retry_hint().expect("hint");
        assert_eq!(hint.after, Duration::from_secs(3));
        assert!(hint.rotate_key);
    }

    #[test]
    fn non_retryable_invalid_request() {
        let err = ProviderError::InvalidRequest("bad".to_string());
        assert!(!err.is_retryable());
        assert!(err.retry_hint().is_none());
    }
}

//! Error types for configuration loading, mutation, and validation.

use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Error type for all config crate operations.
#[derive(Debug)]
pub enum ConfigError {
    /// Filesystem read failure.
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Filesystem write failure.
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Directory creation failure.
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Config parse failure.
    ParseConfig {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// Serialization failure.
    Serialize(serde_json::Error),
    /// Deserialization failure.
    Deserialize(serde_json::Error),
    /// Invalid dot-path syntax or traversal.
    InvalidPath(String),
    /// Validation failure.
    Validation(String),
    /// Invalid document shape.
    InvalidDocument,
    /// Home directory resolution failed.
    HomeDirUnavailable,
    /// Secret loading failure.
    Secrets(crate::SecretsError),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile { path, source } => {
                write!(f, "failed to read {}: {}", path.display(), source)
            }
            Self::WriteFile { path, source } => {
                write!(f, "failed to write {}: {}", path.display(), source)
            }
            Self::CreateDir { path, source } => {
                write!(
                    f,
                    "failed to create directory {}: {}",
                    path.display(),
                    source
                )
            }
            Self::ParseConfig { path, source } => {
                write!(f, "failed to parse config {}: {}", path.display(), source)
            }
            Self::Serialize(source) => write!(f, "serialization error: {source}"),
            Self::Deserialize(source) => write!(f, "deserialization error: {source}"),
            Self::InvalidPath(path) => write!(f, "invalid path: {path}"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
            Self::InvalidDocument => write!(f, "invalid config document"),
            Self::HomeDirUnavailable => write!(f, "unable to resolve home directory"),
            Self::Secrets(source) => Display::fmt(source, f),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<crate::SecretsError> for ConfigError {
    fn from(value: crate::SecretsError) -> Self {
        Self::Secrets(value)
    }
}

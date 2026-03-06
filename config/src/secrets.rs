//! Secrets file loader for `secrets.env` in KEY=VALUE format.

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

/// Secrets loading and access errors.
#[derive(Debug)]
pub enum SecretsError {
    /// Secret file read failure.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Display for SecretsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read secrets file {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for SecretsError {}

/// Loaded secret values from `secrets.env`.
#[derive(Debug, Clone, Default)]
pub struct Secrets {
    values: BTreeMap<String, String>,
}

impl Secrets {
    /// Loads secrets from disk. Missing files return an empty map.
    pub fn load(path: &Path) -> Result<Self, SecretsError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let body = std::fs::read_to_string(path).map_err(|source| SecretsError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        let mut values = BTreeMap::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(Self { values })
    }

    /// Gets a secret value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

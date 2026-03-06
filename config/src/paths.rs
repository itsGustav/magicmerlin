//! State directory path resolution helpers.

use std::path::PathBuf;

use crate::ConfigError;

/// Scope settings used to resolve the OpenClaw state directory.
#[derive(Debug, Clone, Default)]
pub enum PathScope {
    /// Default location (`~/.openclaw`).
    #[default]
    Default,
    /// Profile location (`~/.openclaw-<name>`).
    Profile(String),
    /// Development location (`~/.openclaw-dev`).
    Dev,
}

impl PathScope {
    /// Creates profile scope.
    pub fn profile(name: String) -> Self {
        Self::Profile(name)
    }

    /// Creates dev scope.
    pub fn dev() -> Self {
        Self::Dev
    }
}

/// Canonical state subdirectories and files.
#[derive(Debug, Clone)]
pub struct StatePaths {
    /// Root state directory.
    pub state_dir: PathBuf,
    /// Agents directory.
    pub agents_dir: PathBuf,
    /// Sessions directory.
    pub sessions_dir: PathBuf,
    /// Logs directory.
    pub logs_dir: PathBuf,
    /// Media directory.
    pub media_dir: PathBuf,
    /// Secrets file path.
    pub secrets_file: PathBuf,
}

impl StatePaths {
    /// Resolves all state paths and ensures required directories exist.
    pub fn new(scope: PathScope) -> Result<Self, ConfigError> {
        let state_dir = if let Ok(override_dir) = std::env::var("OPENCLAW_STATE_DIR") {
            PathBuf::from(override_dir)
        } else {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(PathBuf::from)
                .ok_or(ConfigError::HomeDirUnavailable)?;

            match scope {
                PathScope::Default => home.join(".openclaw"),
                PathScope::Profile(name) => home.join(format!(".openclaw-{name}")),
                PathScope::Dev => home.join(".openclaw-dev"),
            }
        };

        let paths = Self {
            agents_dir: state_dir.join("agents"),
            sessions_dir: state_dir.join("sessions"),
            logs_dir: state_dir.join("logs"),
            media_dir: state_dir.join("media"),
            secrets_file: state_dir.join("secrets.env"),
            state_dir,
        };

        std::fs::create_dir_all(&paths.state_dir).map_err(|source| ConfigError::CreateDir {
            path: paths.state_dir.clone(),
            source,
        })?;
        std::fs::create_dir_all(&paths.agents_dir).map_err(|source| ConfigError::CreateDir {
            path: paths.agents_dir.clone(),
            source,
        })?;
        std::fs::create_dir_all(&paths.sessions_dir).map_err(|source| ConfigError::CreateDir {
            path: paths.sessions_dir.clone(),
            source,
        })?;
        std::fs::create_dir_all(&paths.logs_dir).map_err(|source| ConfigError::CreateDir {
            path: paths.logs_dir.clone(),
            source,
        })?;
        std::fs::create_dir_all(&paths.media_dir).map_err(|source| ConfigError::CreateDir {
            path: paths.media_dir.clone(),
            source,
        })?;

        Ok(paths)
    }
}

//! Configuration loading and mutation helpers for MagicMerlin/OpenClaw-shaped state.

mod error;
mod model;
mod paths;
mod secrets;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use error::ConfigError;
pub use model::{Config, GatewayConfig};
pub use paths::{PathScope, StatePaths};
pub use secrets::{Secrets, SecretsError};
use serde_json::Value;

/// Runtime options that control where configuration and state are resolved.
#[derive(Debug, Clone, Default)]
pub struct ConfigOptions {
    /// Optional profile name. When set, state root becomes `~/.openclaw-<profile>`.
    pub profile: Option<String>,
    /// When true, state root becomes `~/.openclaw-dev` and default gateway port is forced to 19001.
    pub dev: bool,
}

/// In-memory configuration document with resolved paths and loaded secrets.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
    state_paths: StatePaths,
    secrets: Secrets,
}

impl ConfigManager {
    /// Loads configuration from disk (or environment override), applies env overlays, and validates.
    pub fn load(options: ConfigOptions) -> Result<Self, ConfigError> {
        let scope = if options.dev {
            PathScope::dev()
        } else if let Some(profile) = options.profile {
            PathScope::profile(profile)
        } else {
            PathScope::Default
        };

        let state_paths = StatePaths::new(scope)?;
        let config_path = resolve_config_path(&state_paths);

        let mut config = if config_path.exists() {
            let raw = fs::read_to_string(&config_path).map_err(|source| ConfigError::ReadFile {
                path: config_path.clone(),
                source,
            })?;
            serde_json::from_str::<Config>(&raw).map_err(|source| ConfigError::ParseConfig {
                path: config_path.clone(),
                source,
            })?
        } else {
            Config::default()
        };

        apply_env_overrides(&mut config)?;
        if options.dev {
            config.gateway.port = Some(19001);
        }
        config.validate()?;

        let secrets = Secrets::load(&state_paths.secrets_file)?;

        Ok(Self {
            config,
            config_path,
            state_paths,
            secrets,
        })
    }

    /// Returns a read-only view of the current typed configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns absolute path to the current config file.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Returns resolved state-directory helper paths.
    pub fn state_paths(&self) -> &StatePaths {
        &self.state_paths
    }

    /// Returns the loaded secrets handle.
    pub fn secrets(&self) -> &Secrets {
        &self.secrets
    }

    /// Gets a value by dot path (`gateway.port`, `agents.defaults.model`, etc.).
    pub fn get(&self, path: &str) -> Option<Value> {
        let value = serde_json::to_value(&self.config).ok()?;
        get_at_path(&value, path).cloned()
    }

    /// Sets a value by dot path and validates the resulting document.
    pub fn set(&mut self, path: &str, raw_value: &str) -> Result<(), ConfigError> {
        let mut value = serde_json::to_value(&self.config).map_err(ConfigError::Serialize)?;
        let parsed = parse_set_value(raw_value);
        set_at_path(&mut value, path, parsed)?;
        self.config = serde_json::from_value(value).map_err(ConfigError::Deserialize)?;
        self.config.validate()?;
        Ok(())
    }

    /// Removes a value by dot path and validates the resulting document.
    pub fn unset(&mut self, path: &str) -> Result<(), ConfigError> {
        let mut value = serde_json::to_value(&self.config).map_err(ConfigError::Serialize)?;
        unset_at_path(&mut value, path)?;
        self.config = serde_json::from_value(value).map_err(ConfigError::Deserialize)?;
        self.config.validate()?;
        Ok(())
    }

    /// Persists the current config as pretty JSON to disk.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let body = serde_json::to_string_pretty(&self.config).map_err(ConfigError::Serialize)?;
        fs::write(&self.config_path, format!("{body}\n")).map_err(|source| ConfigError::WriteFile {
            path: self.config_path.clone(),
            source,
        })
    }
}

/// Resolves config path from env override or state directory.
pub fn resolve_config_path(state_paths: &StatePaths) -> PathBuf {
    if let Ok(path) = std::env::var("OPENCLAW_CONFIG_PATH") {
        return PathBuf::from(path);
    }
    state_paths.state_dir.join("openclaw.json")
}

/// Applies `OPENCLAW_*` environment variable overrides to config values.
pub fn apply_env_overrides(config: &mut Config) -> Result<(), ConfigError> {
    let mut value = serde_json::to_value(&*config).map_err(ConfigError::Serialize)?;
    let allowed_roots = [
        "meta", "wizard", "auth", "acp", "models", "agents", "tools", "bindings", "messages",
        "commands", "channels", "talk", "gateway", "skills", "plugins",
    ];

    for (key, env_value) in std::env::vars() {
        if !key.starts_with("OPENCLAW_") {
            continue;
        }
        if matches!(
            key.as_str(),
            "OPENCLAW_CONFIG_PATH" | "OPENCLAW_PROFILE" | "OPENCLAW_DEV" | "OPENCLAW_STATE_DIR"
        ) {
            continue;
        }

        let suffix = key.trim_start_matches("OPENCLAW_");
        if suffix.is_empty() {
            continue;
        }

        let path = suffix
            .to_ascii_lowercase()
            .replace("__", "-")
            .split('_')
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(".");

        let root = path.split('.').next().unwrap_or_default();
        if !allowed_roots.contains(&root) {
            continue;
        }

        set_at_path(&mut value, &path, parse_set_value(&env_value))?;
    }

    *config = serde_json::from_value(value).map_err(ConfigError::Deserialize)?;
    config.validate()?;
    Ok(())
}

fn parse_set_value(input: &str) -> Value {
    serde_json::from_str(input).unwrap_or_else(|_| Value::String(input.to_string()))
}

fn get_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.').filter(|p| !p.is_empty()) {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

fn set_at_path(root: &mut Value, path: &str, value: Value) -> Result<(), ConfigError> {
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Err(ConfigError::InvalidPath(path.to_string()));
    }

    let mut current = root;
    for (idx, part) in parts.iter().enumerate() {
        let is_last = idx == parts.len() - 1;
        match current {
            Value::Object(map) => {
                if is_last {
                    map.insert((*part).to_string(), value);
                    return Ok(());
                }
                current = map
                    .entry((*part).to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
            }
            _ => {
                return Err(ConfigError::InvalidPath(path.to_string()));
            }
        }
    }

    Ok(())
}

fn unset_at_path(root: &mut Value, path: &str) -> Result<(), ConfigError> {
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Err(ConfigError::InvalidPath(path.to_string()));
    }

    let mut current = root;
    for (idx, part) in parts.iter().enumerate() {
        let is_last = idx == parts.len() - 1;
        match current {
            Value::Object(map) => {
                if is_last {
                    map.remove(*part);
                    return Ok(());
                }
                current = map
                    .get_mut(*part)
                    .ok_or_else(|| ConfigError::InvalidPath(path.to_string()))?;
            }
            _ => return Err(ConfigError::InvalidPath(path.to_string())),
        }
    }

    Ok(())
}

/// Returns a one-level key-value snapshot suitable for CLI display.
pub fn flatten_top_level(config: &Config) -> Result<BTreeMap<String, Value>, ConfigError> {
    let value = serde_json::to_value(config).map_err(ConfigError::Serialize)?;
    let Value::Object(map) = value else {
        return Err(ConfigError::InvalidDocument);
    };

    Ok(map.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn set_and_get_path() {
        let mut value = serde_json::json!({"a": {"b": 1}});
        set_at_path(&mut value, "a.c", Value::String("x".to_string())).unwrap();
        assert_eq!(
            get_at_path(&value, "a.c"),
            Some(&Value::String("x".to_string()))
        );
    }

    #[test]
    fn unset_path() {
        let mut value = serde_json::json!({"a": {"b": 1}});
        unset_at_path(&mut value, "a.b").unwrap();
        assert!(get_at_path(&value, "a.b").is_none());
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn manager_loads_secrets() {
        let _guard = env_lock().lock().expect("lock");
        let temp = tempfile::tempdir().unwrap();
        let state = temp.path().join("state");
        fs::create_dir_all(&state).unwrap();
        fs::write(
            state.join("openclaw.json"),
            r#"{"gateway":{"port":18789},"agents":{"defaults":{"model":"gpt-5"}}}"#,
        )
        .unwrap();

        let mut file = fs::File::create(state.join("secrets.env")).unwrap();
        writeln!(file, "API_TOKEN=abc").unwrap();

        std::env::set_var("OPENCLAW_CONFIG_PATH", state.join("openclaw.json"));
        std::env::set_var("OPENCLAW_STATE_DIR", &state);
        let mut mgr = ConfigManager::load(ConfigOptions::default()).unwrap();
        assert_eq!(mgr.secrets().get("API_TOKEN"), Some("abc"));

        mgr.set("gateway.port", "19000").unwrap();
        assert_eq!(mgr.get("gateway.port"), Some(Value::Number(19000.into())));

        std::env::remove_var("OPENCLAW_CONFIG_PATH");
        std::env::remove_var("OPENCLAW_STATE_DIR");
    }
}

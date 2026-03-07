//! Multi-agent descriptor registry.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AgentError, Result};

/// Agent descriptor loaded from `~/.openclaw/agents/*` directories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptor {
    /// Agent id (directory name).
    pub id: String,
    /// Workspace directory.
    pub workspace_dir: PathBuf,
    /// Agent home directory.
    pub agent_dir: PathBuf,
    /// Sessions directory.
    pub sessions_dir: PathBuf,
    /// Preferred model.
    pub model: Option<String>,
    /// Fallback chain.
    pub fallbacks: Vec<String>,
    /// Optional identity emoji.
    pub identity_emoji: Option<String>,
    /// Raw config overrides.
    pub config_overrides: Value,
}

/// Registry of known agents.
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: Vec<AgentDescriptor>,
}

impl AgentRegistry {
    /// Loads agents from state `agents` directory.
    pub fn load_from(base_agents_dir: impl AsRef<Path>) -> Result<Self> {
        let mut agents = Vec::new();
        let base = base_agents_dir.as_ref();
        if !base.exists() {
            return Ok(Self { agents });
        }

        for entry in fs::read_dir(base).map_err(|source| AgentError::Io {
            path: base.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| AgentError::Io {
                path: base.to_path_buf(),
                source,
            })?;
            if !entry.path().is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            let agent_dir = entry.path();
            let cfg_path = agent_dir.join("agent.json");
            let cfg = if cfg_path.exists() {
                let raw = fs::read_to_string(&cfg_path).map_err(|source| AgentError::Io {
                    path: cfg_path.clone(),
                    source,
                })?;
                serde_json::from_str::<Value>(&raw)?
            } else {
                Value::Object(serde_json::Map::new())
            };

            let model = cfg.get("model").and_then(Value::as_str).map(str::to_string);
            let fallbacks = cfg
                .get("fallbacks")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            agents.push(AgentDescriptor {
                id: id.clone(),
                workspace_dir: cfg
                    .get("workspace_dir")
                    .and_then(Value::as_str)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| agent_dir.clone()),
                agent_dir: agent_dir.clone(),
                sessions_dir: agent_dir.join("sessions"),
                model,
                fallbacks,
                identity_emoji: cfg
                    .get("identity_emoji")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                config_overrides: cfg,
            });
        }

        Ok(Self { agents })
    }

    /// Returns all known agents.
    pub fn all(&self) -> &[AgentDescriptor] {
        &self.agents
    }
}

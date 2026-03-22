//! Multi-agent descriptor registry.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AgentError, Result};

/// Agent runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// Preferred model.
    pub model: Option<String>,
    /// Fallback chain.
    #[serde(default)]
    pub fallbacks: Vec<String>,
    /// Workspace directory.
    pub workspace_dir: Option<PathBuf>,
    /// Identity emoji.
    pub identity_emoji: Option<String>,
    /// Heartbeat quiet hours start (`HH:MM`).
    pub heartbeat_quiet_start: Option<String>,
    /// Heartbeat quiet hours end (`HH:MM`).
    pub heartbeat_quiet_end: Option<String>,
    /// Additional map for future settings.
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

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
    /// Parsed config.
    pub config: AgentConfig,
    /// Last health check timestamp.
    pub last_health_check_epoch: i64,
}

/// Route target for inbound messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteDecision {
    /// Agent id chosen.
    pub agent_id: String,
    /// Confidence score [0.0..1.0].
    pub confidence: u8,
    /// Explanation string.
    pub reason: String,
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
            let cfg = load_config(&agent_dir)?;

            agents.push(AgentDescriptor {
                id: id.clone(),
                workspace_dir: cfg
                    .workspace_dir
                    .clone()
                    .unwrap_or_else(|| agent_dir.clone()),
                agent_dir: agent_dir.clone(),
                sessions_dir: agent_dir.join("sessions"),
                config: cfg,
                last_health_check_epoch: 0,
            });
        }

        agents.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self { agents })
    }

    /// Returns all known agents.
    pub fn all(&self) -> &[AgentDescriptor] {
        &self.agents
    }

    /// Returns descriptor by id.
    pub fn by_id(&self, id: &str) -> Option<&AgentDescriptor> {
        self.agents.iter().find(|agent| agent.id == id)
    }

    /// Adds or replaces descriptor.
    pub fn upsert(&mut self, descriptor: AgentDescriptor) {
        if let Some(existing) = self
            .agents
            .iter_mut()
            .find(|agent| agent.id == descriptor.id)
        {
            *existing = descriptor;
            return;
        }
        self.agents.push(descriptor);
        self.agents.sort_by(|a, b| a.id.cmp(&b.id));
    }

    /// Removes descriptor by id.
    pub fn remove(&mut self, id: &str) {
        self.agents.retain(|agent| agent.id != id);
    }

    /// Computes simple health state (filesystem + core files present).
    pub fn health_report(&self) -> Vec<(String, bool, String)> {
        self.agents
            .iter()
            .map(|agent| {
                let mut reasons = Vec::new();
                if !agent.agent_dir.exists() {
                    reasons.push("agent_dir_missing");
                }
                if !agent.workspace_dir.exists() {
                    reasons.push("workspace_dir_missing");
                }
                if !agent.sessions_dir.exists() {
                    reasons.push("sessions_dir_missing");
                }
                if !agent.agent_dir.join("IDENTITY.md").exists() {
                    reasons.push("identity_missing");
                }
                let healthy = reasons.is_empty();
                (
                    agent.id.clone(),
                    healthy,
                    if healthy {
                        "ok".to_string()
                    } else {
                        reasons.join(",")
                    },
                )
            })
            .collect()
    }

    /// Resolves which agent should handle inbound message.
    pub fn route_message(
        &self,
        channel: &str,
        workspace_hint: Option<&Path>,
        text: &str,
    ) -> Option<RouteDecision> {
        if self.agents.is_empty() {
            return None;
        }

        if let Some(workspace_hint) = workspace_hint {
            for agent in &self.agents {
                if agent.workspace_dir == workspace_hint {
                    return Some(RouteDecision {
                        agent_id: agent.id.clone(),
                        confidence: 95,
                        reason: "workspace match".to_string(),
                    });
                }
            }
        }

        let lower = text.to_ascii_lowercase();
        for agent in &self.agents {
            let identity = agent
                .config
                .identity_emoji
                .clone()
                .unwrap_or_else(|| agent.id.clone());
            if lower.contains(&agent.id.to_ascii_lowercase())
                || lower.contains(&identity.to_ascii_lowercase())
            {
                return Some(RouteDecision {
                    agent_id: agent.id.clone(),
                    confidence: 90,
                    reason: "explicit mention".to_string(),
                });
            }
        }

        if channel == "terminal" {
            return self.agents.first().map(|agent| RouteDecision {
                agent_id: agent.id.clone(),
                confidence: 70,
                reason: "terminal default".to_string(),
            });
        }

        self.agents.first().map(|agent| RouteDecision {
            agent_id: agent.id.clone(),
            confidence: 60,
            reason: "fallback first agent".to_string(),
        })
    }

    /// Marks all agents as recently health-checked.
    pub fn mark_health_checked(&mut self) {
        let now = Utc::now().timestamp();
        for agent in &mut self.agents {
            agent.last_health_check_epoch = now;
        }
    }
}

fn load_config(agent_dir: &Path) -> Result<AgentConfig> {
    let cfg_path = agent_dir.join("agent.json");
    if !cfg_path.exists() {
        return Ok(AgentConfig::default());
    }
    let raw = fs::read_to_string(&cfg_path).map_err(|source| AgentError::Io {
        path: cfg_path.clone(),
        source,
    })?;

    if raw.trim().is_empty() {
        return Ok(AgentConfig::default());
    }

    match serde_json::from_str::<AgentConfig>(&raw) {
        Ok(config) => Ok(config),
        Err(_) => {
            let fallback: Value = serde_json::from_str(&raw)?;
            Ok(AgentConfig {
                model: fallback
                    .get("model")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                fallbacks: fallback
                    .get("fallbacks")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                workspace_dir: fallback
                    .get("workspace_dir")
                    .and_then(Value::as_str)
                    .map(PathBuf::from),
                identity_emoji: fallback
                    .get("identity_emoji")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                heartbeat_quiet_start: fallback
                    .get("heartbeat_quiet_start")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                heartbeat_quiet_end: fallback
                    .get("heartbeat_quiet_end")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                extra: BTreeMap::new(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_agents_and_routes_by_mention() {
        let temp = tempfile::tempdir().expect("tmp");
        let a1 = temp.path().join("merlin");
        let a2 = temp.path().join("helper");
        std::fs::create_dir_all(&a1).expect("dir1");
        std::fs::create_dir_all(&a2).expect("dir2");
        std::fs::write(
            a1.join("agent.json"),
            r#"{"model":"openai/gpt-5.2","identity_emoji":"🧙"}"#,
        )
        .expect("cfg");
        std::fs::write(a1.join("IDENTITY.md"), "Merlin").expect("identity");
        std::fs::create_dir_all(a1.join("sessions")).expect("sessions1");
        std::fs::write(a2.join("IDENTITY.md"), "Helper").expect("identity2");
        std::fs::create_dir_all(a2.join("sessions")).expect("sessions2");

        let registry = AgentRegistry::load_from(temp.path()).expect("registry");
        assert_eq!(registry.all().len(), 2);

        let decision = registry
            .route_message("terminal", None, "ask merlin to respond")
            .expect("decision");
        assert_eq!(decision.agent_id, "merlin");
        assert!(decision.confidence >= 90);
    }

    #[test]
    fn reports_health() {
        let temp = tempfile::tempdir().expect("tmp");
        let a1 = temp.path().join("merlin");
        std::fs::create_dir_all(&a1).expect("dir");
        std::fs::write(a1.join("IDENTITY.md"), "Merlin").expect("identity");
        std::fs::create_dir_all(a1.join("sessions")).expect("sessions");

        let registry = AgentRegistry::load_from(temp.path()).expect("registry");
        let report = registry.health_report();
        assert_eq!(report.len(), 1);
        assert!(report[0].1);
    }
}

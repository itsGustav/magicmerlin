//! Strongly-typed configuration model and validation.

use serde::{Deserialize, Serialize};

use crate::ConfigError;

/// Full OpenClaw-shaped configuration document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Metadata section.
    pub meta: Section,
    /// Wizard section.
    pub wizard: Section,
    /// Auth section.
    pub auth: Section,
    /// ACP section.
    pub acp: Section,
    /// Models section.
    pub models: Section,
    /// Agents section.
    pub agents: AgentsConfig,
    /// Tools section.
    pub tools: Section,
    /// Bindings section.
    pub bindings: Section,
    /// Messages section.
    pub messages: Section,
    /// Commands section.
    pub commands: Section,
    /// Channels section.
    pub channels: Section,
    /// Talk section.
    pub talk: Section,
    /// Gateway section.
    pub gateway: GatewayConfig,
    /// Skills section.
    pub skills: Section,
    /// Plugins section.
    pub plugins: Section,
}

impl Config {
    /// Validates typed fields and value ranges.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(port) = self.gateway.port {
            if port == 0 {
                return Err(ConfigError::Validation(
                    "gateway.port must be in 1..=65535".to_string(),
                ));
            }
        }

        if let Some(timeout_seconds) = self.agents.defaults.timeout_seconds {
            if timeout_seconds == 0 || timeout_seconds > 86_400 {
                return Err(ConfigError::Validation(
                    "agents.defaults.timeout_seconds must be in 1..=86400".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Generic section payload for less-constrained areas.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Section {
    /// Raw JSON object payload.
    #[serde(flatten)]
    pub values: serde_json::Map<String, serde_json::Value>,
}

/// Agents-specific typed fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentsConfig {
    /// Default agent settings.
    pub defaults: AgentDefaults,
    /// Named agent configurations (merlin, henry, paylobster, lobsterprime, …).
    #[serde(default)]
    pub named: std::collections::HashMap<String, NamedAgentConfig>,
    /// Additional fields not currently typed.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Configuration for a named agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NamedAgentConfig {
    /// Model identifier override.
    pub model: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Workspace directory override.
    pub workspace: Option<String>,
    /// Agent directory override.
    pub agent_dir: Option<String>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Agent defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentDefaults {
    /// Default model identifier.
    pub model: Option<String>,
    /// Turn timeout in seconds.
    pub timeout_seconds: Option<u64>,
    /// Additional fields not currently typed.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Gateway-specific typed fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    /// Gateway bind port.
    pub port: Option<u16>,
    /// Gateway bind address.
    pub bind: Option<String>,
    /// Additional fields not currently typed.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

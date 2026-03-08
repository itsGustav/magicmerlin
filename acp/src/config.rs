use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Canonical coding-agent identifiers supported by ACP.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentId {
    /// Anthropic Claude Code CLI.
    ClaudeCode,
    /// OpenAI Codex CLI.
    Codex,
    /// OpenCode agent.
    OpenCode,
    /// Gemini code agent.
    Gemini,
    /// Pi code agent.
    Pi,
    /// Custom external agent name.
    Custom(String),
}

impl AgentId {
    /// Returns display name for this agent id.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
            Self::Pi => "pi",
            Self::Custom(name) => name,
        }
    }
}

/// ACP harness policy and limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentHarnessConfig {
    /// Allowed agent list.
    pub allowed_agents: BTreeSet<AgentId>,
    /// Maximum number of concurrent subprocess sessions.
    pub max_concurrent_sessions: usize,
    /// Session time-to-live in seconds before auto-cleanup.
    pub ttl_seconds: u64,
}

impl Default for AgentHarnessConfig {
    fn default() -> Self {
        let mut allowed = BTreeSet::new();
        allowed.insert(AgentId::ClaudeCode);
        allowed.insert(AgentId::Codex);
        allowed.insert(AgentId::OpenCode);
        allowed.insert(AgentId::Gemini);
        allowed.insert(AgentId::Pi);

        Self {
            allowed_agents: allowed,
            max_concurrent_sessions: 4,
            ttl_seconds: 1800,
        }
    }
}

impl AgentHarnessConfig {
    /// Returns true if the agent is currently allowed by policy.
    pub fn is_allowed(&self, agent: &AgentId) -> bool {
        self.allowed_agents.contains(agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_required_agents() {
        let cfg = AgentHarnessConfig::default();
        assert!(cfg.is_allowed(&AgentId::Codex));
        assert!(cfg.is_allowed(&AgentId::ClaudeCode));
    }

    #[test]
    fn custom_agent_string_is_preserved() {
        let agent = AgentId::Custom("my-agent".to_string());
        assert_eq!(agent.as_str(), "my-agent");
    }
}

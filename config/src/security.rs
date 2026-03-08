use std::collections::BTreeMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Severity level for an audit issue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SecuritySeverity {
    /// Informational finding.
    Info,
    /// Elevated risk.
    Warning,
    /// High-risk finding.
    Critical,
}

/// One security finding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIssue {
    /// Machine-readable issue code.
    pub code: String,
    /// Severity level.
    pub severity: SecuritySeverity,
    /// Human-readable summary.
    pub message: String,
}

/// Input data used by the security audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAuditContext {
    /// True when the bot is publicly reachable.
    pub public_bot: bool,
    /// True when DMs are open without gating.
    pub open_dm_policy: bool,
    /// True when sandbox mode is configured.
    pub sandbox_configured: bool,
    /// Gateway auth token.
    pub gateway_token: Option<String>,
    /// Gateway bind address.
    pub gateway_bind: Option<String>,
    /// Gateway port.
    pub gateway_port: Option<u16>,
    /// Count of stale high-token sessions.
    pub stale_high_token_sessions: usize,
    /// Root workspace for filesystem restriction checks.
    pub workspace_root: PathBuf,
    /// Per-agent tool deny lists.
    #[serde(default)]
    pub tool_deny_lists: BTreeMap<String, Vec<String>>,
    /// Trusted reverse proxies.
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
}

impl Default for SecurityAuditContext {
    fn default() -> Self {
        Self {
            public_bot: false,
            open_dm_policy: false,
            sandbox_configured: true,
            gateway_token: None,
            gateway_bind: Some("127.0.0.1".to_string()),
            gateway_port: Some(18789),
            stale_high_token_sessions: 0,
            workspace_root: PathBuf::from("."),
            tool_deny_lists: BTreeMap::new(),
            trusted_proxies: Vec::new(),
        }
    }
}

/// Aggregated security report.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAuditReport {
    /// Findings emitted by the audit.
    pub issues: Vec<SecurityIssue>,
}

impl SecurityAuditReport {
    /// Returns true when no warning/critical issues were detected.
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Runs security checks for common gateway and runtime risks.
pub fn run_security_audit(ctx: &SecurityAuditContext) -> SecurityAuditReport {
    let mut issues = Vec::new();

    if ctx.public_bot && ctx.open_dm_policy {
        issues.push(SecurityIssue {
            code: "open_dm_policy".to_string(),
            severity: SecuritySeverity::Warning,
            message: "Public bot has open DM policy; require mention or allowlist gating".to_string(),
        });
    }

    if !ctx.sandbox_configured {
        issues.push(SecurityIssue {
            code: "missing_sandbox".to_string(),
            severity: SecuritySeverity::Critical,
            message: "Sandbox is not configured for tool execution".to_string(),
        });
    }

    if ctx
        .gateway_token
        .as_deref()
        .map_or(true, |token| token.trim().is_empty())
    {
        issues.push(SecurityIssue {
            code: "weak_gateway_auth".to_string(),
            severity: SecuritySeverity::Warning,
            message: "Gateway token is missing".to_string(),
        });
    }

    if let Some(bind) = ctx.gateway_bind.as_deref() {
        if bind == "0.0.0.0" || bind == "::" {
            issues.push(SecurityIssue {
                code: "exposed_bind_address".to_string(),
                severity: SecuritySeverity::Warning,
                message: format!("Gateway bind address is publicly exposed: {bind}"),
            });
        }
    }

    if ctx.stale_high_token_sessions > 0 {
        issues.push(SecurityIssue {
            code: "stale_high_token_sessions".to_string(),
            severity: SecuritySeverity::Info,
            message: format!(
                "{} stale sessions have high token usage",
                ctx.stale_high_token_sessions
            ),
        });
    }

    for proxy in &ctx.trusted_proxies {
        if !validate_trusted_proxy(proxy) {
            issues.push(SecurityIssue {
                code: "invalid_trusted_proxy".to_string(),
                severity: SecuritySeverity::Warning,
                message: format!("Trusted proxy entry is invalid: {proxy}"),
            });
        }
    }

    SecurityAuditReport { issues }
}

/// Validates that a file path resolves within the workspace root.
pub fn validate_workspace_path(workspace_root: &Path, requested_path: &Path) -> bool {
    let root = normalize_path(workspace_root);
    let requested = if requested_path.is_absolute() {
        normalize_path(requested_path)
    } else {
        normalize_path(&workspace_root.join(requested_path))
    };
    requested.starts_with(&root)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Returns true when the tool is not denied for the given agent.
pub fn is_tool_allowed(
    deny_lists: &BTreeMap<String, Vec<String>>,
    agent: &str,
    tool_name: &str,
) -> bool {
    let blocked_for_agent = deny_lists
        .get(agent)
        .into_iter()
        .flat_map(|items| items.iter())
        .any(|tool| tool == tool_name || tool == "*");

    let blocked_globally = deny_lists
        .get("*")
        .into_iter()
        .flat_map(|items| items.iter())
        .any(|tool| tool == tool_name || tool == "*");

    !(blocked_for_agent || blocked_globally)
}

/// Validates trusted proxy definitions (`ip` or `ip/cidr`).
pub fn validate_trusted_proxy(value: &str) -> bool {
    if value.trim().is_empty() {
        return false;
    }

    if let Ok(_ip) = value.parse::<IpAddr>() {
        return true;
    }

    let mut parts = value.split('/');
    let ip = parts.next();
    let cidr = parts.next();
    if parts.next().is_some() {
        return false;
    }

    let Some(ip) = ip else {
        return false;
    };
    let Some(cidr) = cidr else {
        return false;
    };

    let Ok(_ip) = ip.parse::<IpAddr>() else {
        return false;
    };
    let Ok(prefix) = cidr.parse::<u8>() else {
        return false;
    };
    prefix <= 128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_detects_open_dm_policy() {
        let ctx = SecurityAuditContext {
            public_bot: true,
            open_dm_policy: true,
            ..SecurityAuditContext::default()
        };
        let report = run_security_audit(&ctx);
        assert!(report.issues.iter().any(|i| i.code == "open_dm_policy"));
    }

    #[test]
    fn audit_detects_missing_sandbox() {
        let ctx = SecurityAuditContext {
            sandbox_configured: false,
            ..SecurityAuditContext::default()
        };
        let report = run_security_audit(&ctx);
        assert!(report.issues.iter().any(|i| i.code == "missing_sandbox"));
    }

    #[test]
    fn audit_flags_exposed_bind() {
        let ctx = SecurityAuditContext {
            gateway_bind: Some("0.0.0.0".to_string()),
            ..SecurityAuditContext::default()
        };
        let report = run_security_audit(&ctx);
        assert!(report.issues.iter().any(|i| i.code == "exposed_bind_address"));
    }

    #[test]
    fn workspace_restriction_accepts_child_path() {
        let root = PathBuf::from("/workspace");
        assert!(validate_workspace_path(&root, Path::new("project/file.txt")));
    }

    #[test]
    fn workspace_restriction_rejects_escape() {
        let root = PathBuf::from("/workspace");
        assert!(!validate_workspace_path(&root, Path::new("../etc/passwd")));
    }

    #[test]
    fn tool_deny_list_blocks_agent_specific_tool() {
        let mut deny = BTreeMap::new();
        deny.insert("codex".to_string(), vec!["shell.exec".to_string()]);
        assert!(!is_tool_allowed(&deny, "codex", "shell.exec"));
        assert!(is_tool_allowed(&deny, "codex", "search"));
    }

    #[test]
    fn tool_deny_list_blocks_global_wildcard() {
        let mut deny = BTreeMap::new();
        deny.insert("*".to_string(), vec!["*".to_string()]);
        assert!(!is_tool_allowed(&deny, "codex", "search"));
    }

    #[test]
    fn validates_trusted_proxy_values() {
        assert!(validate_trusted_proxy("127.0.0.1"));
        assert!(validate_trusted_proxy("10.0.0.0/8"));
        assert!(!validate_trusted_proxy("bad-value"));
        assert!(!validate_trusted_proxy("1.2.3.4/999"));
    }
}

use serde::de::DeserializeOwned;
use serde_json::Value;

pub mod agent_run;
pub mod approvals;
pub mod config;
pub mod cron;
pub mod health;
pub mod plugins;
pub mod sessions;
pub mod status;
pub mod system;

pub const SUPPORTED_METHODS: &[&str] = &[
    "health",
    "status",
    "system-presence",
    "agent.run",
    "agent.abort",
    "sessions.list",
    "sessions.get",
    "sessions.send",
    "sessions.spawn",
    "sessions.compact",
    "sessions.delete",
    "cron.list",
    "cron.add",
    "cron.edit",
    "cron.rm",
    "cron.run",
    "cron.enable",
    "cron.disable",
    "config.get",
    "config.set",
    "config.unset",
    "config.validate",
    "approvals.list",
    "approvals.approve",
    "approvals.deny",
    "plugins.list",
    "plugins.enable",
    "plugins.disable",
    "system.event",
    "system.heartbeat",
    "system.presence",
    "system.restart",
    // Back-compat methods still exposed.
    "cron.remove",
    "cron.pause",
    "cron.resume",
    "cron.status",
    "cron.runs",
    "cron.deadLetters",
    "sessions.preview",
    "sessions.show",
    "approvals.get",
    "approvals.set",
    "approvals.allowlist.add",
    "approvals.allowlist.remove",
    "approvals.allowlist.list",
    "pairing.list",
    "pairing.approve",
    "pairing.reject",
    "plugins.get",
    "chat.send",
    "acp.sessions.list",
    "acp.spawn",
    "acp.cleanup",
    "security.audit",
];

pub fn parse_or_default<T>(params: Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let normalized = if params.is_null() {
        Value::Object(serde_json::Map::new())
    } else {
        params
    };

    serde_json::from_value(normalized).map_err(|err| err.to_string())
}

pub fn parse_strict<T>(params: Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    serde_json::from_value(params).map_err(|err| err.to_string())
}

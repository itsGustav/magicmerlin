use serde::de::DeserializeOwned;
use serde_json::Value;

pub mod agent_run;
pub mod agents;
pub mod approvals;
pub mod browser;
pub mod channels;
pub mod config;
pub mod cron;
pub mod directory;
pub mod health;
pub mod hooks;
pub mod logs;
pub mod memory;
pub mod models;
pub mod nodes;
pub mod plugins;
pub mod run;
pub mod sandbox;
pub mod sessions;
pub mod skills;
pub mod status;
pub mod system;

pub const SUPPORTED_METHODS: &[&str] = &[
    // Core
    "health",
    "status",
    "system-presence",
    // Agent execution
    "agent.run",
    "agent.abort",
    // Sessions
    "sessions.list",
    "sessions.get",
    "sessions.send",
    "sessions.spawn",
    "sessions.compact",
    "sessions.delete",
    "sessions.history",
    "sessions.export",
    // Cron
    "cron.list",
    "cron.add",
    "cron.edit",
    "cron.rm",
    "cron.run",
    "cron.enable",
    "cron.disable",
    // Config
    "config.get",
    "config.set",
    "config.unset",
    "config.validate",
    "config.list",
    "config.export",
    "config.import",
    // Approvals
    "approvals.list",
    "approvals.approve",
    "approvals.deny",
    // Plugins
    "plugins.list",
    "plugins.enable",
    "plugins.disable",
    "plugins.install",
    // System
    "system.event",
    "system.heartbeat",
    "system.presence",
    "system.restart",
    "system.info",
    "system.env",
    // Memory
    "memory.search",
    "memory.get",
    "memory.list",
    // Models
    "models.list",
    "models.set",
    "models.test",
    "models.status",
    // Channels
    "channels.list",
    "channels.status",
    "channels.login",
    "channels.logout",
    "channels.restart",
    "channels.send",
    // Hooks
    "hooks.list",
    "hooks.add",
    "hooks.remove",
    "hooks.test",
    // Logs
    "logs.tail",
    "logs.query",
    // Run queue
    "run.list",
    "run.status",
    // Agents management
    "agents.list",
    "agents.get",
    "agents.add",
    "agents.remove",
    "agents.config",
    // Skills
    "skills.list",
    "skills.get",
    // Directory
    "directory.search",
    "directory.get",
    // Nodes
    "nodes.list",
    "nodes.describe",
    "nodes.run",
    "nodes.invoke",
    // Sandbox
    "sandbox.list",
    "sandbox.start",
    "sandbox.stop",
    "sandbox.status",
    "sandbox.exec",
    // Browser
    "browser.start",
    "browser.stop",
    "browser.status",
    "browser.navigate",
    "browser.screenshot",
    "browser.act",
    "browser.snapshot",
    // Channels extended
    "channels.react",
    "channels.delete",
    // Nodes extended
    "nodes.notify",
    "nodes.location_get",
    "nodes.screen_record",
    "nodes.camera_snap",
    // Browser extended
    "browser.tabs",
    "browser.open",
    // Subagents
    "subagents.list",
    "subagents.steer",
    "subagents.kill",
    // Gateway control
    "gateway.status",
    "gateway.restart",
    "gateway.config.get",
    "gateway.config.patch",
    // Approvals extended
    "approvals.pending",
    // Sessions extended
    "sessions.yield",
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

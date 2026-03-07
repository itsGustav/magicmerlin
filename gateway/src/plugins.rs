use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRegistry {
    pub plugins: Vec<Plugin>,
}

/// Resolve the path to plugins.json, relative to the gateway binary's directory
/// or falling back to the current working directory.
pub fn registry_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAGICMERLIN_PLUGINS_FILE") {
        return PathBuf::from(p);
    }

    if let Ok(exe) = std::env::current_exe() {
        let candidate = exe.parent().unwrap_or(Path::new(".")).join("plugins.json");
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from("plugins.json")
}

pub fn load_registry() -> Result<PluginRegistry> {
    let path = registry_path();
    if !path.exists() {
        return Ok(PluginRegistry {
            plugins: Vec::new(),
        });
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
    let reg: PluginRegistry =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("parse {}: {e}", path.display()))?;
    Ok(reg)
}

pub fn save_registry(registry: &PluginRegistry) -> Result<()> {
    let path = registry_path();
    let body = serde_json::to_string_pretty(registry)?;
    std::fs::write(&path, format!("{body}\n"))
        .map_err(|e| anyhow::anyhow!("write {}: {e}", path.display()))
}

pub fn set_plugin_enabled(name: &str, enabled: bool) -> Result<bool> {
    let mut registry = load_registry()?;
    let mut changed = false;
    for plugin in &mut registry.plugins {
        if plugin.name == name {
            plugin.enabled = Some(enabled);
            changed = true;
            break;
        }
    }
    if changed {
        save_registry(&registry)?;
    }
    Ok(changed)
}

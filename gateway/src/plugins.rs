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
    // Check env override first.
    if let Ok(p) = std::env::var("MAGICMERLIN_PLUGINS_FILE") {
        return PathBuf::from(p);
    }

    // Try alongside the binary.
    if let Ok(exe) = std::env::current_exe() {
        let candidate = exe.parent().unwrap_or(Path::new(".")).join("plugins.json");
        if candidate.exists() {
            return candidate;
        }
    }

    // Fallback: cwd-relative (typical dev workflow).
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

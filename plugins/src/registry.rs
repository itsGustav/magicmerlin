use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::plugin::bundled_plugins;

/// Manifest definition loaded from plugin metadata files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Plugin identifier.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Human-readable summary.
    pub description: String,
}

/// Serializable plugin info returned by list operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    /// Plugin identifier.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Human-readable summary.
    pub description: String,
    /// Whether plugin is enabled.
    pub enabled: bool,
    /// Config namespace tied to plugin identity.
    pub namespace: String,
    /// Source of the plugin (`bundled` or `manifest`).
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginRegistryFileEntry {
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default)]
    namespace_config: Map<String, Value>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PluginRegistryFile {
    #[serde(default)]
    plugins: Vec<PluginRegistryFileEntry>,
}

/// Full plugin registry with metadata and isolated namespace config maps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginRegistry {
    /// Effective plugin list.
    pub plugins: Vec<PluginInfo>,
    /// Config maps keyed by plugin namespace.
    #[serde(default)]
    pub namespace_config: BTreeMap<String, Map<String, Value>>,
}

/// Resolves plugin registry file location.
pub fn registry_path() -> PathBuf {
    if let Ok(path) = std::env::var("MAGICMERLIN_PLUGINS_FILE") {
        return PathBuf::from(path);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("plugins.json");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from("plugins.json")
}

/// Scans plugin directories and reads `plugin.json` manifests.
pub fn discover_plugin_manifests(roots: &[PathBuf]) -> Result<Vec<PluginManifest>> {
    let mut manifests = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }
        let entries = fs::read_dir(root)
            .with_context(|| format!("read plugin root {}", root.display()))?;
        for entry in entries {
            let entry = entry.with_context(|| format!("read entry in {}", root.display()))?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    manifests.push(read_manifest(&manifest_path)?);
                }
            } else if path.file_name().is_some_and(|name| name == "plugin.json") {
                manifests.push(read_manifest(&path)?);
            }
        }
    }

    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    manifests.dedup_by(|a, b| a.name == b.name);
    Ok(manifests)
}

fn read_manifest(path: &Path) -> Result<PluginManifest> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let manifest = serde_json::from_str::<PluginManifest>(&body)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(manifest)
}

/// Loads plugin registry and merges file state with bundled and discovered plugins.
pub fn load_registry() -> Result<PluginRegistry> {
    let file_path = registry_path();
    let file_registry = if file_path.exists() {
        let raw = fs::read_to_string(&file_path)
            .with_context(|| format!("read {}", file_path.display()))?;
        serde_json::from_str::<PluginRegistryFile>(&raw)
            .with_context(|| format!("parse {}", file_path.display()))?
    } else {
        PluginRegistryFile::default()
    };

    let mut info_by_name: BTreeMap<String, PluginInfo> = BTreeMap::new();
    let mut config_by_namespace: BTreeMap<String, Map<String, Value>> = BTreeMap::new();

    for plugin in bundled_plugins() {
        let name = plugin.name().to_string();
        info_by_name.insert(
            name.clone(),
            PluginInfo {
                name: name.clone(),
                version: plugin.version().to_string(),
                description: plugin.description().to_string(),
                enabled: true,
                namespace: name,
                source: "bundled".to_string(),
            },
        );
    }

    let default_roots = default_plugin_roots();
    for manifest in discover_plugin_manifests(&default_roots)? {
        info_by_name.entry(manifest.name.clone()).or_insert(PluginInfo {
            name: manifest.name.clone(),
            version: manifest.version,
            description: manifest.description,
            enabled: true,
            namespace: manifest.name,
            source: "manifest".to_string(),
        });
    }

    for entry in file_registry.plugins {
        let namespace = entry.name.clone();
        if !entry.namespace_config.is_empty() {
            config_by_namespace.insert(namespace.clone(), entry.namespace_config);
        }
        let plugin = info_by_name.entry(entry.name.clone()).or_insert(PluginInfo {
            name: entry.name.clone(),
            version: entry.version.clone().unwrap_or_else(|| "0.0.0".to_string()),
            description: entry.description.clone().unwrap_or_default(),
            enabled: entry.enabled,
            namespace: namespace.clone(),
            source: "manifest".to_string(),
        });

        plugin.enabled = entry.enabled;
        if let Some(version) = entry.version {
            plugin.version = version;
        }
        if let Some(description) = entry.description {
            plugin.description = description;
        }
        plugin.namespace = namespace;
    }

    Ok(PluginRegistry {
        plugins: info_by_name.into_values().collect(),
        namespace_config: config_by_namespace,
    })
}

/// Persists plugin registry enablement and namespace config to disk.
pub fn save_registry(registry: &PluginRegistry) -> Result<()> {
    let file_path = registry_path();
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }

    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();
    for plugin in &registry.plugins {
        if !seen.insert(plugin.name.clone()) {
            continue;
        }
        let config = registry
            .namespace_config
            .get(&plugin.namespace)
            .cloned()
            .unwrap_or_default();
        entries.push(PluginRegistryFileEntry {
            name: plugin.name.clone(),
            version: Some(plugin.version.clone()),
            description: Some(plugin.description.clone()),
            enabled: plugin.enabled,
            namespace_config: config,
        });
    }

    let file = PluginRegistryFile { plugins: entries };
    let body = serde_json::to_string_pretty(&file)?;
    fs::write(&file_path, format!("{body}\n"))
        .with_context(|| format!("write {}", file_path.display()))?;
    Ok(())
}

/// Enables or disables a plugin by name and persists the registry.
pub fn set_plugin_enabled(name: &str, enabled: bool) -> Result<bool> {
    let mut registry = load_registry()?;
    let mut changed = false;
    for plugin in &mut registry.plugins {
        if plugin.name == name {
            if plugin.enabled != enabled {
                plugin.enabled = enabled;
                changed = true;
            }
            break;
        }
    }

    if changed {
        save_registry(&registry)?;
    }

    Ok(changed)
}

fn default_plugin_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(state_dir) = std::env::var("OPENCLAW_STATE_DIR") {
        roots.push(PathBuf::from(state_dir).join("plugins"));
    }

    if let Ok(home) = std::env::var("HOME") {
        roots.push(PathBuf::from(home).join(".openclaw/plugins"));
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn discovers_manifest_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let plugin_dir = temp.path().join("example-plugin");
        fs::create_dir_all(&plugin_dir).expect("plugin dir");
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{"name":"example","version":"1.2.3","description":"desc"}"#,
        )
        .expect("manifest write");

        let manifests = discover_plugin_manifests(&[temp.path().to_path_buf()]).expect("discover");
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].name, "example");
    }

    #[test]
    fn default_registry_contains_bundled_plugins() {
        let _guard = env_lock().lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        std::env::set_var("MAGICMERLIN_PLUGINS_FILE", temp.path().join("plugins.json"));
        let registry = load_registry().expect("load");
        assert!(registry.plugins.iter().any(|p| p.name == "session-memory"));
        std::env::remove_var("MAGICMERLIN_PLUGINS_FILE");
    }

    #[test]
    fn set_plugin_enabled_persists() {
        let _guard = env_lock().lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let file = temp.path().join("plugins.json");
        std::env::set_var("MAGICMERLIN_PLUGINS_FILE", &file);

        let mut registry = load_registry().expect("load");
        registry.plugins.push(PluginInfo {
            name: "custom".to_string(),
            version: "0.1.0".to_string(),
            description: "custom".to_string(),
            enabled: true,
            namespace: "custom".to_string(),
            source: "manifest".to_string(),
        });
        save_registry(&registry).expect("save");

        let changed = set_plugin_enabled("custom", false).expect("set");
        assert!(changed);

        let updated = load_registry().expect("reload");
        let custom = updated
            .plugins
            .iter()
            .find(|p| p.name == "custom")
            .expect("custom plugin");
        assert!(!custom.enabled);
        std::env::remove_var("MAGICMERLIN_PLUGINS_FILE");
    }

    #[test]
    fn registry_path_honors_env() {
        let _guard = env_lock().lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let file = temp.path().join("custom.json");
        std::env::set_var("MAGICMERLIN_PLUGINS_FILE", &file);
        assert_eq!(registry_path(), file);
        std::env::remove_var("MAGICMERLIN_PLUGINS_FILE");
    }
}

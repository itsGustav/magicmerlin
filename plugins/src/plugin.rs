use std::collections::BTreeMap;

use serde_json::{Map, Value};
use thiserror::Error;

use crate::registry::PluginInfo;

/// Context passed to plugins so each plugin only receives its own configuration namespace.
#[derive(Debug, Clone, Default)]
pub struct PluginContext {
    /// Plugin-scoped configuration values.
    pub config: Map<String, Value>,
}

/// Runtime lifecycle state for a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is loaded but not initialized.
    Registered,
    /// Plugin has initialized resources.
    Initialized,
    /// Plugin is actively running.
    Running,
    /// Plugin has been stopped.
    Stopped,
}

/// Error type for plugin lifecycle operations.
#[derive(Debug, Error)]
pub enum PluginLifecycleError {
    /// Operation is invalid for current lifecycle state.
    #[error("invalid state transition for plugin {plugin}: {from:?} -> {to:?}")]
    InvalidTransition {
        /// Plugin name.
        plugin: String,
        /// Current plugin state.
        from: PluginState,
        /// Requested plugin state.
        to: PluginState,
    },
    /// Plugin execution callback failed.
    #[error("plugin {plugin} callback failed: {message}")]
    Callback {
        /// Plugin name.
        plugin: String,
        /// Error message.
        message: String,
    },
}

/// Trait for all plugins.
pub trait Plugin: Send {
    /// Returns the plugin name.
    fn name(&self) -> &str;

    /// Returns plugin version.
    fn version(&self) -> &str;

    /// Returns plugin description.
    fn description(&self) -> &str;

    /// Initializes plugin resources.
    fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginLifecycleError> {
        Ok(())
    }

    /// Starts plugin processing.
    fn start(&mut self, _ctx: &PluginContext) -> Result<(), PluginLifecycleError> {
        Ok(())
    }

    /// Stops plugin processing.
    fn stop(&mut self, _ctx: &PluginContext) -> Result<(), PluginLifecycleError> {
        Ok(())
    }
}

/// Minimal built-in plugin implementation used for bundled plugin entries.
#[derive(Debug, Clone)]
pub struct BuiltinPlugin {
    name: String,
    version: String,
    description: String,
}

impl BuiltinPlugin {
    /// Creates a new built-in plugin with static metadata.
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
        }
    }
}

impl Plugin for BuiltinPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }
}

struct RuntimeEntry {
    plugin: Box<dyn Plugin>,
    state: PluginState,
    enabled: bool,
    context: PluginContext,
}

/// In-process runtime to manage plugin lifecycle transitions.
pub struct PluginRuntime {
    entries: BTreeMap<String, RuntimeEntry>,
}

impl PluginRuntime {
    /// Creates an empty plugin runtime.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Builds runtime with bundled plugins pre-registered.
    pub fn with_bundled_plugins() -> Self {
        let mut runtime = Self::new();
        for plugin in bundled_plugins() {
            runtime.register(plugin, PluginContext::default(), true);
        }
        runtime
    }

    /// Registers a plugin into the runtime.
    pub fn register(&mut self, plugin: Box<dyn Plugin>, context: PluginContext, enabled: bool) {
        let key = plugin.name().to_string();
        let entry = RuntimeEntry {
            plugin,
            state: PluginState::Registered,
            enabled,
            context,
        };
        self.entries.insert(key, entry);
    }

    /// Initializes all enabled plugins.
    pub fn init_enabled(&mut self) -> Result<(), PluginLifecycleError> {
        for entry in self.entries.values_mut().filter(|e| e.enabled) {
            if entry.state != PluginState::Registered {
                return Err(PluginLifecycleError::InvalidTransition {
                    plugin: entry.plugin.name().to_string(),
                    from: entry.state,
                    to: PluginState::Initialized,
                });
            }
            entry.plugin.init(&entry.context)?;
            entry.state = PluginState::Initialized;
        }
        Ok(())
    }

    /// Starts all initialized plugins.
    pub fn start_enabled(&mut self) -> Result<(), PluginLifecycleError> {
        for entry in self.entries.values_mut().filter(|e| e.enabled) {
            if entry.state != PluginState::Initialized {
                return Err(PluginLifecycleError::InvalidTransition {
                    plugin: entry.plugin.name().to_string(),
                    from: entry.state,
                    to: PluginState::Running,
                });
            }
            entry.plugin.start(&entry.context)?;
            entry.state = PluginState::Running;
        }
        Ok(())
    }

    /// Stops all running plugins.
    pub fn stop_enabled(&mut self) -> Result<(), PluginLifecycleError> {
        for entry in self.entries.values_mut().filter(|e| e.enabled) {
            if entry.state != PluginState::Running {
                return Err(PluginLifecycleError::InvalidTransition {
                    plugin: entry.plugin.name().to_string(),
                    from: entry.state,
                    to: PluginState::Stopped,
                });
            }
            entry.plugin.stop(&entry.context)?;
            entry.state = PluginState::Stopped;
        }
        Ok(())
    }

    /// Returns metadata for registered plugins.
    pub fn list(&self) -> Vec<PluginInfo> {
        self.entries
            .values()
            .map(|entry| PluginInfo {
                name: entry.plugin.name().to_string(),
                version: entry.plugin.version().to_string(),
                description: entry.plugin.description().to_string(),
                enabled: entry.enabled,
                namespace: entry.plugin.name().to_string(),
                source: "bundled".to_string(),
            })
            .collect()
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns bundled plugins required by the core gateway runtime.
pub fn bundled_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(BuiltinPlugin::new(
            "session-memory",
            "0.1.0",
            "Auto-save session context to memory files",
        )),
        Box::new(BuiltinPlugin::new(
            "command-logger",
            "0.1.0",
            "Log all commands to file",
        )),
        Box::new(BuiltinPlugin::new(
            "boot-md",
            "0.1.0",
            "Load AGENTS.md/SOUL.md files at session start",
        )),
        Box::new(BuiltinPlugin::new(
            "bootstrap-extra-files",
            "0.1.0",
            "Load additional workspace files before agent start",
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        version: String,
        description: String,
        steps: Vec<&'static str>,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: "test".to_string(),
                steps: Vec::new(),
            }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginLifecycleError> {
            self.steps.push("init");
            Ok(())
        }

        fn start(&mut self, _ctx: &PluginContext) -> Result<(), PluginLifecycleError> {
            self.steps.push("start");
            Ok(())
        }

        fn stop(&mut self, _ctx: &PluginContext) -> Result<(), PluginLifecycleError> {
            self.steps.push("stop");
            Ok(())
        }
    }

    #[test]
    fn runtime_lifecycle_transitions() {
        let mut runtime = PluginRuntime::new();
        runtime.register(
            Box::new(TestPlugin::new("t")),
            PluginContext::default(),
            true,
        );

        assert!(runtime.init_enabled().is_ok());
        assert!(runtime.start_enabled().is_ok());
        assert!(runtime.stop_enabled().is_ok());
    }

    #[test]
    fn bundled_plugins_include_required_names() {
        let names: Vec<String> = bundled_plugins()
            .into_iter()
            .map(|p| p.name().to_string())
            .collect();
        assert!(names.contains(&"session-memory".to_string()));
        assert!(names.contains(&"command-logger".to_string()));
        assert!(names.contains(&"boot-md".to_string()));
        assert!(names.contains(&"bootstrap-extra-files".to_string()));
    }

    #[test]
    fn with_bundled_plugins_lists_entries() {
        let runtime = PluginRuntime::with_bundled_plugins();
        assert!(runtime.list().len() >= 4);
    }
}

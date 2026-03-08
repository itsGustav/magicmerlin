use anyhow::Result;
pub use magicmerlin_plugins::registry::PluginRegistry;

/// Loads plugin registry from the canonical plugin subsystem.
pub fn load_registry() -> Result<PluginRegistry> {
    magicmerlin_plugins::load_registry()
}

/// Sets plugin enabled state and persists registry.
pub fn set_plugin_enabled(name: &str, enabled: bool) -> Result<bool> {
    magicmerlin_plugins::set_plugin_enabled(name, enabled)
}

//! Plugin and skill runtime for MagicMerlin.

pub mod plugin;
pub mod registry;
pub mod skills;

pub use plugin::{
    BuiltinPlugin, Plugin, PluginContext, PluginLifecycleError, PluginRuntime, PluginState,
};
pub use registry::{
    discover_plugin_manifests, load_registry, registry_path, save_registry, set_plugin_enabled,
    PluginInfo, PluginManifest, PluginRegistry,
};
pub use skills::{
    check_skill_dependencies, discover_skills, execute_skill_script, skills_xml_block,
    DependencyReport, Skill, SkillDependencyStatus,
};

use std::fs;
use std::sync::{Mutex, OnceLock};

use magicmerlin_config::{ConfigManager, ConfigOptions, PathScope, StatePaths};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn loads_with_env_overlay_and_dev_port() {
    let _guard = env_lock().lock().expect("lock");

    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("openclaw.json");
    fs::write(
        &cfg,
        r#"{"gateway":{"port":18789},"agents":{"defaults":{"model":"gpt-5"}}}"#,
    )
    .expect("write config");

    std::env::set_var("OPENCLAW_CONFIG_PATH", &cfg);
    std::env::set_var("OPENCLAW_STATE_DIR", temp.path().join("state"));
    std::env::set_var("OPENCLAW_GATEWAY_PORT", "18888");

    let mgr = ConfigManager::load(ConfigOptions {
        profile: None,
        dev: false,
    })
    .expect("load");
    assert_eq!(
        mgr.get("gateway.port").and_then(|v| v.as_u64()),
        Some(18888)
    );

    let dev_mgr = ConfigManager::load(ConfigOptions {
        profile: None,
        dev: true,
    })
    .expect("dev load");
    assert_eq!(
        dev_mgr.get("gateway.port").and_then(|v| v.as_u64()),
        Some(19001)
    );

    std::env::remove_var("OPENCLAW_GATEWAY_PORT");
    std::env::remove_var("OPENCLAW_CONFIG_PATH");
    std::env::remove_var("OPENCLAW_STATE_DIR");
}

#[test]
fn rejects_unknown_top_level_keys() {
    let _guard = env_lock().lock().expect("lock");

    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("openclaw.json");
    fs::write(&cfg, r#"{"gateway":{"port":18789},"oops":true}"#).expect("write config");

    std::env::set_var("OPENCLAW_CONFIG_PATH", &cfg);
    std::env::set_var("OPENCLAW_STATE_DIR", temp.path().join("state"));
    let result = ConfigManager::load(ConfigOptions::default());
    assert!(result.is_err());
    std::env::remove_var("OPENCLAW_CONFIG_PATH");
    std::env::remove_var("OPENCLAW_STATE_DIR");
}

#[test]
fn state_paths_profile_and_default() {
    let _guard = env_lock().lock().expect("lock");

    let temp = tempfile::tempdir().expect("tempdir");
    std::env::set_var("OPENCLAW_STATE_DIR", temp.path().join("root"));

    let default_paths = StatePaths::new(PathScope::Default).expect("default");
    assert!(default_paths.state_dir.ends_with("root"));

    let prof_paths = StatePaths::new(PathScope::profile("work".to_string())).expect("profile");
    assert!(prof_paths.state_dir.ends_with("root"));

    std::env::remove_var("OPENCLAW_STATE_DIR");
}

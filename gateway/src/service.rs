use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Writes the current process ID to the given PID file path.
pub fn write_pid_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create pid parent dir: {}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", std::process::id()))
        .with_context(|| format!("write pid file: {}", path.display()))
}

/// Removes a PID file if it exists.
pub fn remove_pid_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("remove pid file: {}", path.display())),
    }
}

/// Returns the default gateway PID file location under the state directory.
pub fn default_pid_file(state_dir: &Path) -> PathBuf {
    state_dir.join("gateway").join("gateway.pid")
}

/// Generates a macOS LaunchAgent plist content.
pub fn generate_launchagent_plist(gateway_bin: &Path, state_dir: &Path, port: u16) -> String {
    let log_dir = state_dir.join("logs");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key><string>ai.magicmerlin.gateway</string>\n  <key>ProgramArguments</key>\n  <array>\n    <string>{}</string>\n    <string>--serve</string>\n    <string>{}</string>\n    <string>--daemon</string>\n  </array>\n  <key>RunAtLoad</key><true/>\n  <key>KeepAlive</key><true/>\n  <key>StandardOutPath</key><string>{}</string>\n  <key>StandardErrorPath</key><string>{}</string>\n</dict>\n</plist>\n",
        gateway_bin.display(),
        port,
        log_dir.join("gateway.launchd.out.log").display(),
        log_dir.join("gateway.launchd.err.log").display(),
    )
}

/// Installs a LaunchAgent plist into `~/Library/LaunchAgents`.
pub fn install_launchagent(plist_body: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME is not set")?;
    let path = PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("ai.magicmerlin.gateway.plist");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create launchagent dir: {}", parent.display()))?;
    }
    fs::write(&path, plist_body)
        .with_context(|| format!("write launchagent plist: {}", path.display()))?;
    Ok(path)
}

/// Removes installed LaunchAgent plist.
pub fn uninstall_launchagent() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME is not set")?;
    let path = PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("ai.magicmerlin.gateway.plist");
    remove_pid_file(&path)?;
    Ok(path)
}

/// Generates a systemd user unit for the gateway.
pub fn generate_systemd_unit(gateway_bin: &Path, port: u16) -> String {
    format!(
        "[Unit]\nDescription=MagicMerlin Gateway\nAfter=network.target\n\n[Service]\nExecStart={} --serve {} --daemon\nRestart=always\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n",
        gateway_bin.display(),
        port,
    )
}

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

/// Reads a pid file and parses the process id.
pub fn read_pid_file(path: &Path) -> Result<Option<u32>> {
    let body = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("read pid file: {}", path.display())),
    };

    let pid = body
        .trim()
        .parse::<u32>()
        .with_context(|| format!("invalid pid file content: {}", path.display()))?;
    Ok(Some(pid))
}

/// Returns true if a pid appears to be alive.
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let status = std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        // Conservative fallback on non-unix systems in this workspace.
        let _ = pid;
        false
    }
}

/// Removes stale pid file (non-running process), returning true if removed.
pub fn remove_stale_pid_file(path: &Path) -> Result<bool> {
    let Some(pid) = read_pid_file(path)? else {
        return Ok(false);
    };

    if is_process_running(pid) {
        return Ok(false);
    }

    remove_pid_file(path)?;
    Ok(true)
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

/// Build launchctl load command arguments.
#[allow(dead_code)]
pub fn launchctl_load_args(plist_path: &Path) -> Vec<String> {
    vec![
        "bootstrap".to_string(),
        format!("gui/{}", nix_uid()),
        plist_path.display().to_string(),
    ]
}

/// Build launchctl unload command arguments.
#[allow(dead_code)]
pub fn launchctl_unload_args(plist_path: &Path) -> Vec<String> {
    vec![
        "bootout".to_string(),
        format!("gui/{}", nix_uid()),
        plist_path.display().to_string(),
    ]
}

/// Build systemctl args for user-service operations.
#[allow(dead_code)]
pub fn systemctl_user_args(action: &str, unit_name: &str) -> Vec<String> {
    vec![
        "--user".to_string(),
        action.to_string(),
        unit_name.to_string(),
    ]
}

#[allow(dead_code)]
fn nix_uid() -> u32 {
    #[cfg(unix)]
    {
        // Portable enough for this crate without extra deps.
        if let Ok(output) = std::process::Command::new("id").arg("-u").output() {
            if output.status.success() {
                if let Ok(uid) = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u32>()
                {
                    return uid;
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_unit_has_execstart() {
        let body = generate_systemd_unit(Path::new("/usr/bin/gateway"), 18789);
        assert!(body.contains("ExecStart=/usr/bin/gateway --serve 18789 --daemon"));
    }

    #[test]
    fn launchctl_args_include_plist_path() {
        let path = Path::new("/tmp/test.plist");
        let args = launchctl_load_args(path);
        assert_eq!(args[0], "bootstrap");
        assert_eq!(args[2], "/tmp/test.plist");
    }

    #[test]
    fn stale_pid_is_removed() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pid_path = temp.path().join("gateway.pid");
        fs::write(&pid_path, "999999\n").expect("write pid");

        let removed = remove_stale_pid_file(&pid_path).expect("remove stale");
        assert!(removed);
        assert!(!pid_path.exists());
    }
}

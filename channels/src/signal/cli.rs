//! Signal-cli subprocess wrapper.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::framework::{ChannelError, InboundMessage, Result};

use super::types::SignalEnvelope;

/// Subprocess wrapper around the `signal-cli` binary.
#[derive(Debug)]
pub struct SignalCliWrapper {
    /// Path to the signal-cli binary.
    pub binary: PathBuf,
    /// Registered Signal phone number.
    pub account: String,
    /// Optional config/data directory override.
    pub data_dir: Option<PathBuf>,
}

impl SignalCliWrapper {
    /// Creates a new wrapper for the given binary and account.
    pub fn new(binary: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            binary: PathBuf::from(binary.into()),
            account: account.into(),
            data_dir: None,
        }
    }

    /// Sets the config/data directory for signal-cli.
    pub fn with_data_dir(mut self, dir: PathBuf) -> Self {
        self.data_dir = Some(dir);
        self
    }

    /// Builds the base command with account and optional config dir.
    fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-u").arg(&self.account);
        if let Some(dir) = &self.data_dir {
            cmd.arg("--config").arg(dir);
        }
        cmd
    }

    /// Sends a text message to a phone number.
    pub async fn send(&self, recipient: &str, text: &str) -> Result<()> {
        let output = self
            .base_command()
            .args(["send", "-m", text, recipient])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                ChannelError::PlatformRequest(format!("signal-cli send failed to spawn: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "signal-cli send exited with {}: {stderr}",
                output.status
            )));
        }
        Ok(())
    }

    /// Sends a text message to a group.
    pub async fn send_to_group(&self, group_id: &str, text: &str) -> Result<()> {
        let output = self
            .base_command()
            .args(["send", "-m", text, "-g", group_id])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                ChannelError::PlatformRequest(format!("signal-cli group send failed to spawn: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "signal-cli group send exited with {}: {stderr}",
                output.status
            )));
        }
        Ok(())
    }

    /// Sends a message with a file attachment to a phone number.
    pub async fn send_with_attachment(
        &self,
        recipient: &str,
        text: &str,
        attachment: &Path,
    ) -> Result<()> {
        let output = self
            .base_command()
            .args(["send", "-m", text, "-a"])
            .arg(attachment)
            .arg(recipient)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                ChannelError::PlatformRequest(format!(
                    "signal-cli send+attachment failed to spawn: {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "signal-cli send+attachment exited with {}: {stderr}",
                output.status
            )));
        }
        Ok(())
    }

    /// Runs `signal-cli receive --output=json` once and returns parsed messages.
    ///
    /// The receive command processes pending messages and exits. For continuous
    /// polling, call this repeatedly with a sleep interval (handled by
    /// [`super::SignalChannel::run_receive_loop`]).
    pub async fn receive_once(&self) -> Result<Vec<InboundMessage>> {
        let output = self
            .base_command()
            .args(["receive", "--output=json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|e| {
                ChannelError::PlatformRequest(format!("signal-cli receive failed to spawn: {e}"))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut messages = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parsed: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // signal-cli wraps envelopes: {"envelope": {...}}
            let envelope_value = parsed.get("envelope").cloned().unwrap_or(parsed);

            let envelope: SignalEnvelope = match serde_json::from_value(envelope_value) {
                Ok(e) => e,
                Err(_) => continue,
            };

            if let Some(mut inbound) = envelope.into_inbound() {
                inbound.normalize();
                messages.push(inbound);
            }
        }

        Ok(messages)
    }

    /// Checks whether the signal-cli binary is reachable.
    pub fn is_available(&self) -> bool {
        if self.binary.is_absolute() {
            return self.binary.exists();
        }
        which_in_path(&self.binary)
    }

    /// Returns the command arguments that would be used for a send operation.
    pub fn build_send_args(&self, recipient: &str, text: &str) -> Vec<String> {
        let mut args = vec![
            self.binary.display().to_string(),
            "-u".to_string(),
            self.account.clone(),
        ];
        if let Some(dir) = &self.data_dir {
            args.push("--config".to_string());
            args.push(dir.display().to_string());
        }
        args.extend([
            "send".to_string(),
            "-m".to_string(),
            text.to_string(),
            recipient.to_string(),
        ]);
        args
    }
}

/// Checks whether a binary name can be found on `$PATH`.
fn which_in_path(binary: &Path) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(binary))
        .any(|p| p.exists())
}

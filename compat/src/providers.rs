//! Compat adapter interfaces + snapshot-backed providers.
//!
//! In v0.x, providers are backed purely by the captured snapshots.

use anyhow::Result;
use serde_json::Value;

use crate::snapshots::{find_repo_root, SnapshotBundle, SnapshotHashes};

pub trait CliProvider {
  fn openclaw_help_text(&self) -> &str;
  fn openclaw_cron_help_text(&self) -> &str;
}

pub trait StatusProvider {
  fn openclaw_status_json(&self) -> &Value;
}

pub trait CronProvider {
  fn cron_help_text(&self) -> &str;
}

pub trait ToolRegistryProvider {
  /// Names like `functions.read`.
  fn tool_names(&self) -> Vec<String>;

  /// Raw source snapshot.
  fn tool_surface_markdown(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct SnapshotBackedProviders {
  repo_root: std::path::PathBuf,
  snapshots: SnapshotBundle,
}

impl SnapshotBackedProviders {
  pub fn load() -> Result<Self> {
    let repo_root = find_repo_root()?;
    let snapshots = SnapshotBundle::load(&repo_root)?;
    Ok(Self { repo_root, snapshots })
  }

  pub fn snapshots(&self) -> &SnapshotBundle {
    &self.snapshots
  }

  pub fn hashes(&self) -> Result<SnapshotHashes> {
    self.snapshots.hashes(&self.repo_root)
  }
}

impl CliProvider for SnapshotBackedProviders {
  fn openclaw_help_text(&self) -> &str {
    &self.snapshots.openclaw_help
  }

  fn openclaw_cron_help_text(&self) -> &str {
    &self.snapshots.openclaw_cron_help
  }
}

impl StatusProvider for SnapshotBackedProviders {
  fn openclaw_status_json(&self) -> &Value {
    &self.snapshots.openclaw_status
  }
}

impl CronProvider for SnapshotBackedProviders {
  fn cron_help_text(&self) -> &str {
    &self.snapshots.openclaw_cron_help
  }
}

impl ToolRegistryProvider for SnapshotBackedProviders {
  fn tool_names(&self) -> Vec<String> {
    self.snapshots.tool_names()
  }

  fn tool_surface_markdown(&self) -> &str {
    &self.snapshots.runtime_tool_surface_md
  }
}

//! Snapshot handling.
//!
//! In Parity v0.x this module is **snapshot-backed**: we treat the files under
//! `compat/snapshots/` as the golden reference for OpenClaw’s surface.
//!
//! - Parsing is tolerant (unknown fields allowed)
//! - Drift is surfaced via stable sha256 hashes + a combined fingerprint

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Path (relative to repo root) where OpenClaw compatibility snapshots live.
pub const SNAPSHOT_DIR: &str = "magicmerlin/compat/snapshots";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotManifest {
    pub captured_at: String,
    pub openclaw_version: String,

    /// Optional: stable fingerprint (sha256 over sorted "name sha256\n" entries).
    pub fingerprint: Option<String>,

    /// Optional: sha256 hex by logical snapshot name (e.g. "openclawHelp").
    pub snapshot_hashes: Option<BTreeMap<String, String>>,

    pub files: SnapshotFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotFiles {
    pub openclaw_help: String,
    pub openclaw_cron_help: String,
    pub openclaw_status_json: String,
    pub openclaw_status_header: Option<String>,
    pub openclaw_version_txt: String,
    pub runtime_tool_surface_md: String,

    /// Optional full CLI help tree (all commands/subcommands) captured as JSON.
    pub openclaw_help_tree_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SnapshotBundle {
    pub manifest: SnapshotManifest,
    pub openclaw_help: String,
    pub openclaw_cron_help: String,
    pub openclaw_status: serde_json::Value,
    pub openclaw_status_header: Option<String>,
    pub openclaw_version_txt: String,
    pub runtime_tool_surface_md: String,

    pub openclaw_help_tree: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotHashes {
    /// sha256 hex by logical snapshot name.
    pub files: BTreeMap<String, String>,
    /// A single stable fingerprint: sha256 over sorted entries "name sha256\n".
    pub fingerprint: String,
}

/// Find repo root by walking up from CWD looking for `magicmerlin/Cargo.toml`.
pub fn find_repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("current_dir")?;
    for dir in cwd.ancestors() {
        if dir.join("magicmerlin").join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err(anyhow!(
    "Could not find repo root (expected to find magicmerlin/Cargo.toml in an ancestor of {cwd:?})"
  ))
}

pub fn load_manifest(repo_root: &Path) -> Result<SnapshotManifest> {
    let path = repo_root.join(SNAPSHOT_DIR).join("manifest.json");
    let raw = fs::read_to_string(&path).with_context(|| format!("read manifest: {path:?}"))?;
    let manifest: SnapshotManifest =
        serde_json::from_str(&raw).with_context(|| format!("parse manifest json: {path:?}"))?;
    Ok(manifest)
}

impl SnapshotBundle {
    pub fn load(repo_root: &Path) -> Result<Self> {
        let manifest = load_manifest(repo_root)?;
        let base = repo_root.join(SNAPSHOT_DIR);

        let openclaw_help = read_text(base.join(&manifest.files.openclaw_help))?;
        let openclaw_cron_help = read_text(base.join(&manifest.files.openclaw_cron_help))?;
        let openclaw_status_header = manifest
            .files
            .openclaw_status_header
            .as_ref()
            .map(|p| read_text(base.join(p)))
            .transpose()?;

        let openclaw_status_raw = read_text(base.join(&manifest.files.openclaw_status_json))?;
        let openclaw_status: serde_json::Value =
            serde_json::from_str(&openclaw_status_raw).context("parse openclaw_status.json")?;

        let openclaw_version_txt = read_text(base.join(&manifest.files.openclaw_version_txt))?;
        let runtime_tool_surface_md =
            read_text(base.join(&manifest.files.runtime_tool_surface_md))?;

        let openclaw_help_tree = manifest
            .files
            .openclaw_help_tree_json
            .as_ref()
            .map(|p| read_text(base.join(p)))
            .transpose()?
            .map(|raw| serde_json::from_str::<serde_json::Value>(&raw))
            .transpose()
            .context("parse openclaw_help_tree.json")?;

        Ok(Self {
            manifest,
            openclaw_help,
            openclaw_cron_help,
            openclaw_status,
            openclaw_status_header,
            openclaw_version_txt,
            runtime_tool_surface_md,
            openclaw_help_tree,
        })
    }

    pub fn hashes(&self, repo_root: &Path) -> Result<SnapshotHashes> {
        let base = repo_root.join(SNAPSHOT_DIR);

        let mut files: BTreeMap<String, String> = BTreeMap::new();

        let mut add = |name: &str, rel: &str| -> Result<()> {
            let sha = sha256_hex(&fs::read(base.join(rel)).with_context(|| format!("read {rel}"))?);
            files.insert(name.to_string(), sha);
            Ok(())
        };

        add("openclawHelp", &self.manifest.files.openclaw_help)?;
        add("openclawCronHelp", &self.manifest.files.openclaw_cron_help)?;
        add(
            "openclawStatusJson",
            &self.manifest.files.openclaw_status_json,
        )?;
        add(
            "openclawVersionTxt",
            &self.manifest.files.openclaw_version_txt,
        )?;
        add(
            "runtimeToolSurfaceMd",
            &self.manifest.files.runtime_tool_surface_md,
        )?;

        if let Some(p) = &self.manifest.files.openclaw_help_tree_json {
            add("openclawHelpTreeJson", p)?;
        }

        let mut hasher = Sha256::new();
        for (k, v) in &files {
            hasher.update(k.as_bytes());
            hasher.update(b" ");
            hasher.update(v.as_bytes());
            hasher.update(b"\n");
        }
        let fingerprint = hex::encode(hasher.finalize());

        Ok(SnapshotHashes { files, fingerprint })
    }

    pub fn tool_names(&self) -> Vec<String> {
        // Very simple parser for the current markdown snapshot format.
        // Example line:
        // - `functions.read` — read file contents...
        let mut out = Vec::new();
        for line in self.runtime_tool_surface_md.lines() {
            let line = line.trim();
            if !line.starts_with("- `") {
                continue;
            }
            if let Some(rest) = line.strip_prefix("- `") {
                if let Some(end) = rest.find('`') {
                    out.push(rest[..end].to_string());
                }
            }
        }
        out
    }
}

fn read_text(path: PathBuf) -> Result<String> {
    fs::read_to_string(&path).with_context(|| format!("read text file: {path:?}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

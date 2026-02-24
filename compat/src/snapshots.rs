//! Snapshot handling (stub).
//!
//! In Parity v0 this module is *data-only*: MagicMerlin reads the snapshot files
//! under `compat/snapshots/` and uses them as a golden reference.
//!
//! Parsing + validation comes in later milestones.

/// Path (relative to repo root) where OpenClaw compatibility snapshots live.
pub const SNAPSHOT_DIR: &str = "magicmerlin/compat/snapshots";

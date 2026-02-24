//! OpenClaw compatibility layer.
//!
//! This crate is the *first* thing we build: it defines and enforces the
//! compatibility contract between MagicMerlin and OpenClaw.

pub mod snapshots;

/// Current compatibility version (human-managed).
///
/// Bump only when the compatibility contract changes.
pub const COMPAT_VERSION: &str = "v0";

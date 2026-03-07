//! HEARTBEAT.md loading and execution planning.

use std::fs;
use std::path::Path;

use crate::error::{AgentError, Result};

/// Heartbeat execution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatOutcome {
    /// No actionable lines found.
    Ok,
    /// Parsed task lines from heartbeat file.
    Tasks(Vec<String>),
}

/// Loads and interprets `HEARTBEAT.md` in agent directory.
pub fn run_heartbeat(agent_dir: impl AsRef<Path>) -> Result<HeartbeatOutcome> {
    let path = agent_dir.as_ref().join("HEARTBEAT.md");
    if !path.exists() {
        return Ok(HeartbeatOutcome::Ok);
    }

    let content = fs::read_to_string(&path).map_err(|source| AgentError::Io {
        path: path.clone(),
        source,
    })?;

    let tasks = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();

    if tasks.is_empty() {
        return Ok(HeartbeatOutcome::Ok);
    }

    Ok(HeartbeatOutcome::Tasks(tasks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_heartbeat_returns_ok() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("HEARTBEAT.md"), "# noop\n\n").expect("write");
        assert_eq!(
            run_heartbeat(temp.path()).expect("heartbeat"),
            HeartbeatOutcome::Ok
        );
    }
}

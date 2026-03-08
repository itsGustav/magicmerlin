//! HEARTBEAT.md loading and execution planning.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};

use crate::error::{AgentError, Result};

/// Parsed heartbeat task from markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatTask {
    /// Task description.
    pub text: String,
    /// Task category (best effort).
    pub category: String,
}

/// Heartbeat execution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatOutcome {
    /// No actionable lines found.
    Ok,
    /// Parsed task lines from heartbeat file.
    Tasks(Vec<String>),
}

/// Structured heartbeat run result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatRunResult {
    /// Whether quiet hours blocked execution.
    pub quiet_hours: bool,
    /// Tasks selected to run now.
    pub tasks_to_run: Vec<HeartbeatTask>,
    /// Human-readable status.
    pub status: String,
}

/// Tracks heartbeat timestamps per category.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeartbeatState {
    /// Last check timestamp per category.
    #[serde(default)]
    pub last_check_epoch_by_category: BTreeMap<String, i64>,
}

/// Loads and interprets `HEARTBEAT.md` in agent directory.
pub fn run_heartbeat(agent_dir: impl AsRef<Path>) -> Result<HeartbeatOutcome> {
    let tasks = parse_heartbeat_tasks(agent_dir.as_ref())?;
    if tasks.is_empty() {
        return Ok(HeartbeatOutcome::Ok);
    }

    Ok(HeartbeatOutcome::Tasks(
        tasks.into_iter().map(|task| task.text).collect(),
    ))
}

/// Executes heartbeat planning with quiet-hours awareness.
pub fn run_heartbeat_with_state(
    agent_dir: impl AsRef<Path>,
    quiet_start_hour: Option<u32>,
    quiet_end_hour: Option<u32>,
    state: &mut HeartbeatState,
) -> Result<HeartbeatRunResult> {
    let now = Local::now();
    let hour = now.hour();
    let in_quiet_hours = is_in_quiet_hours(hour, quiet_start_hour, quiet_end_hour);

    if in_quiet_hours {
        return Ok(HeartbeatRunResult {
            quiet_hours: true,
            tasks_to_run: Vec::new(),
            status: "HEARTBEAT_OK (quiet hours)".to_string(),
        });
    }

    let tasks = parse_heartbeat_tasks(agent_dir.as_ref())?;
    if tasks.is_empty() {
        return Ok(HeartbeatRunResult {
            quiet_hours: false,
            tasks_to_run: Vec::new(),
            status: "HEARTBEAT_OK".to_string(),
        });
    }

    let now_epoch = now.timestamp();
    for task in &tasks {
        state
            .last_check_epoch_by_category
            .insert(task.category.clone(), now_epoch);
    }

    Ok(HeartbeatRunResult {
        quiet_hours: false,
        tasks_to_run: tasks,
        status: "HEARTBEAT_TASKS".to_string(),
    })
}

/// Loads heartbeat state from JSON file.
pub fn load_state(path: impl AsRef<Path>) -> Result<HeartbeatState> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(HeartbeatState::default());
    }

    let body = fs::read_to_string(path).map_err(|source| AgentError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    if body.trim().is_empty() {
        return Ok(HeartbeatState::default());
    }

    serde_json::from_str::<HeartbeatState>(&body).map_err(AgentError::from)
}

/// Persists heartbeat state to JSON file.
pub fn save_state(path: impl AsRef<Path>, state: &HeartbeatState) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AgentError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let body = serde_json::to_string_pretty(state)?;
    fs::write(path, body).map_err(|source| AgentError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_heartbeat_tasks(agent_dir: &Path) -> Result<Vec<HeartbeatTask>> {
    let path = agent_dir.join("HEARTBEAT.md");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).map_err(|source| AgentError::Io {
        path: path.clone(),
        source,
    })?;

    let tasks = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let text = line
                .trim_start_matches("- [ ]")
                .trim_start_matches("- ")
                .trim()
                .to_string();
            HeartbeatTask {
                category: infer_category(&text),
                text,
            }
        })
        .collect::<Vec<_>>();

    Ok(tasks)
}

fn infer_category(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if lower.contains("memory") {
        return "memory".to_string();
    }
    if lower.contains("session") {
        return "session".to_string();
    }
    if lower.contains("queue") {
        return "queue".to_string();
    }
    if lower.contains("health") {
        return "health".to_string();
    }
    "general".to_string()
}

fn is_in_quiet_hours(hour: u32, start: Option<u32>, end: Option<u32>) -> bool {
    let (Some(start), Some(end)) = (start, end) else {
        return false;
    };

    if start == end {
        return false;
    }

    if start < end {
        return hour >= start && hour < end;
    }

    hour >= start || hour < end
}

/// Resolves default heartbeat state path.
pub fn default_state_path(agent_dir: impl AsRef<Path>) -> PathBuf {
    agent_dir.as_ref().join(".heartbeat_state.json")
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

    #[test]
    fn parses_tasks_and_categories() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(
            temp.path().join("HEARTBEAT.md"),
            "- [ ] Refresh memory summary\n- [ ] session cleanup\n",
        )
        .expect("write");

        let tasks = parse_heartbeat_tasks(temp.path()).expect("tasks");
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].category, "memory");
        assert_eq!(tasks[1].category, "session");
    }

    #[test]
    fn state_round_trip() {
        let temp = tempfile::tempdir().expect("tmp");
        let path = temp.path().join("state.json");

        let mut state = HeartbeatState::default();
        state
            .last_check_epoch_by_category
            .insert("memory".to_string(), 123);
        save_state(&path, &state).expect("save");
        let loaded = load_state(&path).expect("load");

        assert_eq!(
            loaded.last_check_epoch_by_category.get("memory"),
            Some(&123)
        );
    }
}

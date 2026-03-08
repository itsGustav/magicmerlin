//! Session resolution, lifecycle, transcript repair, and sub-agent orchestration.

mod error;

use std::path::{Path, PathBuf};

use chrono::Utc;
use magicmerlin_storage::{RepairReport, TranscriptStore};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::error::{Result, SessionsError};

/// Inbound message context used for resolving canonical session keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionContext {
    /// Channel name (e.g. `telegram`).
    pub channel: String,
    /// Agent name for direct sessions.
    pub agent_name: Option<String>,
    /// Chat id for group channels.
    pub chat_id: Option<String>,
    /// User id for slash command sessions.
    pub user_id: Option<String>,
    /// Whether this is a slash-command context.
    pub slash_command: bool,
    /// Optional custom pattern where `{channel}` and `{chat_id}` can be used.
    pub custom_pattern: Option<String>,
}

/// One persisted session state record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    /// Session key.
    pub session_id: String,
    /// Parent session id for sub-agent sessions.
    pub parent_session_id: Option<String>,
    /// Total tokens consumed.
    pub token_usage: u64,
    /// Total accumulated cost.
    pub total_cost_usd: f64,
    /// Last activity unix timestamp.
    pub last_activity_at: i64,
    /// Number of compactions performed.
    pub compaction_count: u64,
    /// Optional per-session model override.
    pub model_override: Option<String>,
}

/// Session engine backed by sqlite metadata and JSONL transcript files.
#[derive(Debug, Clone)]
pub struct SessionEngine {
    db_path: PathBuf,
    transcript_root: PathBuf,
}

impl SessionEngine {
    /// Creates a new session engine and runs required metadata migrations.
    pub fn new(db_path: impl AsRef<Path>, transcript_root: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let transcript_root = transcript_root.as_ref().to_path_buf();
        std::fs::create_dir_all(&transcript_root).map_err(|source| SessionsError::Io {
            path: transcript_root.clone(),
            source,
        })?;
        migrate(&db_path)?;
        Ok(Self {
            db_path,
            transcript_root,
        })
    }

    /// Resolves a canonical session key from inbound context.
    pub fn resolve_session_key(&self, context: &ResolutionContext) -> String {
        resolve_session_key(context)
    }

    /// Loads existing state or creates a new session state for `session_id`.
    pub fn load_or_create(
        &self,
        session_id: &str,
        parent_session_id: Option<&str>,
    ) -> Result<SessionState> {
        let now = Utc::now().timestamp();
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT OR IGNORE INTO session_state(session_id, parent_session_id, token_usage, total_cost_usd, last_activity_at, compaction_count, model_override)
             VALUES(?1, ?2, 0, 0.0, ?3, 0, NULL)",
            params![session_id, parent_session_id, now],
        )?;
        self.get_state(session_id)
    }

    /// Returns the transcript store bound to one session id.
    pub fn transcript_store(&self, session_id: &str) -> Result<TranscriptStore> {
        let file = self
            .transcript_root
            .join(format!("{}.jsonl", sanitize_session_id(session_id)));
        TranscriptStore::new(file).map_err(SessionsError::from)
    }

    /// Appends one transcript entry atomically and updates activity timestamp.
    pub fn append_message(&self, session_id: &str, message: &Value) -> Result<()> {
        let store = self.transcript_store(session_id)?;
        store.append(message)?;
        self.touch(session_id)
    }

    /// Compacts the transcript when token usage exceeds the threshold percentage.
    pub fn compact_if_needed(
        &self,
        session_id: &str,
        context_window: u64,
        threshold_percent: u64,
    ) -> Result<bool> {
        if context_window == 0 {
            return Ok(false);
        }

        let store = self.transcript_store(session_id)?;
        let entries = store.read(0, None)?;
        let token_usage = entries
            .iter()
            .map(magicmerlin_storage::approx_token_count)
            .sum::<usize>() as u64;
        let used_pct = token_usage.saturating_mul(100) / context_window;
        if used_pct < threshold_percent {
            return Ok(false);
        }

        store.compact(30)?;
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "UPDATE session_state SET compaction_count = compaction_count + 1, last_activity_at = ?2 WHERE session_id = ?1",
            params![session_id, Utc::now().timestamp()],
        )?;
        Ok(true)
    }

    /// Repairs broken tool use/result pairs and returns a repair report.
    pub fn repair_transcript(&self, session_id: &str) -> Result<RepairReport> {
        let store = self.transcript_store(session_id)?;
        let report = store.repair_tool_pairs()?;
        self.touch(session_id)?;
        Ok(report)
    }

    /// Spawns and tracks one sub-agent session attached to a parent session.
    pub fn spawn_sub_agent_session(
        &self,
        parent_session_id: &str,
        agent_name: &str,
    ) -> Result<String> {
        let child_id = format!("sub:{agent_name}:{}", uuid::Uuid::new_v4());
        self.load_or_create(&child_id, Some(parent_session_id))?;
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO sub_agent_links(parent_session_id, child_session_id, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?3)",
            params![parent_session_id, child_id, Utc::now().timestamp()],
        )?;
        Ok(child_id)
    }

    /// Deletes stale sub-agent sessions older than `max_idle_seconds`.
    pub fn cleanup_stale_subagents(&self, max_idle_seconds: i64) -> Result<usize> {
        let cutoff = Utc::now().timestamp() - max_idle_seconds;
        let conn = rusqlite::Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT child_session_id FROM sub_agent_links
             WHERE child_session_id IN (
                SELECT session_id FROM session_state WHERE last_activity_at < ?1
             )",
        )?;
        let child_ids = stmt
            .query_map([cutoff], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for child_id in &child_ids {
            conn.execute(
                "DELETE FROM session_state WHERE session_id = ?1",
                params![child_id],
            )?;
            conn.execute(
                "DELETE FROM sub_agent_links WHERE child_session_id = ?1",
                params![child_id],
            )?;
        }
        Ok(child_ids.len())
    }

    /// Appends an inter-session message envelope to both source and target sessions.
    pub fn send_between_sessions(
        &self,
        from_session_id: &str,
        to_session_id: &str,
        content: &str,
    ) -> Result<()> {
        let envelope = serde_json::json!({
            "type": "inter_session",
            "from": from_session_id,
            "to": to_session_id,
            "content": content,
            "at": Utc::now().timestamp(),
        });
        self.append_message(from_session_id, &envelope)?;
        self.append_message(to_session_id, &envelope)?;
        Ok(())
    }

    /// Accumulates token usage and cost for a session.
    pub fn update_usage(&self, session_id: &str, tokens: u64, cost_usd: f64) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "UPDATE session_state
             SET token_usage = token_usage + ?2,
                 total_cost_usd = total_cost_usd + ?3,
                 last_activity_at = ?4
             WHERE session_id = ?1",
            params![session_id, tokens, cost_usd, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Sets or clears a per-session model override.
    pub fn set_model_override(&self, session_id: &str, model: Option<&str>) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "UPDATE session_state SET model_override = ?2, last_activity_at = ?3 WHERE session_id = ?1",
            params![session_id, model, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Fetches current state for one session.
    pub fn get_state(&self, session_id: &str) -> Result<SessionState> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT session_id, parent_session_id, token_usage, total_cost_usd, last_activity_at, compaction_count, model_override
             FROM session_state WHERE session_id = ?1",
        )?;
        let mut rows = stmt.query([session_id])?;
        let Some(row) = rows.next()? else {
            return Err(SessionsError::MissingSession(session_id.to_string()));
        };
        Ok(SessionState {
            session_id: row.get(0)?,
            parent_session_id: row.get(1)?,
            token_usage: row.get(2)?,
            total_cost_usd: row.get(3)?,
            last_activity_at: row.get(4)?,
            compaction_count: row.get(5)?,
            model_override: row.get(6)?,
        })
    }

    fn touch(&self, session_id: &str) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "UPDATE session_state SET last_activity_at = ?2 WHERE session_id = ?1",
            params![session_id, Utc::now().timestamp()],
        )?;
        Ok(())
    }
}

/// Resolves canonical session key patterns used by the gateway and channels.
pub fn resolve_session_key(context: &ResolutionContext) -> String {
    if let Some(pattern) = &context.custom_pattern {
        return pattern
            .replace("{channel}", &context.channel)
            .replace("{chat_id}", context.chat_id.as_deref().unwrap_or(""))
            .replace("{user_id}", context.user_id.as_deref().unwrap_or(""));
    }

    if context.slash_command && context.channel == "telegram" {
        return format!(
            "telegram:slash:{}",
            context.user_id.as_deref().unwrap_or("unknown")
        );
    }

    if let Some(chat_id) = &context.chat_id {
        return format!("{}:{}", context.channel, chat_id);
    }

    format!(
        "agent:{}:main",
        context.agent_name.as_deref().unwrap_or("default")
    )
}

fn sanitize_session_id(id: &str) -> String {
    id.replace(':', "__")
}

fn migrate(db_path: &Path) -> Result<()> {
    let conn = rusqlite::Connection::open(db_path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS session_state (
            session_id TEXT PRIMARY KEY,
            parent_session_id TEXT,
            token_usage INTEGER NOT NULL DEFAULT 0,
            total_cost_usd REAL NOT NULL DEFAULT 0,
            last_activity_at INTEGER NOT NULL,
            compaction_count INTEGER NOT NULL DEFAULT 0,
            model_override TEXT
        );

        CREATE TABLE IF NOT EXISTS sub_agent_links (
            parent_session_id TEXT NOT NULL,
            child_session_id TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_required_session_patterns() {
        let dm = resolve_session_key(&ResolutionContext {
            channel: "telegram".to_string(),
            agent_name: Some("merlin".to_string()),
            chat_id: None,
            user_id: Some("u1".to_string()),
            slash_command: false,
            custom_pattern: None,
        });
        assert_eq!(dm, "agent:merlin:main");

        let group = resolve_session_key(&ResolutionContext {
            channel: "telegram".to_string(),
            agent_name: Some("merlin".to_string()),
            chat_id: Some("1234".to_string()),
            user_id: Some("u1".to_string()),
            slash_command: false,
            custom_pattern: None,
        });
        assert_eq!(group, "telegram:1234");

        let slash = resolve_session_key(&ResolutionContext {
            channel: "telegram".to_string(),
            agent_name: None,
            chat_id: None,
            user_id: Some("42".to_string()),
            slash_command: true,
            custom_pattern: None,
        });
        assert_eq!(slash, "telegram:slash:42");
    }

    #[test]
    fn repairs_transcript_tool_pairs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let engine = SessionEngine::new(
            temp.path().join("db.sqlite"),
            temp.path().join("transcripts"),
        )
        .expect("engine");
        engine.load_or_create("telegram:123", None).expect("create");
        let store = engine.transcript_store("telegram:123").expect("store");
        store
            .append(&serde_json::json!({"type":"tool_use","tool_use_id":"abc"}))
            .expect("append use");
        store
            .append(&serde_json::json!({"type":"tool_result","tool_use_id":"missing"}))
            .expect("append orphan");

        let report = engine.repair_transcript("telegram:123").expect("repair");
        assert_eq!(report.orphan_tool_results_removed, 1);
        assert_eq!(report.synthesized_tool_results, 1);
    }
}

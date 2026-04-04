//! Session lifecycle and transcript/token tracking.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{NaiveDate, Utc};
use magicmerlin_storage::{MemoryManager, SessionFileLock, Storage, TranscriptStore};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AgentError, Result};

/// Result of a session compaction operation.
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Number of transcript messages before compaction.
    pub messages_before: usize,
    /// Number of transcript messages after compaction.
    pub messages_after: usize,
    /// Estimated tokens before compaction.
    pub tokens_before: u64,
    /// Estimated tokens after compaction.
    pub tokens_after: u64,
    /// Content flushed to daily memory, if any.
    pub memory_extracted: Option<String>,
    /// Number of memory candidates extracted during compaction.
    pub memory_candidates_extracted: usize,
}

/// Canonical session key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionKey(pub String);

impl SessionKey {
    /// Builds key for normal agent channel.
    pub fn agent_main(agent_name: &str) -> Self {
        Self(magicmerlin_sessions::resolve_session_key(
            &magicmerlin_sessions::ResolutionContext {
                channel: "agent".to_string(),
                agent_name: Some(agent_name.to_string()),
                chat_id: None,
                user_id: None,
                slash_command: false,
                custom_pattern: None,
            },
        ))
    }

    /// Builds key for telegram channel.
    pub fn telegram(chat_id: &str) -> Self {
        Self(magicmerlin_sessions::resolve_session_key(
            &magicmerlin_sessions::ResolutionContext {
                channel: "telegram".to_string(),
                agent_name: None,
                chat_id: Some(chat_id.to_string()),
                user_id: None,
                slash_command: false,
                custom_pattern: None,
            },
        ))
    }
}

/// Session metadata persisted in sqlite `sessions.metadata`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    /// Model override for this session.
    pub model_override: Option<String>,
    /// Delivery context (for channels/router).
    pub delivery_context: Option<String>,
    /// Total accumulated cost in USD.
    pub accumulated_cost_usd: f64,
    /// Compaction count.
    pub compaction_count: u64,
    /// Additional metadata extension fields.
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

/// Session persisted metadata + transcript handle.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    /// Session key string.
    pub key: SessionKey,
    /// Agent name.
    pub agent_name: String,
    /// Transcript store.
    pub transcript: TranscriptStore,
    /// Running token estimate.
    pub token_count: u64,
    /// Session metadata.
    pub metadata: SessionMetadata,
}

/// Session manager backed by storage sqlite and transcript files.
#[derive(Clone)]
pub struct SessionManager {
    storage: Storage,
    sessions_dir: PathBuf,
    memory: MemoryManager,
    compaction_keep_last: usize,
    lock_timeout: Duration,
}

impl SessionManager {
    /// Creates a new session manager.
    pub fn new(
        storage: Storage,
        sessions_dir: impl AsRef<Path>,
        memory_root: impl AsRef<Path>,
    ) -> Result<Self> {
        let sessions_dir = sessions_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&sessions_dir).map_err(|source| AgentError::Io {
            path: sessions_dir.clone(),
            source,
        })?;

        let memory = MemoryManager::new(memory_root)?;
        Ok(Self {
            storage,
            sessions_dir,
            memory,
            compaction_keep_last: 30,
            lock_timeout: Duration::from_secs(3),
        })
    }

    /// Sets compaction keep-last count.
    pub fn with_compaction_keep_last(mut self, keep_last: usize) -> Self {
        self.compaction_keep_last = keep_last;
        self
    }

    /// Sets per-operation lock timeout.
    pub fn with_lock_timeout(mut self, timeout: Duration) -> Self {
        self.lock_timeout = timeout;
        self
    }

    /// Creates a new session explicitly.
    pub fn create(&self, key: SessionKey, agent_name: &str) -> Result<SessionRecord> {
        let conn = self.storage.connection()?;
        let now = Utc::now().timestamp();
        let metadata_json = serde_json::to_string(&SessionMetadata::default())?;
        conn.execute(
            "INSERT OR REPLACE INTO sessions(id, agent, status, started_at, updated_at, metadata) VALUES(?1, ?2, 'active', ?3, ?3, ?4)",
            params![key.0, agent_name, now, metadata_json],
        )?;

        let transcript = self.transcript_store(&key, agent_name)?;
        Ok(SessionRecord {
            key,
            agent_name: agent_name.to_string(),
            transcript,
            token_count: 0,
            metadata: SessionMetadata::default(),
        })
    }

    /// Loads existing or creates new session and transcript.
    pub fn load_or_create(&self, key: SessionKey, agent_name: &str) -> Result<SessionRecord> {
        let conn = self.storage.connection()?;
        let now = Utc::now().timestamp();
        let default_metadata = serde_json::to_string(&SessionMetadata::default())?;
        conn.execute(
            "INSERT OR IGNORE INTO sessions(id, agent, status, started_at, updated_at, metadata) VALUES(?1, ?2, 'active', ?3, ?3, ?4)",
            params![key.0, agent_name, now, default_metadata],
        )?;

        let metadata_raw: Option<String> = conn
            .query_row(
                "SELECT metadata FROM sessions WHERE id = ?1",
                params![key.0],
                |row| row.get(0),
            )
            .ok();

        let metadata = metadata_raw
            .as_deref()
            .and_then(|raw| serde_json::from_str::<SessionMetadata>(raw).ok())
            .unwrap_or_default();

        let transcript = self.transcript_store(&key, agent_name)?;
        let entries = transcript.read(0, None)?;
        let token_count = entries
            .iter()
            .map(magicmerlin_storage::approx_token_count)
            .sum::<usize>() as u64;

        Ok(SessionRecord {
            key,
            agent_name: agent_name.to_string(),
            transcript,
            token_count,
            metadata,
        })
    }

    /// Loads session if it exists.
    pub fn load(&self, key: SessionKey, agent_name: &str) -> Result<Option<SessionRecord>> {
        let conn = self.storage.connection()?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            params![key.0],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(None);
        }
        self.load_or_create(key, agent_name).map(Some)
    }

    /// Appends transcript entry and updates token counter.
    pub fn append_message(&self, session: &mut SessionRecord, entry: Value) -> Result<()> {
        let _lock = self.acquire_transcript_lock(&session.transcript)?;
        session.transcript.append(&entry)?;
        session.token_count = session
            .token_count
            .saturating_add(magicmerlin_storage::approx_token_count(&entry) as u64);

        let conn = self.storage.connection()?;
        conn.execute(
            "UPDATE sessions SET updated_at=?2 WHERE id=?1",
            params![session.key.0, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Deletes session metadata and transcript file.
    pub fn delete(&self, session: &SessionRecord) -> Result<()> {
        let conn = self.storage.connection()?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session.key.0])?;
        let path = session.transcript.path().to_path_buf();
        if path.exists() {
            std::fs::remove_file(&path).map_err(|source| AgentError::Io {
                path: path.clone(),
                source,
            })?;
        }
        Ok(())
    }

    /// Persists updated session metadata.
    pub fn save_metadata(&self, session: &SessionRecord) -> Result<()> {
        let conn = self.storage.connection()?;
        let metadata_json = serde_json::to_string(&session.metadata)?;
        conn.execute(
            "UPDATE sessions SET metadata = ?2, updated_at = ?3 WHERE id = ?1",
            params![session.key.0, metadata_json, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Adds estimated usage cost to session metadata.
    pub fn add_cost(&self, session: &mut SessionRecord, additional_cost_usd: f64) -> Result<()> {
        session.metadata.accumulated_cost_usd += additional_cost_usd.max(0.0);
        self.save_metadata(session)
    }

    /// Sets model override.
    pub fn set_model_override(
        &self,
        session: &mut SessionRecord,
        model_override: Option<String>,
    ) -> Result<()> {
        session.metadata.model_override = model_override;
        self.save_metadata(session)
    }

    /// Compacts transcript when nearing context limit and writes memory note first.
    /// Returns `None` if compaction was not needed, or `Some(CompactionResult)` with stats.
    pub fn compact_if_needed(
        &self,
        session: &mut SessionRecord,
        context_limit: u64,
        threshold_percent: u64,
    ) -> Result<Option<CompactionResult>> {
        if context_limit == 0 {
            return Ok(None);
        }

        let used_pct = (session.token_count.saturating_mul(100)) / context_limit;
        if used_pct < threshold_percent {
            return Ok(None);
        }

        Ok(Some(self.do_compact(session, used_pct)?))
    }

    /// Forces transcript compaction regardless of thresholds.
    pub fn compact_now(&self, session: &mut SessionRecord) -> Result<CompactionResult> {
        self.do_compact(session, 100)
    }

    fn do_compact(&self, session: &mut SessionRecord, used_pct: u64) -> Result<CompactionResult> {
        let entries_before = session.transcript.read(0, None)?;
        let messages_before = entries_before.len();
        let tokens_before = entries_before
            .iter()
            .map(magicmerlin_storage::approx_token_count)
            .sum::<usize>() as u64;

        let memory_extracted = self.pre_compaction_memory_flush(session, used_pct)?;
        let memory_candidates_extracted = memory_extracted
            .as_ref()
            .map(|s| s.lines().count())
            .unwrap_or(0);

        let _lock = self.acquire_transcript_lock(&session.transcript)?;
        session.transcript.compact(self.compaction_keep_last)?;

        let entries_after = session.transcript.read(0, None)?;
        let messages_after = entries_after.len();
        let tokens_after = entries_after
            .iter()
            .map(magicmerlin_storage::approx_token_count)
            .sum::<usize>() as u64;

        session.token_count = tokens_after;
        session.metadata.compaction_count = session.metadata.compaction_count.saturating_add(1);
        self.save_metadata(session)?;

        Ok(CompactionResult {
            messages_before,
            messages_after,
            tokens_before,
            tokens_after,
            memory_extracted,
            memory_candidates_extracted,
        })
    }

    /// Checks whether current token usage exceeds a context-threshold percentage.
    pub fn is_over_context_threshold(
        &self,
        session: &SessionRecord,
        context_limit: u64,
        threshold_percent: u64,
    ) -> bool {
        if context_limit == 0 {
            return false;
        }
        let used_pct = (session.token_count.saturating_mul(100)) / context_limit;
        used_pct >= threshold_percent
    }

    /// Returns summarized token counters for the session transcript.
    pub fn token_summary(&self, session: &SessionRecord) -> Result<(u64, usize)> {
        let values = session.transcript.read(0, None)?;
        Ok((
            values
                .iter()
                .map(magicmerlin_storage::approx_token_count)
                .sum::<usize>() as u64,
            values.len(),
        ))
    }

    /// Reads all messages from transcript.
    pub fn read_messages(&self, session: &SessionRecord) -> Result<Vec<Value>> {
        session.transcript.read(0, None).map_err(AgentError::from)
    }

    /// Estimates context window utilization as a 0.0..1.0 float.
    ///
    /// Uses the session's running token count and divides by `context_window`.
    /// If `context_window` is 0, returns 0.0.
    pub fn estimate_context_percent(&self, session: &SessionRecord, context_window: u64) -> f32 {
        if context_window == 0 {
            return 0.0;
        }
        (session.token_count as f32) / (context_window as f32)
    }

    /// Returns sessions root directory.
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    fn transcript_store(&self, key: &SessionKey, agent_name: &str) -> Result<TranscriptStore> {
        let transcript_path = self
            .sessions_dir
            .join(agent_name)
            .join(format!("{}.jsonl", sanitize_key(&key.0)));
        TranscriptStore::new(transcript_path).map_err(AgentError::from)
    }

    fn acquire_transcript_lock(&self, transcript: &TranscriptStore) -> Result<SessionFileLock> {
        SessionFileLock::acquire(transcript.path(), self.lock_timeout).map_err(AgentError::from)
    }

    fn pre_compaction_memory_flush(
        &self,
        session: &SessionRecord,
        used_pct: u64,
    ) -> Result<Option<String>> {
        let lines = extract_memory_candidates(&session.transcript.read(0, None)?)
            .into_iter()
            .take(20)
            .collect::<Vec<_>>();

        let now = Utc::now();
        let note = format!(
            "Compacting session {} at {}% context utilization; extracted {} memory hints",
            session.key.0, used_pct, lines.len()
        );
        self.memory
            .append_daily_entry(now.date_naive(), &note)
            .map_err(AgentError::from)?;

        if lines.is_empty() {
            return Ok(None);
        }

        self.write_memory_summary(now.date_naive(), &session.key.0, &lines)?;

        let summary = lines.join("\n");
        Ok(Some(summary))
    }

    fn write_memory_summary(
        &self,
        date: NaiveDate,
        session_key: &str,
        lines: &[String],
    ) -> Result<()> {
        let mut body = self
            .memory
            .read_daily(date)
            .map_err(AgentError::from)?
            .unwrap_or_default();
        body.push_str(&format!("\n## Session Memory Flush ({session_key})\n"));
        for line in lines {
            body.push_str("- ");
            body.push_str(line);
            body.push('\n');
        }

        let daily_path = self.memory.daily_path(date);
        std::fs::write(&daily_path, body).map_err(|source| AgentError::Io {
            path: daily_path,
            source,
        })
    }
}

fn extract_memory_candidates(entries: &[Value]) -> Vec<String> {
    let mut candidates = Vec::new();
    for entry in entries {
        let role = entry
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let content = entry
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if content.is_empty() {
            continue;
        }

        if role == "assistant" {
            // Extract lines with explicit memory markers
            for line in content.lines() {
                let l = line.trim();
                if l.starts_with("Remember:")
                    || l.starts_with("Note:")
                    || l.starts_with("Important:")
                    || l.starts_with("Key insight:")
                    || l.starts_with("I'll remember")
                    || l.starts_with("Decision:")
                {
                    candidates.push(l.to_string());
                } else if l.len() > 20
                    && l.len() < 200
                    && (l.contains("0x") && l.contains("contract"))
                {
                    // Capture contract addresses / web3 facts
                    candidates.push(l.to_string());
                }
            }
        }

        // Keep broad-stroke extraction for user messages with actionable content
        if role == "user" {
            let important = content.contains("TODO")
                || content.contains("remember")
                || content.contains("deadline")
                || content.contains("follow up");
            if important {
                candidates.push(format!("[{role}] {}", truncate(content, 160)));
            }
        }
    }
    candidates
}

fn truncate(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }
    let mut out = String::new();
    for c in input.chars().take(max) {
        out.push(c);
    }
    out.push_str("...");
    out
}

fn sanitize_key(input: &str) -> String {
    input.replace(':', "__")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_updates_session() {
        let temp = tempfile::tempdir().expect("tmp");
        let storage = Storage::new(temp.path().join("db.sqlite")).expect("storage");
        let manager = SessionManager::new(storage, temp.path().join("sessions"), temp.path())
            .expect("manager");

        let mut session = manager
            .load_or_create(SessionKey::agent_main("merlin"), "merlin")
            .expect("session");
        manager
            .append_message(
                &mut session,
                serde_json::json!({"role":"user","content":"hi"}),
            )
            .expect("append");

        assert!(session.token_count > 0);
    }

    #[test]
    fn compacts_and_persists_metadata() {
        let temp = tempfile::tempdir().expect("tmp");
        let storage = Storage::new(temp.path().join("db.sqlite")).expect("storage");
        let manager = SessionManager::new(storage, temp.path().join("sessions"), temp.path())
            .expect("manager");

        let mut session = manager
            .load_or_create(SessionKey::agent_main("merlin"), "merlin")
            .expect("session");

        for _ in 0..120 {
            manager
                .append_message(
                    &mut session,
                    serde_json::json!({"role":"user","content":"remember TODO: follow up on deployment deadline"}),
                )
                .expect("append");
        }

        let result = manager
            .compact_if_needed(&mut session, 100, 50)
            .expect("compact");
        assert!(result.is_some());
        let compaction = result.expect("compaction result");
        assert!(compaction.messages_before >= 120);
        assert!(compaction.messages_after < compaction.messages_before);
        assert!(compaction.tokens_before > compaction.tokens_after);
        assert!(session.metadata.compaction_count >= 1);

        let (tokens, count) = manager.token_summary(&session).expect("summary");
        assert!(tokens > 0);
        assert!(count > 0);
    }

    #[test]
    fn deletes_session_and_transcript() {
        let temp = tempfile::tempdir().expect("tmp");
        let storage = Storage::new(temp.path().join("db.sqlite")).expect("storage");
        let manager = SessionManager::new(storage, temp.path().join("sessions"), temp.path())
            .expect("manager");

        let mut session = manager
            .create(SessionKey::agent_main("merlin"), "merlin")
            .expect("session");
        manager
            .append_message(
                &mut session,
                serde_json::json!({"role":"assistant","content":"ready"}),
            )
            .expect("append");

        let transcript_path = session.transcript.path().to_path_buf();
        assert!(transcript_path.exists());

        manager.delete(&session).expect("delete");
        assert!(!transcript_path.exists());
    }
}

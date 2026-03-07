//! Session lifecycle and transcript/token tracking.

use std::path::{Path, PathBuf};

use chrono::Utc;
use magicmerlin_storage::{MemoryManager, Storage, TranscriptStore};
use rusqlite::params;

use crate::error::{AgentError, Result};

/// Canonical session key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionKey(pub String);

impl SessionKey {
    /// Builds key for normal agent channel.
    pub fn agent_main(agent_name: &str) -> Self {
        Self(format!("agent:{agent_name}:main"))
    }

    /// Builds key for telegram channel.
    pub fn telegram(chat_id: &str) -> Self {
        Self(format!("telegram:{chat_id}"))
    }
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
}

/// Session manager backed by storage sqlite and transcript files.
#[derive(Clone)]
pub struct SessionManager {
    storage: Storage,
    sessions_dir: PathBuf,
    memory: MemoryManager,
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
        })
    }

    /// Loads existing or creates new session and transcript.
    pub fn load_or_create(&self, key: SessionKey, agent_name: &str) -> Result<SessionRecord> {
        let conn = self.storage.connection()?;
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT OR IGNORE INTO sessions(id, agent, status, started_at, updated_at, metadata) VALUES(?1, ?2, 'active', ?3, ?3, '{}')",
            params![key.0, agent_name, now],
        )?;

        let transcript_path = self
            .sessions_dir
            .join(agent_name)
            .join(format!("{}.jsonl", sanitize_key(&key.0)));
        let transcript = TranscriptStore::new(transcript_path)?;
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
        })
    }

    /// Appends transcript entry and updates token counter.
    pub fn append_message(
        &self,
        session: &mut SessionRecord,
        entry: serde_json::Value,
    ) -> Result<()> {
        session.transcript.append(&entry)?;
        session.token_count += magicmerlin_storage::approx_token_count(&entry) as u64;

        let conn = self.storage.connection()?;
        conn.execute(
            "UPDATE sessions SET updated_at=?2 WHERE id=?1",
            params![session.key.0, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    /// Compacts transcript when nearing context limit and writes memory note first.
    pub fn compact_if_needed(
        &self,
        session: &mut SessionRecord,
        context_limit: u64,
        threshold_percent: u64,
    ) -> Result<bool> {
        if context_limit == 0 {
            return Ok(false);
        }

        let used_pct = (session.token_count.saturating_mul(100)) / context_limit;
        if used_pct < threshold_percent {
            return Ok(false);
        }

        let note = format!(
            "Compacting session {} at {}% context utilization",
            session.key.0, used_pct
        );
        self.memory
            .append_daily_entry(Utc::now().date_naive(), &note)?;
        session.transcript.compact(30)?;

        let entries = session.transcript.read(0, None)?;
        session.token_count = entries
            .iter()
            .map(magicmerlin_storage::approx_token_count)
            .sum::<usize>() as u64;
        Ok(true)
    }
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
}

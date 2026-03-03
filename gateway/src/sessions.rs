use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub agent: Option<String>,
    pub status: String,
    pub started_at: i64,
    pub updated_at: i64,
    pub metadata: Option<Value>,
}

pub async fn migrate_sessions(db_path: &Path) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
              id         TEXT PRIMARY KEY,
              agent      TEXT,
              status     TEXT NOT NULL DEFAULT 'active',
              started_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL,
              metadata   TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at);
            "#,
        )?;
        Ok(())
    })
    .await?
}

pub async fn upsert_session(
    db_path: &Path,
    id: &str,
    agent: Option<&str>,
    status: &str,
    metadata: Option<&Value>,
) -> Result<()> {
    let path = db_path.to_owned();
    let id = id.to_string();
    let agent = agent.map(|s| s.to_string());
    let status = status.to_string();
    let metadata_str = metadata.map(|v| serde_json::to_string(v).unwrap_or_default());
    let now = Utc::now().timestamp();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "INSERT INTO sessions (id, agent, status, started_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               status = excluded.status,
               updated_at = excluded.started_at,
               metadata = COALESCE(excluded.metadata, sessions.metadata)",
            rusqlite::params![id, agent, status, now, metadata_str],
        )?;
        Ok(())
    })
    .await?
}

pub async fn list_sessions(db_path: &Path, limit: usize) -> Result<Vec<Session>> {
    let path = db_path.to_owned();
    let limit = limit.min(500).max(1) as i64;
    tokio::task::spawn_blocking(move || -> Result<Vec<Session>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT id, agent, status, started_at, updated_at, metadata
             FROM sessions ORDER BY updated_at DESC LIMIT ?1",
        )?;

        let rows = stmt
            .query_map([limit], |row| {
                let metadata_str: Option<String> = row.get(5)?;
                let metadata: Option<Value> =
                    metadata_str.and_then(|s| serde_json::from_str(&s).ok());
                Ok(Session {
                    id: row.get(0)?,
                    agent: row.get(1)?,
                    status: row.get(2)?,
                    started_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    metadata,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    })
    .await?
}

pub async fn get_session(db_path: &Path, id: &str) -> Result<Option<Session>> {
    let path = db_path.to_owned();
    let id = id.to_string();
    tokio::task::spawn_blocking(move || -> Result<Option<Session>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT id, agent, status, started_at, updated_at, metadata
             FROM sessions WHERE id = ?1",
        )?;

        let mut rows = stmt.query([&id])?;
        if let Some(row) = rows.next()? {
            let metadata_str: Option<String> = row.get(5)?;
            let metadata: Option<Value> = metadata_str.and_then(|s| serde_json::from_str(&s).ok());
            return Ok(Some(Session {
                id: row.get(0)?,
                agent: row.get(1)?,
                status: row.get(2)?,
                started_at: row.get(3)?,
                updated_at: row.get(4)?,
                metadata,
            }));
        }

        Ok(None)
    })
    .await?
}

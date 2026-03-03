use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub id: i64,
    pub agent: String,
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    pub id: i64,
    pub agent: String,
    pub pattern: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalsState {
    pub approvals: Vec<Approval>,
    pub allowlist: Vec<AllowlistEntry>,
}

pub async fn migrate_approvals(db_path: &Path) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS approvals (
              id         INTEGER PRIMARY KEY AUTOINCREMENT,
              agent      TEXT NOT NULL DEFAULT '*',
              key        TEXT NOT NULL,
              value      TEXT NOT NULL,
              updated_at INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_approvals_agent_key ON approvals(agent, key);

            CREATE TABLE IF NOT EXISTS approval_allowlist (
              id         INTEGER PRIMARY KEY AUTOINCREMENT,
              agent      TEXT NOT NULL DEFAULT '*',
              pattern    TEXT NOT NULL,
              created_at INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_allowlist_agent_pattern ON approval_allowlist(agent, pattern);
            "#,
        )?;
        Ok(())
    })
    .await?
}

pub async fn get_approvals(db_path: &Path) -> Result<ApprovalsState> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<ApprovalsState> {
        let conn = rusqlite::Connection::open(path)?;

        let mut stmt =
            conn.prepare("SELECT id, agent, key, value, updated_at FROM approvals ORDER BY id")?;
        let approvals = stmt
            .query_map([], |row| {
                Ok(Approval {
                    id: row.get(0)?,
                    agent: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut stmt = conn
            .prepare("SELECT id, agent, pattern, created_at FROM approval_allowlist ORDER BY id")?;
        let allowlist = stmt
            .query_map([], |row| {
                Ok(AllowlistEntry {
                    id: row.get(0)?,
                    agent: row.get(1)?,
                    pattern: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(ApprovalsState {
            approvals,
            allowlist,
        })
    })
    .await?
}

pub async fn set_approvals_from_file(db_path: &Path, file_path: &Path) -> Result<usize> {
    let raw = std::fs::read_to_string(file_path).map_err(|e| anyhow::anyhow!("read file: {e}"))?;

    let entries: Vec<ApprovalFileEntry> =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("parse JSON: {e}"))?;

    let path = db_path.to_owned();
    let now = Utc::now().timestamp();
    let count = entries.len();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        for entry in &entries {
            let agent = entry.agent.as_deref().unwrap_or("*");
            conn.execute(
                "INSERT INTO approvals (agent, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(agent, key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                rusqlite::params![agent, entry.key, entry.value, now],
            )?;
        }
        Ok(())
    })
    .await??;

    Ok(count)
}

#[derive(Debug, serde::Deserialize)]
struct ApprovalFileEntry {
    agent: Option<String>,
    key: String,
    value: String,
}

pub async fn allowlist_add(db_path: &Path, pattern: &str, agent: Option<&str>) -> Result<()> {
    let path = db_path.to_owned();
    let pattern = pattern.to_string();
    let agent = agent.unwrap_or("*").to_string();
    let now = Utc::now().timestamp();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "INSERT INTO approval_allowlist (agent, pattern, created_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(agent, pattern) DO NOTHING",
            rusqlite::params![agent, pattern, now],
        )?;
        Ok(())
    })
    .await?
}

pub async fn allowlist_remove(db_path: &Path, pattern: &str, agent: Option<&str>) -> Result<()> {
    let path = db_path.to_owned();
    let pattern = pattern.to_string();
    let agent = agent.unwrap_or("*").to_string();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "DELETE FROM approval_allowlist WHERE agent = ?1 AND pattern = ?2",
            rusqlite::params![agent, pattern],
        )?;
        Ok(())
    })
    .await?
}

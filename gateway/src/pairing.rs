use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmScope {
    Shared,
    Peer,
}

impl DmScope {
    pub fn from_env() -> Self {
        match std::env::var("MAGICMERLIN_DM_SCOPE") {
            Ok(v) if v.eq_ignore_ascii_case("peer") => Self::Peer,
            _ => Self::Shared,
        }
    }
}

pub fn resolve_dm_session_key(
    scope: DmScope,
    channel: &str,
    peer_id: &str,
    account_id: Option<&str>,
) -> String {
    match scope {
        DmScope::Shared => "dm:shared".to_string(),
        DmScope::Peer => {
            let account = account_id.filter(|s| !s.is_empty()).unwrap_or("default");
            format!("dm:{channel}:{account}:{peer_id}")
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingRequest {
    pub id: i64,
    pub channel: String,
    pub peer_id: String,
    pub account_id: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub approved_at: Option<i64>,
    pub approved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingState {
    pub channel: String,
    pub peer_id: String,
    pub account_id: Option<String>,
    pub status: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PairingAction {
    Approve,
    Reject,
}

#[derive(Debug, Clone)]
pub enum PairingActionOutcome {
    Updated(PairingRequest),
    NotFound,
    InvalidState { current_status: String },
}

fn normalize_account_id(account_id: Option<&str>) -> String {
    account_id.unwrap_or_default().to_string()
}

fn denormalize_account_id(account_id: String) -> Option<String> {
    if account_id.is_empty() {
        None
    } else {
        Some(account_id)
    }
}

pub async fn migrate_pairing(db_path: &Path) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS pairing_requests (
              id          INTEGER PRIMARY KEY AUTOINCREMENT,
              channel     TEXT NOT NULL,
              peer_id     TEXT NOT NULL,
              account_id  TEXT NOT NULL DEFAULT '',
              status      TEXT NOT NULL,
              created_at  INTEGER NOT NULL,
              approved_at INTEGER,
              approved_by TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_pairing_requests_status_created_at
              ON pairing_requests(status, created_at DESC);

            CREATE TABLE IF NOT EXISTS pairing_state (
              channel     TEXT NOT NULL,
              peer_id     TEXT NOT NULL,
              account_id  TEXT NOT NULL DEFAULT '',
              status      TEXT NOT NULL,
              updated_at  INTEGER NOT NULL,
              PRIMARY KEY(channel, peer_id, account_id)
            );
            "#,
        )?;
        Ok(())
    })
    .await?
}

pub async fn create_pairing_request(
    db_path: &Path,
    channel: &str,
    peer_id: &str,
    account_id: Option<&str>,
) -> Result<i64> {
    let path = db_path.to_owned();
    let channel = channel.to_string();
    let peer_id = peer_id.to_string();
    let account_id = normalize_account_id(account_id);
    let created_at = Utc::now().timestamp();

    tokio::task::spawn_blocking(move || -> Result<i64> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "INSERT INTO pairing_requests (channel, peer_id, account_id, status, created_at)
             VALUES (?1, ?2, ?3, 'pending', ?4)",
            rusqlite::params![channel, peer_id, account_id, created_at],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await?
}

pub async fn list_pairing_requests(
    db_path: &Path,
    channel: Option<&str>,
    account_id: Option<&str>,
    status: Option<&str>,
    limit: usize,
) -> Result<Vec<PairingRequest>> {
    let path = db_path.to_owned();
    let channel = channel.map(|s| s.to_string());
    let account_id = account_id.map(|s| s.to_string());
    let status = status.map(|s| s.to_string());
    let limit = limit.clamp(1, 500) as i64;

    tokio::task::spawn_blocking(move || -> Result<Vec<PairingRequest>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT id, channel, peer_id, account_id, status, created_at, approved_at, approved_by
             FROM pairing_requests
             WHERE (?1 IS NULL OR channel = ?1)
               AND (?2 IS NULL OR account_id = ?2)
               AND (?3 IS NULL OR status = ?3)
             ORDER BY created_at DESC
             LIMIT ?4",
        )?;

        let rows = stmt
            .query_map(
                rusqlite::params![channel, account_id, status, limit],
                |row| {
                    let account_id: String = row.get(3)?;
                    Ok(PairingRequest {
                        id: row.get(0)?,
                        channel: row.get(1)?,
                        peer_id: row.get(2)?,
                        account_id: denormalize_account_id(account_id),
                        status: row.get(4)?,
                        created_at: row.get(5)?,
                        approved_at: row.get(6)?,
                        approved_by: row.get(7)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    })
    .await?
}

pub async fn list_pairing_state(
    db_path: &Path,
    channel: Option<&str>,
    peer_id: Option<&str>,
    account_id: Option<&str>,
    limit: usize,
) -> Result<Vec<PairingState>> {
    let path = db_path.to_owned();
    let channel = channel.map(|s| s.to_string());
    let peer_id = peer_id.map(|s| s.to_string());
    let account_id = account_id.map(|s| s.to_string());
    let limit = limit.clamp(1, 500) as i64;

    tokio::task::spawn_blocking(move || -> Result<Vec<PairingState>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT channel, peer_id, account_id, status, updated_at
             FROM pairing_state
             WHERE (?1 IS NULL OR channel = ?1)
               AND (?2 IS NULL OR peer_id = ?2)
               AND (?3 IS NULL OR account_id = ?3)
             ORDER BY updated_at DESC
             LIMIT ?4",
        )?;

        let rows = stmt
            .query_map(
                rusqlite::params![channel, peer_id, account_id, limit],
                |row| {
                    let account_id: String = row.get(2)?;
                    Ok(PairingState {
                        channel: row.get(0)?,
                        peer_id: row.get(1)?,
                        account_id: denormalize_account_id(account_id),
                        status: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    })
    .await?
}

pub async fn apply_pairing_action(
    db_path: &Path,
    request_id: i64,
    action: PairingAction,
    actor: Option<&str>,
) -> Result<PairingActionOutcome> {
    let path = db_path.to_owned();
    let actor = actor.map(|s| s.to_string());

    tokio::task::spawn_blocking(move || -> Result<PairingActionOutcome> {
        let mut conn = rusqlite::Connection::open(path)?;
        let tx = conn.transaction()?;

        let (id, channel, peer_id, account_id, created_at) = {
            let mut stmt = tx.prepare(
                "SELECT id, channel, peer_id, account_id, status, created_at, approved_at, approved_by
                 FROM pairing_requests
                 WHERE id = ?1",
            )?;
            let mut rows = stmt.query([request_id])?;
            let Some(row) = rows.next()? else {
                return Ok(PairingActionOutcome::NotFound);
            };

            let current_status: String = row.get(4)?;
            if current_status != "pending" {
                return Ok(PairingActionOutcome::InvalidState { current_status });
            }

            let id: i64 = row.get(0)?;
            let channel: String = row.get(1)?;
            let peer_id: String = row.get(2)?;
            let account_id: String = row.get(3)?;
            let created_at: i64 = row.get(5)?;
            (id, channel, peer_id, account_id, created_at)
        };

        let now = Utc::now().timestamp();
        let (new_status, approved_at, approved_by) = match action {
            PairingAction::Approve => ("approved", Some(now), actor.clone()),
            PairingAction::Reject => ("rejected", None, None),
        };

        tx.execute(
            "UPDATE pairing_requests
             SET status = ?1, approved_at = ?2, approved_by = ?3
             WHERE id = ?4 AND status = 'pending'",
            rusqlite::params![new_status, approved_at, approved_by, request_id],
        )?;

        tx.execute(
            "INSERT INTO pairing_state (channel, peer_id, account_id, status, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(channel, peer_id, account_id)
             DO UPDATE SET status = excluded.status, updated_at = excluded.updated_at",
            rusqlite::params![channel, peer_id, account_id, new_status, now],
        )?;

        tx.commit()?;

        Ok(PairingActionOutcome::Updated(PairingRequest {
            id,
            channel,
            peer_id,
            account_id: denormalize_account_id(account_id),
            status: new_status.to_string(),
            created_at,
            approved_at,
            approved_by,
        }))
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pairing_approve_lifecycle_updates_request_and_state() {
        let tmp = tempfile::NamedTempFile::new().expect("temp db");
        migrate_pairing(tmp.path()).await.expect("migrate pairing");

        let id = create_pairing_request(tmp.path(), "telegram", "peer-1", Some("acc-1"))
            .await
            .expect("create request");

        let outcome = apply_pairing_action(
            tmp.path(),
            id,
            PairingAction::Approve,
            Some("gateway-operator"),
        )
        .await
        .expect("approve");

        let PairingActionOutcome::Updated(request) = outcome else {
            panic!("expected updated");
        };
        assert_eq!(request.status, "approved");
        assert!(request.approved_at.is_some());
        assert_eq!(request.approved_by.as_deref(), Some("gateway-operator"));

        let rows = list_pairing_state(
            tmp.path(),
            Some("telegram"),
            Some("peer-1"),
            Some("acc-1"),
            10,
        )
        .await
        .expect("state rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "approved");
    }

    #[tokio::test]
    async fn pairing_reject_lifecycle_updates_request_and_state() {
        let tmp = tempfile::NamedTempFile::new().expect("temp db");
        migrate_pairing(tmp.path()).await.expect("migrate pairing");

        let id = create_pairing_request(tmp.path(), "telegram", "peer-2", None)
            .await
            .expect("create request");

        let outcome = apply_pairing_action(tmp.path(), id, PairingAction::Reject, Some("operator"))
            .await
            .expect("reject");

        let PairingActionOutcome::Updated(request) = outcome else {
            panic!("expected updated");
        };
        assert_eq!(request.status, "rejected");
        assert!(request.approved_at.is_none());
        assert!(request.approved_by.is_none());

        let rows = list_pairing_state(tmp.path(), Some("telegram"), Some("peer-2"), None, 10)
            .await
            .expect("state rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "rejected");
    }
}

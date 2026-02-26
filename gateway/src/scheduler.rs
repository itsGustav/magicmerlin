use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use chrono::{TimeZone, Utc};
use cron::Schedule;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: i64,
    pub name: String,
    pub schedule: String,
    pub kind: String,
    pub payload: Value,
    pub enabled: bool,
    pub last_run_at: Option<i64>,
    pub next_run_at: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerState {
    pub job_count: i64,
    pub next_run_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    db_path: PathBuf,
    http: Client,
}

impl Scheduler {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        migrate(&db_path).await?;
        // Ensure all jobs have a next_run_at.
        normalize_next_run_at(&db_path).await?;
        Ok(Self {
            db_path,
            http: Client::new(),
        })
    }

    #[allow(dead_code)]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub async fn state(&self) -> Result<SchedulerState> {
        scheduler_state(&self.db_path).await
    }

    pub async fn list_jobs(&self) -> Result<Vec<Job>> {
        list_jobs(&self.db_path).await
    }

    pub async fn add_job(
        &self,
        name: String,
        schedule: String,
        kind: String,
        payload: Value,
    ) -> Result<i64> {
        // Validate schedule now.
        compute_next_run_at(&schedule, Utc::now().timestamp())?;
        add_job(&self.db_path, name, schedule, kind, payload).await
    }

    pub async fn remove_job(&self, id: i64) -> Result<()> {
        remove_job(&self.db_path, id).await
    }

    pub async fn run_job_now(&self, id: i64) -> Result<()> {
        let job = get_job(&self.db_path, id)
            .await?
            .ok_or_else(|| anyhow!("job not found"))?;
        self.run_job(&job).await
    }

    pub fn spawn_daemon(self: std::sync::Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(err) = self.tick_once().await {
                    eprintln!("[scheduler] tick error: {err:#}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        })
    }

    async fn tick_once(&self) -> Result<()> {
        // Find due jobs and run them sequentially (simple + deterministic).
        let now = Utc::now().timestamp();
        let due = due_jobs(&self.db_path, now).await?;
        for job in due {
            // Skip disabled.
            if !job.enabled {
                continue;
            }
            let _ = self.run_job(&job).await;
        }

        // Sleep until next boundary (1s tick keeps it simple).
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    async fn run_job(&self, job: &Job) -> Result<()> {
        let now = Utc::now().timestamp();

        // Execute.
        let run_res = match job.kind.as_str() {
            "discord_webhook" => self.run_discord_webhook(job).await,
            "http_get" => self.run_http_get(job).await,
            other => Err(anyhow!("unsupported job kind: {other}")),
        };

        // Persist result + next_run.
        let (last_error, ok) = match run_res {
            Ok(()) => (None, true),
            Err(e) => (Some(format!("{e:#}")), false),
        };

        let next_run_at = match compute_next_run_at(&job.schedule, now) {
            Ok(ts) => Some(ts),
            Err(e) => {
                // If schedule is invalid, disable and persist error.
                let msg = Some(format!("invalid schedule: {e:#}"));
                update_job_run(&self.db_path, job.id, now, None, msg, false).await?;
                return Ok(());
            }
        };

        update_job_run(&self.db_path, job.id, now, next_run_at, last_error, ok).await?;
        Ok(())
    }

    async fn run_discord_webhook(&self, job: &Job) -> Result<()> {
        let mut payload = job.payload.clone();
        let obj = payload
            .as_object_mut()
            .ok_or_else(|| anyhow!("discord_webhook payload must be a JSON object"))?;

        let webhook_url = obj
            .remove("webhook_url")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| anyhow!("discord_webhook payload.webhook_url is required"))?;

        if obj.is_empty() {
            return Err(anyhow!(
                "discord_webhook payload must include at least one Discord field (e.g. content)"
            ));
        }

        let res = self
            .http
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .context("POST discord webhook")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("discord webhook failed: {status} {body}"));
        }

        Ok(())
    }

    async fn run_http_get(&self, job: &Job) -> Result<()> {
        let obj = job
            .payload
            .as_object()
            .ok_or_else(|| anyhow!("http_get payload must be a JSON object"))?;

        let url = obj
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("http_get payload.url is required"))?;

        let mut req = self.http.get(url);

        if let Some(headers) = obj.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in headers {
                if let Some(vs) = v.as_str() {
                    req = req.header(k, vs);
                }
            }
        }

        let res = req.send().await.context("GET url")?;
        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("http_get failed: {status} {body}"));
        }

        Ok(())
    }
}

pub fn default_db_path() -> PathBuf {
    std::env::var("MAGICMERLIN_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./magicmerlin.db"))
}

pub fn compute_next_run_at(schedule: &str, now_ts: i64) -> Result<i64> {
    let sched = Schedule::from_str(schedule)
        .with_context(|| format!("invalid cron schedule: {schedule}"))?;

    let now = Utc
        .timestamp_opt(now_ts, 0)
        .single()
        .ok_or_else(|| anyhow!("invalid timestamp: {now_ts}"))?;

    let next = sched
        .after(&now)
        .next()
        .ok_or_else(|| anyhow!("cron has no next occurrence"))?;

    Ok(next.timestamp())
}

// ---------------------------------------------------------------------------
// SQLite store (rusqlite)
// ---------------------------------------------------------------------------

async fn migrate(db_path: &Path) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            r#"
      PRAGMA journal_mode=WAL;
      PRAGMA synchronous=NORMAL;

      CREATE TABLE IF NOT EXISTS jobs (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        name         TEXT NOT NULL,
        schedule     TEXT NOT NULL,
        kind         TEXT NOT NULL,
        payload      TEXT NOT NULL,
        enabled      INTEGER NOT NULL DEFAULT 1,
        last_run_at  INTEGER,
        next_run_at  INTEGER,
        last_error   TEXT
      );

      CREATE INDEX IF NOT EXISTS idx_jobs_next_run ON jobs(enabled, next_run_at);
      "#,
        )?;
        Ok(())
    })
    .await?
}

async fn normalize_next_run_at(db_path: &Path) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
    let conn = rusqlite::Connection::open(path)?;
    let mut stmt = conn.prepare("SELECT id, schedule FROM jobs WHERE enabled = 1 AND (next_run_at IS NULL OR next_run_at = 0)")?;
    let rows = stmt
      .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?
      .collect::<std::result::Result<Vec<_>, _>>()?;

    let now = Utc::now().timestamp();
    for (id, schedule) in rows {
      if let Ok(next) = compute_next_run_at(&schedule, now) {
        conn.execute(
          "UPDATE jobs SET next_run_at = ?1 WHERE id = ?2",
          rusqlite::params![next, id],
        )?;
      }
    }
    Ok(())
  })
  .await?
}

async fn scheduler_state(db_path: &Path) -> Result<SchedulerState> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<SchedulerState> {
        let conn = rusqlite::Connection::open(path)?;
        let job_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM jobs WHERE enabled = 1", [], |r| {
                r.get(0)
            })?;
        let next_run_at: Option<i64> = conn.query_row(
            "SELECT MIN(next_run_at) FROM jobs WHERE enabled = 1",
            [],
            |r| r.get(0),
        )?;
        Ok(SchedulerState {
            job_count,
            next_run_at,
        })
    })
    .await?
}

async fn list_jobs(db_path: &Path) -> Result<Vec<Job>> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<Vec<Job>> {
    let conn = rusqlite::Connection::open(path)?;
    let mut stmt = conn.prepare(
      "SELECT id, name, schedule, kind, payload, enabled, last_run_at, next_run_at, last_error FROM jobs ORDER BY id ASC",
    )?;

    let jobs = stmt
      .query_map([], |row| {
        let payload_str: String = row.get(4)?;
        let payload: Value = serde_json::from_str(&payload_str).unwrap_or(Value::Null);
        Ok(Job {
          id: row.get(0)?,
          name: row.get(1)?,
          schedule: row.get(2)?,
          kind: row.get(3)?,
          payload,
          enabled: row.get::<_, i64>(5)? != 0,
          last_run_at: row.get(6)?,
          next_run_at: row.get(7)?,
          last_error: row.get(8)?,
        })
      })?
      .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(jobs)
  })
  .await?
}

async fn get_job(db_path: &Path, id: i64) -> Result<Option<Job>> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<Option<Job>> {
    let conn = rusqlite::Connection::open(path)?;
    let mut stmt = conn.prepare(
      "SELECT id, name, schedule, kind, payload, enabled, last_run_at, next_run_at, last_error FROM jobs WHERE id = ?1",
    )?;

    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
      let payload_str: String = row.get(4)?;
      let payload: Value = serde_json::from_str(&payload_str).unwrap_or(Value::Null);
      return Ok(Some(Job {
        id: row.get(0)?,
        name: row.get(1)?,
        schedule: row.get(2)?,
        kind: row.get(3)?,
        payload,
        enabled: row.get::<_, i64>(5)? != 0,
        last_run_at: row.get(6)?,
        next_run_at: row.get(7)?,
        last_error: row.get(8)?,
      }));
    }

    Ok(None)
  })
  .await?
}

async fn add_job(
    db_path: &Path,
    name: String,
    schedule: String,
    kind: String,
    payload: Value,
) -> Result<i64> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<i64> {
    let conn = rusqlite::Connection::open(path)?;

    let now = Utc::now().timestamp();
    let next = compute_next_run_at(&schedule, now)?;

    let payload_str = serde_json::to_string(&payload)?;

    conn.execute(
      "INSERT INTO jobs (name, schedule, kind, payload, enabled, next_run_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
      rusqlite::params![name, schedule, kind, payload_str, next],
    )?;

    Ok(conn.last_insert_rowid())
  })
  .await?
}

async fn remove_job(db_path: &Path, id: i64) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute("DELETE FROM jobs WHERE id = ?1", [id])?;
        Ok(())
    })
    .await?
}

async fn due_jobs(db_path: &Path, now_ts: i64) -> Result<Vec<Job>> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<Vec<Job>> {
    let conn = rusqlite::Connection::open(path)?;
    let mut stmt = conn.prepare(
      "SELECT id, name, schedule, kind, payload, enabled, last_run_at, next_run_at, last_error FROM jobs WHERE enabled = 1 AND next_run_at IS NOT NULL AND next_run_at <= ?1 ORDER BY next_run_at ASC LIMIT 25",
    )?;

    let jobs = stmt
      .query_map([now_ts], |row| {
        let payload_str: String = row.get(4)?;
        let payload: Value = serde_json::from_str(&payload_str).unwrap_or(Value::Null);
        Ok(Job {
          id: row.get(0)?,
          name: row.get(1)?,
          schedule: row.get(2)?,
          kind: row.get(3)?,
          payload,
          enabled: row.get::<_, i64>(5)? != 0,
          last_run_at: row.get(6)?,
          next_run_at: row.get(7)?,
          last_error: row.get(8)?,
        })
      })?
      .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(jobs)
  })
  .await?
}

async fn update_job_run(
    db_path: &Path,
    id: i64,
    last_run_at: i64,
    next_run_at: Option<i64>,
    last_error: Option<String>,
    keep_enabled: bool,
) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
    let conn = rusqlite::Connection::open(path)?;
    let enabled = if keep_enabled { 1 } else { 0 };
    conn.execute(
      "UPDATE jobs SET last_run_at = ?1, next_run_at = ?2, last_error = ?3, enabled = ?4 WHERE id = ?5",
      rusqlite::params![last_run_at, next_run_at, last_error, enabled, id],
    )?;
    Ok(())
  })
  .await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_next_run_at_basic() {
        // Every second.
        let now = 1_700_000_000i64;
        let next = compute_next_run_at("* * * * * *", now).unwrap();
        assert!(next > now);
    }
}

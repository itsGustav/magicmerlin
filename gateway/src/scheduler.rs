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

    pub attempts: i64,
    pub max_attempts: i64,
    pub backoff_seconds: i64,

    pub last_run_at: Option<i64>,
    pub next_run_at: Option<i64>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: i64,
    pub job_id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub status: String,
    pub error: Option<String>,
    pub metadata: Option<Value>,
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
        // Ensure session + approval tables exist for the scheduler to write to.
        crate::sessions::migrate_sessions(&db_path).await?;
        crate::approvals::migrate_approvals(&db_path).await?;
        // Ensure all enabled jobs have next_run_at.
        normalize_next_run_at(&db_path).await?;
        Ok(Self {
            db_path,
            http: Client::new(),
        })
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
        max_attempts: Option<i64>,
        backoff_seconds: Option<i64>,
    ) -> Result<i64> {
        // Validate schedule now.
        compute_next_run_at(&schedule, Utc::now().timestamp())?;

        add_job(
            &self.db_path,
            name,
            schedule,
            kind,
            payload,
            max_attempts,
            backoff_seconds,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_job(
        &self,
        id: i64,
        name: Option<String>,
        schedule: Option<String>,
        kind: Option<String>,
        payload: Option<Value>,
        max_attempts: Option<i64>,
        backoff_seconds: Option<i64>,
    ) -> Result<()> {
        // Validate new schedule if provided.
        if let Some(ref sched) = schedule {
            compute_next_run_at(sched, Utc::now().timestamp())?;
        }
        edit_job_db(
            &self.db_path,
            id,
            name,
            schedule,
            kind,
            payload,
            max_attempts,
            backoff_seconds,
        )
        .await
    }

    pub async fn remove_job(&self, id: i64) -> Result<()> {
        remove_job(&self.db_path, id).await
    }

    pub async fn clear_jobs(&self) -> Result<usize> {
        clear_jobs(&self.db_path).await
    }

    pub async fn pause_job(&self, id: i64) -> Result<()> {
        set_job_enabled(&self.db_path, id, false).await
    }

    pub async fn resume_job(&self, id: i64) -> Result<()> {
        set_job_enabled(&self.db_path, id, true).await
    }

    pub async fn run_job_now(&self, id: i64) -> Result<()> {
        let job = get_job(&self.db_path, id)
            .await?
            .ok_or_else(|| anyhow!("job not found"))?;
        self.run_job(&job).await
    }

    pub async fn list_dead_letters(&self, limit: usize) -> Result<Vec<DeadLetter>> {
        list_dead_letters(&self.db_path, limit).await
    }

    pub async fn list_runs(&self, job_id: Option<i64>, limit: usize) -> Result<Vec<Run>> {
        list_runs_db(&self.db_path, job_id, limit).await
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
        let started_at = Utc::now().timestamp();

        // For agent_turn jobs, upsert a session record.
        if job.kind == "agent_turn" {
            let session_id = format!("job:{}", job.id);
            let _ = crate::sessions::upsert_session(
                &self.db_path,
                &session_id,
                Some("agent_turn"),
                "running",
                Some(&serde_json::json!({"jobName": job.name})),
            )
            .await;
        }

        // Execute.
        let run_res: Result<Option<Value>> = match job.kind.as_str() {
            "discord_webhook" => self.run_discord_webhook(job).await.map(|_| None),
            "discord_bot" => self.run_discord_bot(job).await.map(|_| None),
            "http_get" => self.run_http_get(job).await.map(|_| None),
            "agent_turn" => self.run_agent_turn(job).await.map(Some),
            other => Err(anyhow!("unsupported job kind: {other}")),
        };

        let ended_at = Utc::now().timestamp();
        let now = ended_at;

        // Update session status for agent_turn jobs.
        if job.kind == "agent_turn" {
            let session_id = format!("job:{}", job.id);
            let status = if run_res.is_ok() {
                "completed"
            } else {
                "failed"
            };
            let _ = crate::sessions::upsert_session(
                &self.db_path,
                &session_id,
                Some("agent_turn"),
                status,
                None,
            )
            .await;
        }

        match run_res {
            Ok(metadata) => {
                // Record successful run.
                let _ = insert_run(
                    &self.db_path,
                    job.id,
                    started_at,
                    Some(ended_at),
                    "success",
                    None,
                    metadata.as_ref(),
                )
                .await;

                match compute_next_run_at(&job.schedule, now) {
                    Ok(next) => {
                        update_job_after_run(
                            &self.db_path,
                            job.id,
                            now,
                            Some(next),
                            Some("success".to_string()),
                            None,
                            0,
                            true,
                        )
                        .await?;
                        Ok(())
                    }
                    Err(e) => {
                        // Should not happen (we validate on insert), but if DB was edited: disable.
                        update_job_after_run(
                            &self.db_path,
                            job.id,
                            now,
                            None,
                            Some("invalid_schedule".to_string()),
                            Some(format!("{e:#}")),
                            0,
                            false,
                        )
                        .await?;
                        Ok(())
                    }
                }
            }

            Err(e) => {
                let err_str = format!("{e:#}");

                // Record failed run.
                let _ = insert_run(
                    &self.db_path,
                    job.id,
                    started_at,
                    Some(ended_at),
                    "error",
                    Some(&err_str),
                    None,
                )
                .await;

                let attempts = job.attempts.saturating_add(1);

                // Determine whether to retry.
                if attempts >= job.max_attempts.max(1) {
                    // Dead-letter + disable.
                    insert_dead_letter(&self.db_path, job, now, &err_str).await?;
                    update_job_after_run(
                        &self.db_path,
                        job.id,
                        now,
                        None,
                        Some("dead_letter".to_string()),
                        Some(err_str),
                        attempts,
                        false,
                    )
                    .await?;
                } else {
                    let retry_at = now + compute_backoff(job.backoff_seconds, attempts);
                    update_job_after_run(
                        &self.db_path,
                        job.id,
                        now,
                        Some(retry_at),
                        Some("retry".to_string()),
                        Some(err_str),
                        attempts,
                        true,
                    )
                    .await?;
                }

                Ok(())
            }
        }
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

    async fn run_discord_bot(&self, job: &Job) -> Result<()> {
        let mut payload = job.payload.clone();
        let obj = payload
            .as_object_mut()
            .ok_or_else(|| anyhow!("discord_bot payload must be a JSON object"))?;

        let channel_id = obj
            .remove("channel_id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| anyhow!("discord_bot payload.channel_id is required"))?;

        let bot_token = obj
            .remove("bot_token")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .or_else(|| std::env::var("MAGICMERLIN_DISCORD_BOT_TOKEN").ok())
            .ok_or_else(|| anyhow!("discord bot token required (payload.bot_token or MAGICMERLIN_DISCORD_BOT_TOKEN)"))?;

        if obj.is_empty() {
            return Err(anyhow!(
                "discord_bot payload must include message fields (e.g. content)"
            ));
        }

        let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages");
        let res = self
            .http
            .post(url)
            .header("Authorization", format!("Bot {bot_token}"))
            .json(&payload)
            .send()
            .await
            .context("POST discord bot message")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("discord bot send failed: {status} {body}"));
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

    async fn run_agent_turn(&self, job: &Job) -> Result<Value> {
        let obj = job
            .payload
            .as_object()
            .ok_or_else(|| anyhow!("agent_turn payload must be a JSON object"))?;

        let message = obj
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("agent_turn payload.message is required"))?;

        let timeout_secs = obj
            .get("timeoutSeconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        // Only allow shell execution for messages starting with "Run:" or "Run ".
        let cmd = if let Some(rest) = message.strip_prefix("Run:") {
            rest.trim()
        } else if let Some(rest) = message.strip_prefix("Run ") {
            rest.trim()
        } else {
            return Err(anyhow!(
                "agent_turn: message must start with 'Run:' or 'Run ' (got: {})",
                &message[..message.len().min(80)]
            ));
        };

        if cmd.is_empty() {
            return Err(anyhow!("agent_turn: empty command after Run prefix"));
        }

        // Gate behind env var.
        if std::env::var("MAGICMERLIN_ALLOW_SHELL").as_deref() != Ok("1") {
            return Err(anyhow!(
                "agent_turn shell execution requires MAGICMERLIN_ALLOW_SHELL=1"
            ));
        }

        // Execute with timeout.
        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output(),
        )
        .await
        .map_err(|_| anyhow!("agent_turn: command timed out after {timeout_secs}s"))?
        .context("agent_turn: failed to spawn shell command")?;

        // Truncate stdout/stderr to 8KB each.
        const MAX_OUTPUT: usize = 8 * 1024;
        let stdout_raw = String::from_utf8_lossy(&output.stdout);
        let stderr_raw = String::from_utf8_lossy(&output.stderr);
        let stdout: &str = if stdout_raw.len() > MAX_OUTPUT {
            &stdout_raw[..MAX_OUTPUT]
        } else {
            &stdout_raw
        };
        let stderr: &str = if stderr_raw.len() > MAX_OUTPUT {
            &stderr_raw[..MAX_OUTPUT]
        } else {
            &stderr_raw
        };

        let metadata = serde_json::json!({
            "exitCode": output.status.code(),
            "stdout": stdout,
            "stderr": stderr,
        });

        if !output.status.success() {
            return Err(anyhow!(
                "agent_turn: command exited with status {}",
                output.status
            ));
        }

        Ok(metadata)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadLetter {
    pub id: i64,
    pub job_id: i64,
    pub failed_at: i64,
    pub error: String,
    pub job_snapshot: Value,
}

pub fn default_db_path() -> PathBuf {
    if let Ok(p) = std::env::var("MAGICMERLIN_DB_PATH") {
        return PathBuf::from(p);
    }

    // Resolve state directory.
    let state_dir = if let Ok(p) = std::env::var("MAGICMERLIN_STATE_DIR") {
        PathBuf::from(p)
    } else if let Ok(p) = std::env::var("MAGICMERLIN_HOME") {
        PathBuf::from(p)
    } else {
        // Default: ~/.magicmerlin
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".magicmerlin")
    };

    // Best-effort create.
    let _ = std::fs::create_dir_all(&state_dir);

    state_dir.join("magicmerlin.db")
}

// ---------------------------------------------------------------------------
// Schedule parsing: plain cron | every:<s>@<anchor> | cron:<expr>@<tz>
// ---------------------------------------------------------------------------

pub fn compute_next_run_at(schedule: &str, now_ts: i64) -> Result<i64> {
    if let Some(rest) = schedule.strip_prefix("every:") {
        return compute_next_interval(rest, now_ts);
    }
    if let Some(rest) = schedule.strip_prefix("cron:") {
        return compute_next_cron_tz(rest, now_ts);
    }
    // Plain cron expression.
    compute_next_plain_cron(schedule, now_ts)
}

fn normalize_cron_fields(expr: &str) -> String {
    let mut fields: Vec<String> = expr.split_whitespace().map(|s| s.to_string()).collect();

    if fields.len() == 5 {
        // Standard 5-field cron -> prepend seconds=0 for the `cron` crate (6-field).
        fields.insert(0, "0".to_string());
    }

    // Some schedulers encode Sunday as 0. The `cron` crate accepts 1-7.
    // Normalize exact "0" in day-of-week position to "7".
    if fields.len() >= 6 && fields[5] == "0" {
        fields[5] = "7".to_string();
    }

    fields.join(" ")
}

fn compute_next_plain_cron(schedule: &str, now_ts: i64) -> Result<i64> {
    let normalized = normalize_cron_fields(schedule);
    let sched = Schedule::from_str(&normalized)
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

fn compute_next_interval(spec: &str, now_ts: i64) -> Result<i64> {
    // Format: <seconds> or <seconds>@<anchor_unix_ts>
    let (seconds_str, anchor_str) = match spec.split_once('@') {
        Some((s, a)) => (s, Some(a)),
        None => (spec, None),
    };

    let interval: i64 = seconds_str
        .parse()
        .context("invalid interval seconds in every: schedule")?;
    if interval <= 0 {
        return Err(anyhow!("interval must be positive"));
    }

    let anchor = match anchor_str {
        Some(a) if !a.is_empty() => a.parse::<i64>().context("invalid anchor timestamp")?,
        _ => now_ts,
    };

    if anchor > now_ts {
        // Anchor is in the future — next run is the anchor itself.
        return Ok(anchor);
    }

    let elapsed = now_ts - anchor;
    let periods = elapsed / interval;
    let next = anchor + (periods + 1) * interval;
    Ok(next)
}

fn compute_next_cron_tz(spec: &str, now_ts: i64) -> Result<i64> {
    // Format: <cron_expr>@<IANA_tz>
    let (cron_part, tz_str) = spec
        .rsplit_once('@')
        .ok_or_else(|| anyhow!("cron:<expr>@<tz> requires @ separator and IANA timezone"))?;

    let tz: chrono_tz::Tz = tz_str
        .parse()
        .map_err(|_| anyhow!("invalid IANA timezone: {tz_str}"))?;

    let normalized = normalize_cron_fields(cron_part);
    let sched = Schedule::from_str(&normalized)
        .with_context(|| format!("invalid cron schedule: {cron_part}"))?;

    let now_utc = Utc
        .timestamp_opt(now_ts, 0)
        .single()
        .ok_or_else(|| anyhow!("invalid timestamp: {now_ts}"))?;

    let now_local = now_utc.with_timezone(&tz);

    let next = sched
        .after(&now_local)
        .next()
        .ok_or_else(|| anyhow!("cron has no next occurrence"))?;

    Ok(next.with_timezone(&Utc).timestamp())
}

fn compute_backoff(base_seconds: i64, attempts: i64) -> i64 {
    let base = base_seconds.max(1);
    let exp = (attempts.saturating_sub(1)).min(10) as u32;
    let mult = 1_i64 << exp;
    (base.saturating_mul(mult)).min(60 * 60) // cap at 1h
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
              id              INTEGER PRIMARY KEY AUTOINCREMENT,
              name            TEXT NOT NULL,
              schedule        TEXT NOT NULL,
              kind            TEXT NOT NULL,
              payload         TEXT NOT NULL,
              enabled         INTEGER NOT NULL DEFAULT 1,
              attempts        INTEGER NOT NULL DEFAULT 0,
              max_attempts    INTEGER NOT NULL DEFAULT 3,
              backoff_seconds INTEGER NOT NULL DEFAULT 30,
              last_run_at     INTEGER,
              next_run_at     INTEGER,
              last_status     TEXT,
              last_error      TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_jobs_next_run ON jobs(enabled, next_run_at);

            CREATE TABLE IF NOT EXISTS dead_letters (
              id           INTEGER PRIMARY KEY AUTOINCREMENT,
              job_id       INTEGER NOT NULL,
              failed_at    INTEGER NOT NULL,
              error        TEXT NOT NULL,
              job_snapshot TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_dead_letters_job ON dead_letters(job_id, failed_at);

            CREATE TABLE IF NOT EXISTS runs (
              id         INTEGER PRIMARY KEY AUTOINCREMENT,
              job_id     INTEGER NOT NULL,
              started_at INTEGER NOT NULL,
              ended_at   INTEGER,
              status     TEXT NOT NULL,
              error      TEXT,
              metadata   TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_runs_job ON runs(job_id, started_at);
            "#,
        )?;

        // Handle upgrades from older schema.
        ensure_job_columns(&conn)?;
        Ok(())
    })
    .await?
}

fn ensure_job_columns(conn: &rusqlite::Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(jobs)")?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let has = |name: &str| cols.iter().any(|c| c == name);

    if !has("attempts") {
        conn.execute(
            "ALTER TABLE jobs ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !has("max_attempts") {
        conn.execute(
            "ALTER TABLE jobs ADD COLUMN max_attempts INTEGER NOT NULL DEFAULT 3",
            [],
        )?;
    }
    if !has("backoff_seconds") {
        conn.execute(
            "ALTER TABLE jobs ADD COLUMN backoff_seconds INTEGER NOT NULL DEFAULT 30",
            [],
        )?;
    }
    if !has("last_status") {
        conn.execute("ALTER TABLE jobs ADD COLUMN last_status TEXT", [])?;
    }

    Ok(())
}

async fn normalize_next_run_at(db_path: &Path) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT id, schedule FROM jobs WHERE enabled = 1 AND (next_run_at IS NULL OR next_run_at = 0)",
        )?;
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
            "SELECT id, name, schedule, kind, payload, enabled, attempts, max_attempts, backoff_seconds, last_run_at, next_run_at, last_status, last_error FROM jobs ORDER BY id ASC",
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
                    attempts: row.get(6)?,
                    max_attempts: row.get(7)?,
                    backoff_seconds: row.get(8)?,
                    last_run_at: row.get(9)?,
                    next_run_at: row.get(10)?,
                    last_status: row.get(11)?,
                    last_error: row.get(12)?,
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
            "SELECT id, name, schedule, kind, payload, enabled, attempts, max_attempts, backoff_seconds, last_run_at, next_run_at, last_status, last_error FROM jobs WHERE id = ?1",
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
                attempts: row.get(6)?,
                max_attempts: row.get(7)?,
                backoff_seconds: row.get(8)?,
                last_run_at: row.get(9)?,
                next_run_at: row.get(10)?,
                last_status: row.get(11)?,
                last_error: row.get(12)?,
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
    max_attempts: Option<i64>,
    backoff_seconds: Option<i64>,
) -> Result<i64> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<i64> {
        let conn = rusqlite::Connection::open(path)?;

        let now = Utc::now().timestamp();
        let next = compute_next_run_at(&schedule, now)?;

        let payload_str = serde_json::to_string(&payload)?;
        let max_attempts = max_attempts.unwrap_or(3).max(1);
        let backoff_seconds = backoff_seconds.unwrap_or(30).max(1);

        conn.execute(
            "INSERT INTO jobs (name, schedule, kind, payload, enabled, attempts, max_attempts, backoff_seconds, next_run_at) VALUES (?1, ?2, ?3, ?4, 1, 0, ?5, ?6, ?7)",
            rusqlite::params![name, schedule, kind, payload_str, max_attempts, backoff_seconds, next],
        )?;

        Ok(conn.last_insert_rowid())
    })
    .await?
}

#[allow(clippy::too_many_arguments)]
async fn edit_job_db(
    db_path: &Path,
    id: i64,
    name: Option<String>,
    schedule: Option<String>,
    kind: Option<String>,
    payload: Option<Value>,
    max_attempts: Option<i64>,
    backoff_seconds: Option<i64>,
) -> Result<()> {
    // Read current job, merge, write back.
    let job = get_job(db_path, id)
        .await?
        .ok_or_else(|| anyhow!("job {id} not found"))?;

    let new_name = name.unwrap_or(job.name);
    let new_schedule = schedule.unwrap_or(job.schedule);
    let new_kind = kind.unwrap_or(job.kind);
    let new_payload = payload.unwrap_or(job.payload);
    let new_max_attempts = max_attempts.unwrap_or(job.max_attempts);
    let new_backoff_seconds = backoff_seconds.unwrap_or(job.backoff_seconds);

    // Recompute next_run_at if the job is enabled.
    let next = if job.enabled {
        Some(compute_next_run_at(&new_schedule, Utc::now().timestamp())?)
    } else {
        job.next_run_at
    };

    let payload_str = serde_json::to_string(&new_payload)?;
    let path = db_path.to_owned();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "UPDATE jobs SET name=?1, schedule=?2, kind=?3, payload=?4, max_attempts=?5, backoff_seconds=?6, next_run_at=?7 WHERE id=?8",
            rusqlite::params![new_name, new_schedule, new_kind, payload_str, new_max_attempts, new_backoff_seconds, next, id],
        )?;
        Ok(())
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

async fn clear_jobs(db_path: &Path) -> Result<usize> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<usize> {
        let conn = rusqlite::Connection::open(path)?;
        let n = conn.execute("DELETE FROM jobs", [])?;
        Ok(n)
    })
    .await?
}

async fn set_job_enabled(db_path: &Path, id: i64, enabled: bool) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;

        if enabled {
            // compute next
            let schedule: String =
                conn.query_row("SELECT schedule FROM jobs WHERE id = ?1", [id], |r| {
                    r.get(0)
                })?;
            let now = Utc::now().timestamp();
            let next = compute_next_run_at(&schedule, now)?;
            conn.execute(
                "UPDATE jobs SET enabled = 1, next_run_at = ?1, attempts = 0, last_status = 'resumed', last_error = NULL WHERE id = ?2",
                rusqlite::params![next, id],
            )?;
        } else {
            conn.execute(
                "UPDATE jobs SET enabled = 0, next_run_at = NULL, last_status = 'paused' WHERE id = ?1",
                [id],
            )?;
        }

        Ok(())
    })
    .await?
}

async fn due_jobs(db_path: &Path, now_ts: i64) -> Result<Vec<Job>> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<Vec<Job>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, schedule, kind, payload, enabled, attempts, max_attempts, backoff_seconds, last_run_at, next_run_at, last_status, last_error FROM jobs WHERE enabled = 1 AND next_run_at IS NOT NULL AND next_run_at <= ?1 ORDER BY next_run_at ASC LIMIT 25",
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
                    attempts: row.get(6)?,
                    max_attempts: row.get(7)?,
                    backoff_seconds: row.get(8)?,
                    last_run_at: row.get(9)?,
                    next_run_at: row.get(10)?,
                    last_status: row.get(11)?,
                    last_error: row.get(12)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(jobs)
    })
    .await?
}

#[allow(clippy::too_many_arguments)]
async fn update_job_after_run(
    db_path: &Path,
    id: i64,
    last_run_at: i64,
    next_run_at: Option<i64>,
    last_status: Option<String>,
    last_error: Option<String>,
    attempts: i64,
    keep_enabled: bool,
) -> Result<()> {
    let path = db_path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        let enabled = if keep_enabled { 1 } else { 0 };
        conn.execute(
            "UPDATE jobs SET last_run_at = ?1, next_run_at = ?2, last_status = ?3, last_error = ?4, attempts = ?5, enabled = ?6 WHERE id = ?7",
            rusqlite::params![last_run_at, next_run_at, last_status, last_error, attempts, enabled, id],
        )?;
        Ok(())
    })
    .await?
}

async fn insert_dead_letter(db_path: &Path, job: &Job, failed_at: i64, error: &str) -> Result<()> {
    let path = db_path.to_owned();
    let job_snapshot = serde_json::to_value(job)?;
    let job_snapshot_str = serde_json::to_string(&job_snapshot)?;
    let error = error.to_string();
    let job_id = job.id;

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "INSERT INTO dead_letters (job_id, failed_at, error, job_snapshot) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![job_id, failed_at, error, job_snapshot_str],
        )?;
        Ok(())
    })
    .await?
}

async fn insert_run(
    db_path: &Path,
    job_id: i64,
    started_at: i64,
    ended_at: Option<i64>,
    status: &str,
    error: Option<&str>,
    metadata: Option<&Value>,
) -> Result<()> {
    let path = db_path.to_owned();
    let status = status.to_string();
    let error = error.map(|s| s.to_string());
    let metadata_str = metadata.map(|v| serde_json::to_string(v).unwrap_or_default());

    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "INSERT INTO runs (job_id, started_at, ended_at, status, error, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![job_id, started_at, ended_at, status, error, metadata_str],
        )?;
        Ok(())
    })
    .await?
}

async fn list_runs_db(db_path: &Path, job_id: Option<i64>, limit: usize) -> Result<Vec<Run>> {
    let path = db_path.to_owned();
    let limit = limit.clamp(1, 500) as i64;
    tokio::task::spawn_blocking(move || -> Result<Vec<Run>> {
        let conn = rusqlite::Connection::open(path)?;
        let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match job_id {
            Some(jid) => (
                "SELECT id, job_id, started_at, ended_at, status, error, metadata FROM runs WHERE job_id = ?1 ORDER BY id DESC LIMIT ?2".to_string(),
                vec![Box::new(jid) as Box<dyn rusqlite::types::ToSql>, Box::new(limit)],
            ),
            None => (
                "SELECT id, job_id, started_at, ended_at, status, error, metadata FROM runs ORDER BY id DESC LIMIT ?1".to_string(),
                vec![Box::new(limit) as Box<dyn rusqlite::types::ToSql>],
            ),
        };

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let metadata_str: Option<String> = row.get(6)?;
                let metadata: Option<Value> =
                    metadata_str.and_then(|s| serde_json::from_str(&s).ok());
                Ok(Run {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    status: row.get(4)?,
                    error: row.get(5)?,
                    metadata,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    })
    .await?
}

async fn list_dead_letters(db_path: &Path, limit: usize) -> Result<Vec<DeadLetter>> {
    let path = db_path.to_owned();
    let limit = limit.clamp(1, 500) as i64;
    tokio::task::spawn_blocking(move || -> Result<Vec<DeadLetter>> {
        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare(
            "SELECT id, job_id, failed_at, error, job_snapshot FROM dead_letters ORDER BY id DESC LIMIT ?1",
        )?;

        let rows = stmt
            .query_map([limit], |row| {
                let job_snapshot_str: String = row.get(4)?;
                let job_snapshot: Value =
                    serde_json::from_str(&job_snapshot_str).unwrap_or(Value::Null);
                Ok(DeadLetter {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    failed_at: row.get(2)?,
                    error: row.get(3)?,
                    job_snapshot,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_next_run_at_basic() {
        let now = 1_700_000_000i64;
        let next = compute_next_run_at("* * * * * *", now).unwrap();
        assert!(next > now);
    }

    #[test]
    fn cron_5_field_normalization() {
        let now = 1_700_000_000i64;
        // 5-field: every minute at second 0
        let next = compute_next_run_at("* * * * *", now).unwrap();
        assert!(next > now);
        // Should be at second 0 of the next minute
        assert_eq!(next % 60, 0);
    }

    #[test]
    fn interval_schedule_basic() {
        let now = 1_700_000_000i64;
        let next = compute_next_run_at("every:60", now).unwrap();
        // With no anchor, anchor=now, so next = now + 60
        assert_eq!(next, now + 60);
    }

    #[test]
    fn interval_schedule_with_anchor() {
        let anchor = 1_700_000_000i64;
        let now = anchor + 125; // 2 full intervals of 60 + 5 seconds in
        let next = compute_next_run_at(&format!("every:60@{anchor}"), now).unwrap();
        // next should be anchor + 3*60 = anchor + 180
        assert_eq!(next, anchor + 180);
    }

    #[test]
    fn interval_schedule_anchor_in_future() {
        let now = 1_700_000_000i64;
        let anchor = now + 500;
        let next = compute_next_run_at(&format!("every:60@{anchor}"), now).unwrap();
        assert_eq!(next, anchor);
    }

    #[test]
    fn cron_tz_schedule() {
        let now = 1_700_000_000i64;
        let next = compute_next_run_at("cron:* * * * * *@America/New_York", now).unwrap();
        assert!(next > now);
    }

    #[test]
    fn cron_tz_5_field() {
        let now = 1_700_000_000i64;
        let next = compute_next_run_at("cron:* * * * *@Europe/London", now).unwrap();
        assert!(next > now);
    }

    #[test]
    fn backoff_increases() {
        let b1 = compute_backoff(10, 1);
        let b2 = compute_backoff(10, 2);
        let b3 = compute_backoff(10, 3);
        assert_eq!(b1, 10);
        assert_eq!(b2, 20);
        assert_eq!(b3, 40);
    }
}

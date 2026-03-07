use std::{
    collections::HashMap,
    fs,
    io::Read as _,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use magicmerlin_auto_reply::{format_reply, parse_slash_command, Platform, SlashCommand};
use magicmerlin_compat::{
    providers::{SnapshotBackedProviders, StatusProvider, ToolRegistryProvider},
    COMPAT_VERSION,
};
use magicmerlin_config::{ConfigManager, ConfigOptions};
use magicmerlin_gateway::{methods::SUPPORTED_METHODS, pairing};
use magicmerlin_logging::{init_with as init_logging, LogLevel};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use tracing::info;

mod approvals;
mod plugins;
mod scheduler;
mod service;
mod sessions;

use scheduler::{default_db_path, DeadLetter, Scheduler};

#[derive(Parser, Debug)]
#[command(name = "magicmerlin-gateway")]
#[command(about = "MagicMerlin gateway (compat-first)")]
struct Args {
    /// Print compat version + snapshot fingerprint and exit.
    #[arg(long)]
    print_compat: bool,

    /// Serve a minimal HTTP API backed by snapshots.
    ///
    /// Example: --serve 8080
    #[arg(long)]
    serve: Option<u16>,

    /// Address to bind the HTTP server to.
    ///
    /// Use 0.0.0.0 for LAN access.
    #[arg(long, default_value = "127.0.0.1")]
    bind: IpAddr,

    /// SQLite DB path (defaults to ./magicmerlin.db, or MAGICMERLIN_DB_PATH env)
    #[arg(long)]
    db_path: Option<PathBuf>,

    /// Start the scheduler loop alongside the HTTP server (requires --serve).
    #[arg(long)]
    daemon: bool,

    /// Emit JSON output for --print-compat.
    #[arg(long)]
    json: bool,

    /// Logging level: silent|fatal|error|warn|info|debug|trace.
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Disable ANSI colors in console logs.
    #[arg(long)]
    no_color: bool,

    /// OpenClaw profile name (`~/.openclaw-<profile>`).
    #[arg(long)]
    profile: Option<String>,

    /// Use development profile (`~/.openclaw-dev`).
    #[arg(long)]
    dev: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print combined compat + scheduler state
    Status {
        /// Emit JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage cron jobs
    Cron {
        #[command(subcommand)]
        command: CronCommand,
    },

    /// Manage sessions
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },

    /// Manage approvals
    Approvals {
        #[command(subcommand)]
        command: ApprovalsCommand,
    },

    /// Manage plugins
    Plugins {
        #[command(subcommand)]
        command: PluginsCommand,
    },

    /// Service management helpers
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
}

#[derive(Subcommand, Debug)]
enum CronCommand {
    /// List jobs
    List {
        #[arg(long)]
        json: bool,
    },

    /// Add a job
    Add {
        #[arg(long)]
        name: String,

        /// Cron expression (UTC), interval (every:<s>@<anchor>), or tz-aware (cron:<expr>@<tz>)
        #[arg(long)]
        schedule: String,

        /// Kind: http_get | discord_webhook | discord_bot | agent_turn
        #[arg(long)]
        kind: String,

        /// JSON payload string
        #[arg(long)]
        payload: String,

        /// Maximum retry attempts before dead-lettering the job
        #[arg(long)]
        max_attempts: Option<i64>,

        /// Base backoff seconds (exponential)
        #[arg(long)]
        backoff_seconds: Option<i64>,
    },

    /// Edit a job (update fields by id)
    Edit {
        id: i64,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        schedule: Option<String>,

        #[arg(long)]
        kind: Option<String>,

        #[arg(long)]
        payload: Option<String>,

        #[arg(long)]
        max_attempts: Option<i64>,

        #[arg(long)]
        backoff_seconds: Option<i64>,
    },

    /// Remove a job by id
    Remove { id: i64 },

    /// Remove a job by id (alias for remove)
    Rm { id: i64 },

    /// Trigger a job once, immediately
    Run { id: i64 },

    /// Pause a job (disable)
    Pause { id: i64 },

    /// Disable a job (alias for pause)
    Disable { id: i64 },

    /// Resume a job (enable)
    Resume { id: i64 },

    /// Enable a job (alias for resume)
    Enable { id: i64 },

    /// Show recent run history
    Runs {
        /// Filter by job ID
        #[arg(long)]
        job_id: Option<i64>,

        #[arg(long, default_value_t = 50)]
        limit: usize,

        #[arg(long)]
        json: bool,
    },

    /// Print scheduler state (job count, next run)
    Status {
        #[arg(long)]
        json: bool,
    },

    /// List dead-lettered job failures
    DeadLetters {
        #[arg(long, default_value_t = 50)]
        limit: usize,

        #[arg(long)]
        json: bool,
    },

    /// Export jobs to a JSON file
    Export {
        #[arg(long)]
        file: PathBuf,
    },

    /// Import jobs from a JSON file
    Import {
        #[arg(long)]
        file: PathBuf,

        /// Remove existing jobs before importing
        #[arg(long)]
        replace: bool,
    },

    /// Import OpenClaw cron jobs (from `openclaw cron list --json`)
    #[command(name = "import-openclaw")]
    ImportOpenclaw {
        /// Path to JSON file
        #[arg(long)]
        file: Option<PathBuf>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SessionsCommand {
    /// List sessions
    List {
        #[arg(long)]
        json: bool,

        #[arg(long, default_value_t = 50)]
        limit: usize,
    },

    /// Show a single session
    Show {
        id: String,

        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ApprovalsCommand {
    /// Get current approvals and allowlist
    Get {
        #[arg(long)]
        json: bool,
    },

    /// Set approvals from a JSON file
    Set {
        #[arg(long)]
        file: PathBuf,
    },

    /// Manage the approval allowlist
    Allowlist {
        #[command(subcommand)]
        command: AllowlistCommand,
    },
}

#[derive(Subcommand, Debug)]
enum AllowlistCommand {
    /// Add a pattern to the allowlist
    Add {
        pattern: String,

        /// Agent scope (default: '*')
        #[arg(long)]
        agent: Option<String>,
    },

    /// Remove a pattern from the allowlist
    Remove {
        pattern: String,

        /// Agent scope (default: '*')
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum PluginsCommand {
    /// List registered plugins
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ServiceCommand {
    /// Print generated LaunchAgent plist
    Launchagent {
        #[arg(long)]
        gateway_bin: Option<PathBuf>,
    },
    /// Install LaunchAgent plist into ~/Library/LaunchAgents
    InstallLaunchagent {
        #[arg(long)]
        gateway_bin: Option<PathBuf>,
    },
    /// Uninstall LaunchAgent plist
    UninstallLaunchagent,
    /// Print generated systemd user unit
    Systemd {
        #[arg(long)]
        gateway_bin: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompatInfo {
    compat_version: &'static str,
    fingerprint: String,
    snapshot_hashes: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortableJob {
    name: String,
    schedule: String,
    kind: String,
    payload: serde_json::Value,
    enabled: Option<bool>,
    max_attempts: Option<i64>,
    backoff_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortableJobsFile {
    version: String,
    jobs: Vec<PortableJob>,
}

// ---------------------------------------------------------------------------
// OpenClaw import types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawCronList {
    jobs: Vec<OpenClawJob>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawJob {
    name: Option<String>,
    id: Option<String>,
    schedule: OpenClawSchedule,
    payload: OpenClawPayload,
    enabled: Option<bool>,
    #[serde(default)]
    max_attempts: Option<i64>,
    #[serde(default)]
    backoff_seconds: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawSchedule {
    kind: String,
    // For kind=cron — field is "expr" in OpenClaw JSON
    #[serde(alias = "expression")]
    expr: Option<String>,
    // For kind=cron — field is "tz" in OpenClaw JSON
    #[serde(alias = "timezone")]
    tz: Option<String>,
    // For kind=every
    every_ms: Option<u64>,
    anchor_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawPayload {
    kind: String,
    message: Option<String>,
    timeout_seconds: Option<u64>,
    model: Option<String>,
    thinking: Option<serde_json::Value>,
}

fn convert_openclaw_schedule(sched: &OpenClawSchedule) -> Result<String> {
    match sched.kind.as_str() {
        "every" => {
            let every_ms = sched
                .every_ms
                .ok_or_else(|| anyhow::anyhow!("every schedule missing everyMs"))?;
            let seconds = every_ms / 1000;
            if seconds == 0 {
                return Err(anyhow::anyhow!("everyMs must be >= 1000"));
            }
            match sched.anchor_ms {
                Some(anchor_ms) => {
                    let anchor_ts = (anchor_ms / 1000) as i64;
                    Ok(format!("every:{seconds}@{anchor_ts}"))
                }
                None => Ok(format!("every:{seconds}")),
            }
        }
        "cron" => {
            let expr = sched
                .expr
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("cron schedule missing expr"))?;
            match &sched.tz {
                Some(tz) if !tz.is_empty() => Ok(format!("cron:{expr}@{tz}")),
                _ => Ok(expr.to_string()),
            }
        }
        other => Err(anyhow::anyhow!("unknown OpenClaw schedule kind: {other}")),
    }
}

fn convert_openclaw_job(oc: OpenClawJob, index: usize) -> Result<PortableJob> {
    let schedule = convert_openclaw_schedule(&oc.schedule)?;

    let (kind, payload) = match oc.payload.kind.as_str() {
        "agentTurn" => {
            let mut map = serde_json::Map::new();
            if let Some(msg) = oc.payload.message {
                map.insert("message".to_string(), serde_json::Value::String(msg));
            }
            if let Some(ts) = oc.payload.timeout_seconds {
                map.insert(
                    "timeoutSeconds".to_string(),
                    serde_json::Value::Number(ts.into()),
                );
            }
            if let Some(model) = oc.payload.model {
                map.insert("model".to_string(), serde_json::Value::String(model));
            }
            if let Some(thinking) = oc.payload.thinking {
                map.insert("thinking".to_string(), thinking);
            }
            ("agent_turn".to_string(), serde_json::Value::Object(map))
        }
        other => {
            // Pass through as-is — unknown payload kinds become the kind field.
            let payload = serde_json::json!({
                "originalKind": other,
                "message": oc.payload.message,
            });
            (other.to_string(), payload)
        }
    };

    let name = oc
        .name
        .or(oc.id)
        .unwrap_or_else(|| format!("openclaw-import-{index}"));

    Ok(PortableJob {
        name,
        schedule,
        kind,
        payload,
        enabled: oc.enabled,
        max_attempts: oc.max_attempts,
        backoff_seconds: oc.backoff_seconds,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = ConfigManager::load(ConfigOptions {
        profile: args.profile.clone(),
        dev: args.dev,
    })?;
    let log_level = LogLevel::from_str(&args.log_level).unwrap_or(LogLevel::Info);
    init_logging(
        log_level,
        !args.no_color,
        Some(&config.state_paths().logs_dir),
    )?;
    info!(
        state_dir = %config.state_paths().state_dir.display(),
        config_path = %config.config_path().display(),
        "initialized gateway runtime"
    );

    // Always load snapshots early; if this fails, we are not compatible.
    let providers = SnapshotBackedProviders::load()?;
    let hashes = providers.hashes()?;

    let info = CompatInfo {
        compat_version: COMPAT_VERSION,
        fingerprint: hashes.fingerprint.clone(),
        snapshot_hashes: hashes.files.clone(),
    };

    // Back-compat: --print-compat
    if args.print_compat {
        if args.json {
            println!("{}", serde_json::to_string_pretty(&info)?);
        } else {
            println!("compat_version={}", info.compat_version);
            println!("fingerprint={}", info.fingerprint);
            println!("snapshots={}", info.snapshot_hashes.len());
        }
        return Ok(());
    }

    let db_path = args.db_path.clone().unwrap_or_else(default_db_path);
    let auth = Arc::new(resolve_gateway_auth(&config));
    let config = Arc::new(Mutex::new(config));

    // CLI subcommands.
    if let Some(cmd) = args.command {
        let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);

        match cmd {
            Command::Service { command } => {
                let cfg = config.lock().await;
                let port = args.serve.unwrap_or_else(|| {
                    cfg.config()
                        .gateway
                        .port
                        .unwrap_or(if args.dev { 19001 } else { 18789 })
                });
                let state_dir = cfg.state_paths().state_dir.clone();
                drop(cfg);

                match command {
                    ServiceCommand::Launchagent { gateway_bin } => {
                        let bin = gateway_bin.unwrap_or_else(default_gateway_bin);
                        let plist = service::generate_launchagent_plist(&bin, &state_dir, port);
                        println!("{plist}");
                    }
                    ServiceCommand::InstallLaunchagent { gateway_bin } => {
                        let bin = gateway_bin.unwrap_or_else(default_gateway_bin);
                        let plist = service::generate_launchagent_plist(&bin, &state_dir, port);
                        let path = service::install_launchagent(&plist)?;
                        println!("{}", path.display());
                    }
                    ServiceCommand::UninstallLaunchagent => {
                        let path = service::uninstall_launchagent()?;
                        println!("{}", path.display());
                    }
                    ServiceCommand::Systemd { gateway_bin } => {
                        let bin = gateway_bin.unwrap_or_else(default_gateway_bin);
                        let unit = service::generate_systemd_unit(&bin, port);
                        println!("{unit}");
                    }
                }
                return Ok(());
            }
            Command::Status { json } => {
                let state = scheduler.state().await?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                          "compat": {
                            "compatVersion": info.compat_version,
                            "fingerprint": info.fingerprint,
                          },
                          "scheduler": state,
                        }))?
                    );
                } else {
                    println!("compatVersion={}", info.compat_version);
                    println!("fingerprint={}", info.fingerprint);
                    println!("jobs={}", state.job_count);
                    println!("nextRunAt={:?}", state.next_run_at);
                }
                return Ok(());
            }

            Command::Sessions { command } => {
                // Ensure sessions table exists.
                sessions::migrate_sessions(&db_path).await?;

                match command {
                    SessionsCommand::List { json, limit } => {
                        let rows = sessions::list_sessions(&db_path, limit).await?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(
                                    &serde_json::json!({"sessions": rows})
                                )?
                            );
                        } else {
                            for s in rows {
                                println!(
                                    "{}\t{}\t{}\t{}\t{}",
                                    s.id,
                                    s.status,
                                    s.agent.as_deref().unwrap_or("-"),
                                    s.started_at,
                                    s.updated_at
                                );
                            }
                        }
                    }
                    SessionsCommand::Show { id, json } => {
                        let session = sessions::get_session(&db_path, &id).await?;
                        match session {
                            Some(s) => {
                                if json {
                                    println!("{}", serde_json::to_string_pretty(&s)?);
                                } else {
                                    println!("id={}", s.id);
                                    println!("status={}", s.status);
                                    println!("agent={}", s.agent.as_deref().unwrap_or("-"));
                                    println!("startedAt={}", s.started_at);
                                    println!("updatedAt={}", s.updated_at);
                                }
                            }
                            None => {
                                eprintln!("session not found: {id}");
                                std::process::exit(1);
                            }
                        }
                    }
                }
                return Ok(());
            }

            Command::Approvals { command } => {
                // Ensure approvals tables exist.
                approvals::migrate_approvals(&db_path).await?;

                match command {
                    ApprovalsCommand::Get { json } => {
                        let state = approvals::get_approvals(&db_path).await?;
                        if json {
                            println!("{}", serde_json::to_string_pretty(&state)?);
                        } else {
                            if state.approvals.is_empty() && state.allowlist.is_empty() {
                                println!("(no approvals configured)");
                            }
                            for a in &state.approvals {
                                println!("approval\tagent={}\t{}={}", a.agent, a.key, a.value);
                            }
                            for e in &state.allowlist {
                                println!("allowlist\tagent={}\t{}", e.agent, e.pattern);
                            }
                        }
                    }
                    ApprovalsCommand::Set { file } => {
                        let count = approvals::set_approvals_from_file(&db_path, &file).await?;
                        println!("{count}");
                    }
                    ApprovalsCommand::Allowlist { command: al_cmd } => match al_cmd {
                        AllowlistCommand::Add { pattern, agent } => {
                            approvals::allowlist_add(&db_path, &pattern, agent.as_deref()).await?;
                            println!("ok");
                        }
                        AllowlistCommand::Remove { pattern, agent } => {
                            approvals::allowlist_remove(&db_path, &pattern, agent.as_deref())
                                .await?;
                            println!("ok");
                        }
                    },
                }
                return Ok(());
            }

            Command::Plugins { command } => {
                match command {
                    PluginsCommand::List { json } => {
                        let reg = plugins::load_registry()?;
                        if json {
                            println!("{}", serde_json::to_string_pretty(&reg)?);
                        } else {
                            if reg.plugins.is_empty() {
                                println!("(no plugins registered)");
                            }
                            for p in &reg.plugins {
                                println!(
                                    "{}\t{}\t{}",
                                    p.name,
                                    p.version.as_deref().unwrap_or("-"),
                                    if p.enabled.unwrap_or(true) {
                                        "enabled"
                                    } else {
                                        "disabled"
                                    }
                                );
                            }
                        }
                    }
                }
                return Ok(());
            }

            Command::Cron { command } => {
                match command {
                    CronCommand::List { json } => {
                        let jobs = scheduler.list_jobs().await?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({"jobs": jobs}))?
                            );
                        } else {
                            for j in jobs {
                                println!(
                                    "{}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}",
                                    j.id,
                                    j.name,
                                    j.kind,
                                    j.enabled,
                                    j.schedule,
                                    j.next_run_at,
                                    j.attempts,
                                    j.last_status.clone().unwrap_or_default()
                                );
                            }
                        }
                    }
                    CronCommand::Add {
                        name,
                        schedule,
                        kind,
                        payload,
                        max_attempts,
                        backoff_seconds,
                    } => {
                        let payload_json: serde_json::Value = serde_json::from_str(&payload)?;
                        let id = scheduler
                            .add_job(
                                name,
                                schedule,
                                kind,
                                payload_json,
                                max_attempts,
                                backoff_seconds,
                            )
                            .await?;
                        println!("{id}");
                    }
                    CronCommand::Edit {
                        id,
                        name,
                        schedule,
                        kind,
                        payload,
                        max_attempts,
                        backoff_seconds,
                    } => {
                        let payload_json = match payload {
                            Some(p) => Some(serde_json::from_str(&p)?),
                            None => None,
                        };
                        scheduler
                            .edit_job(
                                id,
                                name,
                                schedule,
                                kind,
                                payload_json,
                                max_attempts,
                                backoff_seconds,
                            )
                            .await?;
                        println!("ok");
                    }
                    CronCommand::Remove { id } | CronCommand::Rm { id } => {
                        scheduler.remove_job(id).await?;
                        println!("ok");
                    }
                    CronCommand::Run { id } => {
                        scheduler.run_job_now(id).await?;
                        println!("ok");
                    }
                    CronCommand::Pause { id } | CronCommand::Disable { id } => {
                        scheduler.pause_job(id).await?;
                        println!("ok");
                    }
                    CronCommand::Resume { id } | CronCommand::Enable { id } => {
                        scheduler.resume_job(id).await?;
                        println!("ok");
                    }
                    CronCommand::Runs {
                        job_id,
                        limit,
                        json,
                    } => {
                        let rows = scheduler.list_runs(job_id, limit).await?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({"runs": rows}))?
                            );
                        } else {
                            for r in rows {
                                println!(
                                    "{}\tjob={}\tstarted={}\tended={:?}\t{}\t{}",
                                    r.id,
                                    r.job_id,
                                    r.started_at,
                                    r.ended_at,
                                    r.status,
                                    r.error.unwrap_or_default()
                                );
                            }
                        }
                    }
                    CronCommand::Status { json } => {
                        let state = scheduler.state().await?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "scheduler": state,
                                }))?
                            );
                        } else {
                            println!("jobs={}", state.job_count);
                            println!("nextRunAt={:?}", state.next_run_at);
                        }
                    }
                    CronCommand::DeadLetters { limit, json } => {
                        let rows = scheduler.list_dead_letters(limit).await?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(
                                    &serde_json::json!({"deadLetters": rows})
                                )?
                            );
                        } else {
                            for r in rows {
                                println!(
                                    "{}\tjob={}\tfailed_at={}\t{}",
                                    r.id, r.job_id, r.failed_at, r.error
                                );
                            }
                        }
                    }
                    CronCommand::Export { file } => {
                        let jobs = scheduler.list_jobs().await?;
                        let portable = PortableJobsFile {
                            version: "v1".to_string(),
                            jobs: jobs
                                .into_iter()
                                .map(|j| PortableJob {
                                    name: j.name,
                                    schedule: j.schedule,
                                    kind: j.kind,
                                    payload: j.payload,
                                    enabled: Some(j.enabled),
                                    max_attempts: Some(j.max_attempts),
                                    backoff_seconds: Some(j.backoff_seconds),
                                })
                                .collect(),
                        };

                        let body = serde_json::to_string_pretty(&portable)?;
                        fs::write(&file, body)
                            .with_context(|| format!("write export file: {}", file.display()))?;
                        println!("ok");
                    }
                    CronCommand::Import { file, replace } => {
                        let raw = fs::read_to_string(&file)
                            .with_context(|| format!("read import file: {}", file.display()))?;

                        let parsed: PortableJobsFile = match serde_json::from_str(&raw) {
                            Ok(v) => v,
                            Err(_) => {
                                // Back-compat: allow plain array of jobs.
                                let jobs: Vec<PortableJob> = serde_json::from_str(&raw)
                                    .with_context(|| "invalid import JSON format")?;
                                PortableJobsFile {
                                    version: "v1".to_string(),
                                    jobs,
                                }
                            }
                        };

                        if replace {
                            let _ = scheduler.clear_jobs().await?;
                        }

                        let mut imported = 0usize;
                        for j in parsed.jobs {
                            let id = scheduler
                                .add_job(
                                    j.name,
                                    j.schedule,
                                    j.kind,
                                    j.payload,
                                    j.max_attempts,
                                    j.backoff_seconds,
                                )
                                .await?;
                            if matches!(j.enabled, Some(false)) {
                                scheduler.pause_job(id).await?;
                            }
                            imported += 1;
                        }

                        println!("{imported}");
                    }
                    CronCommand::ImportOpenclaw { file, stdin } => {
                        let raw = if stdin {
                            let mut buf = String::new();
                            std::io::stdin()
                                .read_to_string(&mut buf)
                                .context("read stdin")?;
                            buf
                        } else if let Some(path) = file {
                            fs::read_to_string(&path)
                                .with_context(|| format!("read file: {}", path.display()))?
                        } else {
                            anyhow::bail!("import-openclaw requires --file <path> or --stdin");
                        };

                        let oc_list: OpenClawCronList =
                            serde_json::from_str(&raw).context("parse OpenClaw cron list JSON")?;

                        let mut imported = 0usize;
                        let mut errors = Vec::new();
                        for (i, oc_job) in oc_list.jobs.into_iter().enumerate() {
                            let job_name = oc_job
                                .name
                                .clone()
                                .or_else(|| oc_job.id.clone())
                                .unwrap_or_else(|| format!("job-{i}"));

                            match convert_openclaw_job(oc_job, i) {
                                Ok(portable) => {
                                    let enabled = portable.enabled;
                                    match scheduler
                                        .add_job(
                                            portable.name,
                                            portable.schedule,
                                            portable.kind,
                                            portable.payload,
                                            portable.max_attempts,
                                            portable.backoff_seconds,
                                        )
                                        .await
                                    {
                                        Ok(id) => {
                                            if matches!(enabled, Some(false)) {
                                                let _ = scheduler.pause_job(id).await;
                                            }
                                            imported += 1;
                                        }
                                        Err(e) => {
                                            errors.push(format!("{job_name}: {e:#}"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    errors.push(format!("{job_name}: {e:#}"));
                                }
                            }
                        }

                        println!("{imported}");
                        for err in &errors {
                            eprintln!("warning: {err}");
                        }
                    }
                }
                return Ok(());
            }
        }
    }

    // Back-compat: --serve
    if let Some(port) = args.serve {
        if args.daemon {
            serve_http_with_daemon(
                args.bind,
                port,
                providers,
                info,
                db_path,
                config.clone(),
                auth.clone(),
            )
            .await?;
        } else {
            serve_http(
                args.bind,
                port,
                providers,
                info,
                db_path,
                config.clone(),
                auth.clone(),
            )
            .await?;
        }
        return Ok(());
    }

    // Default behavior: be explicit (no silent daemon).
    eprintln!(
        "No action provided. Try: status, cron list, sessions list, approvals get, plugins list, --print-compat, or --serve 8080"
    );
    Ok(())
}

#[derive(Clone)]
struct AppState {
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    scheduler: Arc<Scheduler>,
    db_path: PathBuf,
    config: Arc<Mutex<ConfigManager>>,
    auth: Arc<GatewayAuth>,
    events: broadcast::Sender<GatewayEvent>,
    run_queue: Arc<RunQueue>,
    started_at: Instant,
    presence: Arc<Mutex<SystemPresence>>,
}

#[derive(Debug, Clone, Default)]
struct GatewayAuth {
    token: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GatewayEvent {
    method: String,
    params: Value,
    target_client: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemPresence {
    online: bool,
    last_heartbeat_at: i64,
    connected_clients: usize,
}

impl Default for SystemPresence {
    fn default() -> Self {
        Self {
            online: true,
            last_heartbeat_at: chrono::Utc::now().timestamp(),
            connected_clients: 0,
        }
    }
}

#[derive(Clone, Default)]
struct RunQueue {
    session_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    abort_by_session: Arc<Mutex<HashMap<String, tokio::sync::watch::Sender<bool>>>>,
}

impl RunQueue {
    async fn session_lock(&self, session_id: &str) -> Arc<Mutex<()>> {
        let mut locks = self.session_locks.lock().await;
        locks
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    async fn register_abort(&self, session_id: &str) -> tokio::sync::watch::Receiver<bool> {
        let (tx, rx) = tokio::sync::watch::channel(false);
        self.abort_by_session
            .lock()
            .await
            .insert(session_id.to_string(), tx);
        rx
    }

    async fn clear_abort(&self, session_id: &str) {
        self.abort_by_session.lock().await.remove(session_id);
    }

    async fn abort_session(&self, session_id: &str) -> bool {
        let sender = self.abort_by_session.lock().await.get(session_id).cloned();
        if let Some(sender) = sender {
            let _ = sender.send(true);
            return true;
        }
        false
    }
}

async fn serve_http(
    bind: IpAddr,
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    db_path: PathBuf,
    config: Arc<Mutex<ConfigManager>>,
    auth: Arc<GatewayAuth>,
) -> Result<()> {
    let state_dir = {
        let guard = config.lock().await;
        guard.state_paths().state_dir.clone()
    };
    let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);
    sessions::migrate_sessions(&db_path).await?;
    approvals::migrate_approvals(&db_path).await?;
    pairing::migrate_pairing(&db_path).await?;
    let (events, _) = broadcast::channel(256);
    let state = AppState {
        providers,
        info,
        scheduler,
        db_path,
        config,
        auth,
        events,
        run_queue: Arc::new(RunQueue::default()),
        started_at: Instant::now(),
        presence: Arc::new(Mutex::new(SystemPresence::default())),
    };

    let app = build_router(state);

    let addr = SocketAddr::from((bind, port));
    eprintln!("magicmerlin-gateway listening on http://{addr}");
    let pid_file = service::default_pid_file(&state_dir);
    let _ = service::write_pid_file(&pid_file);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    let _ = service::remove_pid_file(&pid_file);
    Ok(())
}

async fn serve_http_with_daemon(
    bind: IpAddr,
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    db_path: PathBuf,
    config: Arc<Mutex<ConfigManager>>,
    auth: Arc<GatewayAuth>,
) -> Result<()> {
    let state_dir = {
        let guard = config.lock().await;
        guard.state_paths().state_dir.clone()
    };
    let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);
    sessions::migrate_sessions(&db_path).await?;
    approvals::migrate_approvals(&db_path).await?;
    pairing::migrate_pairing(&db_path).await?;
    let daemon_handle = scheduler.clone().spawn_daemon();

    let (events, _) = broadcast::channel(256);
    let state = AppState {
        providers,
        info,
        scheduler,
        db_path,
        config,
        auth,
        events,
        run_queue: Arc::new(RunQueue::default()),
        started_at: Instant::now(),
        presence: Arc::new(Mutex::new(SystemPresence::default())),
    };

    let app = build_router(state);

    let addr = SocketAddr::from((bind, port));
    eprintln!("magicmerlin-gateway (daemon) listening on http://{addr}");
    let pid_file = service::default_pid_file(&state_dir);
    let _ = service::write_pid_file(&pid_file);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run server in foreground; scheduler runs in background.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // If server stops, stop scheduler task too.
    daemon_handle.abort();
    let _ = service::remove_pid_file(&pid_file);
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        // Control UI
        .route("/", get(http_index))
        .route("/chat", post(http_chat))
        .route("/methods", get(http_methods))
        .route("/call", post(http_call))
        .route("/ws", post(http_ws))
        .route(
            "/health",
            get({
                let state = state.clone();
                move || async move {
                    Json(serde_json::json!({
                      "status": "ok",
                      "compatVersion": state.info.compat_version,
                      "fingerprint": state.info.fingerprint,
                    }))
                }
            }),
        )
        .route(
            "/status",
            get({
                let state = state.clone();
                move || async move {
                    let sched = state.scheduler.state().await.ok();
                    Json(serde_json::json!({
                      "compat": {
                        "compatVersion": state.info.compat_version,
                        "fingerprint": state.info.fingerprint,
                      },
                      "scheduler": sched,
                      "openclawStatus": state.providers.openclaw_status_json(),
                    }))
                }
            }),
        )
        .route(
            "/tools",
            get({
                let state = state.clone();
                move || async move {
                    Json(serde_json::json!({
                      "tools": state.providers.tool_names(),
                    }))
                }
            }),
        )
        .route(
            "/snapshots",
            get({
                let state = state.clone();
                move || async move { Json(state.info.clone()) }
            }),
        )
        // Cron API (optionally protected by MAGICMERLIN_API_KEY)
        .route("/cron", get(http_cron_list))
        .route("/cron/run/:id", post(http_cron_run))
        .route("/cron/pause/:id", post(http_cron_pause))
        .route("/cron/resume/:id", post(http_cron_resume))
        .route("/cron/dead-letters", get(http_dead_letters))
        // Sessions / Approvals / Plugins API
        .route("/sessions", get(http_sessions_list))
        .route("/sessions/:id", get(http_sessions_show))
        .route("/approvals", get(http_approvals_get))
        .route("/pairing", get(http_pairing_list))
        .route("/pairing/approve", post(http_pairing_approve))
        .route("/pairing/reject", post(http_pairing_reject))
        .route("/pairing/state", get(http_pairing_state))
        .route("/plugins", get(http_plugins_list))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct MethodCallRequest {
    method: String,
    #[serde(default)]
    params: Value,
}

fn call_error_response(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
    method: &str,
    details: Option<Value>,
) -> (StatusCode, Json<Value>) {
    let mut error = serde_json::Map::new();
    error.insert("code".to_string(), Value::String(code.to_string()));
    error.insert("message".to_string(), Value::String(message.into()));
    error.insert("method".to_string(), Value::String(method.to_string()));
    if let Some(details) = details {
        error.insert("details".to_string(), details);
    }
    (
        status,
        Json(serde_json::json!({
            "error": Value::Object(error),
        })),
    )
}

fn parse_params<T: DeserializeOwned>(
    params: Value,
    method: &str,
) -> std::result::Result<T, (StatusCode, Json<Value>)> {
    let normalized = if params.is_null() {
        serde_json::json!({})
    } else {
        params
    };
    serde_json::from_value(normalized).map_err(|err| {
        call_error_response(
            StatusCode::BAD_REQUEST,
            "invalid_params",
            "invalid params",
            method,
            Some(Value::String(err.to_string())),
        )
    })
}

fn parse_approvals_entries(
    params: Value,
) -> std::result::Result<Vec<approvals::ApprovalFileEntry>, String> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Params {
        #[serde(default)]
        approvals: Option<Vec<approvals::ApprovalFileEntry>>,
        #[serde(default)]
        entries: Option<Vec<approvals::ApprovalFileEntry>>,
        #[serde(default)]
        json: Option<Value>,
    }

    let normalized = if params.is_null() {
        serde_json::json!({})
    } else {
        params
    };
    let parsed: Params =
        serde_json::from_value(normalized).map_err(|e| format!("invalid params: {e}"))?;

    let provided = usize::from(parsed.approvals.is_some())
        + usize::from(parsed.entries.is_some())
        + usize::from(parsed.json.is_some());
    if provided == 0 {
        return Err(
            "missing approvals payload: provide one of approvals, entries, or json".to_string(),
        );
    }
    if provided > 1 {
        return Err(
            "ambiguous approvals payload: provide only one of approvals, entries, or json"
                .to_string(),
        );
    }

    if let Some(v) = parsed.approvals {
        return Ok(v);
    }
    if let Some(v) = parsed.entries {
        return Ok(v);
    }

    let json = parsed.json.expect("checked above");
    match json {
        Value::Array(_) => serde_json::from_value(json).map_err(|e| format!("invalid json: {e}")),
        Value::Object(mut obj) => {
            if let Some(v) = obj.remove("approvals") {
                serde_json::from_value(v).map_err(|e| format!("invalid json.approvals: {e}"))
            } else if let Some(v) = obj.remove("entries") {
                serde_json::from_value(v).map_err(|e| format!("invalid json.entries: {e}"))
            } else {
                Err("invalid json: expected array or object with approvals/entries".to_string())
            }
        }
        _ => Err("invalid json: expected array or object".to_string()),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WsAuthQuery {
    token: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonRpcRequest {
    method: String,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    auth: Option<JsonRpcAuth>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonRpcAuth {
    token: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Error)]
enum RpcError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl RpcError {
    fn code(&self) -> i64 {
        match self {
            Self::Unauthorized => -32001,
            Self::InvalidParams(_) => -32602,
            Self::MethodNotFound(_) => -32601,
            Self::Internal(_) => -32603,
        }
    }
}

fn resolve_gateway_auth(config: &ConfigManager) -> GatewayAuth {
    let token = std::env::var("MAGICMERLIN_GATEWAY_TOKEN").ok().or_else(|| {
        config
            .config()
            .gateway
            .extra
            .get("token")
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
    });
    let password = std::env::var("MAGICMERLIN_GATEWAY_PASSWORD")
        .ok()
        .or_else(|| {
            config
                .config()
                .gateway
                .extra
                .get("password")
                .and_then(Value::as_str)
                .map(std::string::ToString::to_string)
        });
    GatewayAuth { token, password }
}

fn default_gateway_bin() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("magicmerlin-gateway"))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            let _ = signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn http_ws(
    State(state): State<AppState>,
    Query(auth_query): Query<WsAuthQuery>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let client_id = uuid::Uuid::new_v4().to_string();
    let id = req.id.clone().unwrap_or(Value::Null);
    if !is_ws_authorized(
        &state,
        req.auth.as_ref(),
        auth_query.token.as_deref(),
        auth_query.password.as_deref(),
    ) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "jsonrpc":"2.0",
                "error": { "code": RpcError::Unauthorized.code(), "message": RpcError::Unauthorized.to_string() },
                "id": id
            })),
        );
    }

    let response = match dispatch_ws_method(&state, &client_id, &req.method, req.params).await {
        Ok(result) => serde_json::json!({"jsonrpc":"2.0","result":result,"id": id}),
        Err(err) => serde_json::json!({
            "jsonrpc":"2.0",
            "error": { "code": err.code(), "message": err.to_string() },
            "id": id
        }),
    };
    (StatusCode::OK, Json(response))
}

fn is_ws_authorized(
    state: &AppState,
    auth: Option<&JsonRpcAuth>,
    query_token: Option<&str>,
    query_password: Option<&str>,
) -> bool {
    if state.auth.token.is_none() && state.auth.password.is_none() {
        return true;
    }

    let token_matches = state.auth.token.as_deref().is_none_or(|required| {
        auth.and_then(|a| a.token.as_deref()) == Some(required) || query_token == Some(required)
    });
    let password_matches = state.auth.password.as_deref().is_none_or(|required| {
        auth.and_then(|a| a.password.as_deref()) == Some(required)
            || query_password == Some(required)
    });

    token_matches && password_matches
}

async fn dispatch_ws_method(
    state: &AppState,
    client_id: &str,
    method: &str,
    params: Value,
) -> std::result::Result<Value, RpcError> {
    match method {
        "health" => Ok(serde_json::json!({
            "ok": true,
            "uptimeSeconds": state.started_at.elapsed().as_secs(),
            "channelStatus": "online",
        })),
        "status" => {
            let scheduler_state = state
                .scheduler
                .state()
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let presence = state.presence.lock().await.clone();
            let config = state.config.lock().await;
            Ok(serde_json::json!({
                "agents": { "count": 1, "default": "merlin" },
                "sessions": sessions::list_sessions(&state.db_path, 100).await.map_err(|e| RpcError::Internal(e.to_string()))?.len(),
                "models": config.config().agents.defaults.model,
                "config": config.config().gateway,
                "scheduler": scheduler_state,
                "presence": presence,
            }))
        }
        "system-presence" => Ok(serde_json::to_value(state.presence.lock().await.clone())
            .map_err(|e| RpcError::Internal(e.to_string()))?),
        "agent.run" => run_agent_turn(state, client_id, params).await,
        "agent.abort" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                session_id: String,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let aborted = state.run_queue.abort_session(&parsed.session_id).await;
            Ok(serde_json::json!({ "ok": true, "aborted": aborted }))
        }
        "sessions.list" => {
            let list = sessions::list_sessions(&state.db_path, 500)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "sessions": list }))
        }
        "sessions.get" => {
            #[derive(Deserialize)]
            struct Params {
                id: String,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let session = sessions::get_session(&state.db_path, &parsed.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "session": session }))
        }
        "sessions.send" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                session_id: String,
                message: String,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            sessions::upsert_session(
                &state.db_path,
                &parsed.session_id,
                Some("gateway"),
                "active",
                Some(&serde_json::json!({ "lastMessage": parsed.message })),
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "sessions.spawn" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                parent_session_id: String,
                child_session_id: Option<String>,
                agent: Option<String>,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let child_id = parsed.child_session_id.unwrap_or_else(|| {
                format!("sub:{}:{}", parsed.parent_session_id, uuid::Uuid::new_v4())
            });
            sessions::spawn_subsession(
                &state.db_path,
                &parsed.parent_session_id,
                &child_id,
                parsed.agent.as_deref(),
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "sessionId": child_id }))
        }
        "sessions.compact" => {
            #[derive(Deserialize)]
            struct Params {
                id: String,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let compacted = sessions::compact_session(&state.db_path, &parsed.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": compacted }))
        }
        "sessions.delete" => {
            #[derive(Deserialize)]
            struct Params {
                id: String,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let deleted = sessions::delete_session(&state.db_path, &parsed.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": deleted }))
        }
        "cron.list" => Ok(serde_json::json!({
            "jobs": state.scheduler.list_jobs().await.map_err(|e| RpcError::Internal(e.to_string()))?
        })),
        "cron.add" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
                schedule: String,
                kind: String,
                payload: Value,
                max_attempts: Option<i64>,
                backoff_seconds: Option<i64>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let id = state
                .scheduler
                .add_job(
                    p.name,
                    p.schedule,
                    p.kind,
                    p.payload,
                    p.max_attempts,
                    p.backoff_seconds,
                )
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "cron.edit" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                id: i64,
                name: Option<String>,
                schedule: Option<String>,
                kind: Option<String>,
                payload: Option<Value>,
                max_attempts: Option<i64>,
                backoff_seconds: Option<i64>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            state
                .scheduler
                .edit_job(
                    p.id,
                    p.name,
                    p.schedule,
                    p.kind,
                    p.payload,
                    p.max_attempts,
                    p.backoff_seconds,
                )
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "cron.rm" => {
            #[derive(Deserialize)]
            struct Params {
                id: i64,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            state
                .scheduler
                .remove_job(p.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "cron.run" => {
            #[derive(Deserialize)]
            struct Params {
                id: i64,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            state
                .scheduler
                .run_job_now(p.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "cron.enable" => {
            #[derive(Deserialize)]
            struct Params {
                id: i64,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            state
                .scheduler
                .resume_job(p.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "cron.disable" => {
            #[derive(Deserialize)]
            struct Params {
                id: i64,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            state
                .scheduler
                .pause_job(p.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "config.get" => {
            #[derive(Deserialize)]
            struct Params {
                path: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            Ok(serde_json::json!({ "value": cfg.get(&p.path) }))
        }
        "config.set" => {
            #[derive(Deserialize)]
            struct Params {
                path: String,
                value: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            cfg.set(&p.path, &p.value)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "config.unset" => {
            #[derive(Deserialize)]
            struct Params {
                path: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            cfg.unset(&p.path)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({"ok": true}))
        }
        "approvals.list" => {
            let data = approvals::get_approvals(&state.db_path)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::to_value(data).map_err(|e| RpcError::Internal(e.to_string()))?)
        }
        "approvals.approve" => Ok(serde_json::json!({ "ok": true, "status": "approved" })),
        "approvals.deny" => Ok(serde_json::json!({ "ok": true, "status": "denied" })),
        "plugins.list" => {
            let reg = plugins::load_registry().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::to_value(reg).map_err(|e| RpcError::Internal(e.to_string()))?)
        }
        "plugins.enable" => {
            #[derive(Deserialize)]
            struct Params {
                name: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let changed = plugins::set_plugin_enabled(&p.name, true)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": changed }))
        }
        "plugins.disable" => {
            #[derive(Deserialize)]
            struct Params {
                name: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let changed = plugins::set_plugin_enabled(&p.name, false)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": changed }))
        }
        _ => Err(RpcError::MethodNotFound(method.to_string())),
    }
}

async fn run_agent_turn(
    state: &AppState,
    client_id: &str,
    params: Value,
) -> std::result::Result<Value, RpcError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Params {
        session_id: String,
        message: String,
        timeout_seconds: Option<u64>,
    }

    let parsed: Params =
        serde_json::from_value(params).map_err(|e| RpcError::InvalidParams(e.to_string()))?;
    let session_id = parsed.session_id.clone();
    let message = parsed.message.clone();
    if let Some(command) = parse_slash_command(&message) {
        let reply = match command {
            SlashCommand::Status => "session is active".to_string(),
            SlashCommand::Compact => "session compaction requested".to_string(),
            SlashCommand::Reasoning(mode) => format!("reasoning mode: {:?}", mode),
            SlashCommand::Model(model) => {
                format!("model {}", model.unwrap_or_else(|| "unchanged".to_string()))
            }
            SlashCommand::Reset => "session reset requested".to_string(),
            SlashCommand::Help => "/status /compact /reasoning /model /reset /help".to_string(),
        };
        return Ok(
            serde_json::json!({"ok": true, "reply": reply, "sessionId": session_id, "kind":"command"}),
        );
    }
    let timeout = Duration::from_secs(parsed.timeout_seconds.unwrap_or(60));

    let lock = state.run_queue.session_lock(&session_id).await;
    let _guard = lock.lock().await;
    let mut abort_rx = state.run_queue.register_abort(&session_id).await;

    let _ = state.events.send(GatewayEvent {
        method: "agent.partial".to_string(),
        params: serde_json::json!({"sessionId": session_id, "status":"queued"}),
        target_client: Some(client_id.to_string()),
    });

    let session_id_for_run = session_id.clone();
    let message_for_run = message.clone();
    let run_fut = async {
        let _ = state.events.send(GatewayEvent {
            method: "agent.partial".to_string(),
            params: serde_json::json!({"sessionId": session_id_for_run, "status":"running"}),
            target_client: Some(client_id.to_string()),
        });
        tokio::time::sleep(Duration::from_millis(120)).await;
        sessions::upsert_session(
            &state.db_path,
            &session_id_for_run,
            Some("gateway"),
            "active",
            Some(&serde_json::json!({ "lastInput": message_for_run })),
        )
        .await
        .map_err(|e| RpcError::Internal(e.to_string()))?;
        Ok::<String, RpcError>(format!("Auto reply: {}", message_for_run))
    };

    let result = tokio::select! {
        changed = abort_rx.changed() => {
            if changed.is_ok() && *abort_rx.borrow() {
                Err(RpcError::Internal("aborted".to_string()))
            } else {
                Err(RpcError::Internal("abort channel closed".to_string()))
            }
        }
        timed = tokio::time::timeout(timeout, run_fut) => {
            match timed {
                Ok(reply) => reply,
                Err(_) => Err(RpcError::Internal("run timed out".to_string())),
            }
        }
    };

    state.run_queue.clear_abort(&session_id).await;
    match result {
        Ok(reply) => {
            let formatted = format_reply(Platform::Telegram, &reply);
            let _ = state.events.send(GatewayEvent {
                method: "agent.partial".to_string(),
                params: serde_json::json!({"sessionId": session_id, "status":"completed", "text": reply, "chunks": formatted}),
                target_client: Some(client_id.to_string()),
            });
            Ok(serde_json::json!({"ok": true, "reply": reply, "sessionId": session_id}))
        }
        Err(err) => {
            let _ = state.events.send(GatewayEvent {
                method: "agent.partial".to_string(),
                params: serde_json::json!({"sessionId": session_id, "status":"failed", "error": err.to_string()}),
                target_client: Some(client_id.to_string()),
            });
            Err(err)
        }
    }
}

async fn http_methods() -> impl IntoResponse {
    Json(serde_json::json!(SUPPORTED_METHODS))
}

async fn http_call(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<MethodCallRequest>,
) -> impl IntoResponse {
    let method_name = req.method.clone();

    if !is_authorized(&headers) {
        return call_error_response(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "unauthorized",
            method_name.as_str(),
            None,
        );
    }

    match method_name.as_str() {
        "health" => (
            StatusCode::OK,
            Json(serde_json::json!({
              "status": "ok",
              "compatVersion": state.info.compat_version,
              "fingerprint": state.info.fingerprint,
            })),
        ),
        "status" => {
            let sched = state.scheduler.state().await.ok();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                  "compat": {
                    "compatVersion": state.info.compat_version,
                    "fingerprint": state.info.fingerprint,
                  },
                  "scheduler": sched,
                  "openclawStatus": state.providers.openclaw_status_json(),
                })),
            )
        }
        "cron.list" => match state.scheduler.list_jobs().await {
            Ok(jobs) => (StatusCode::OK, Json(serde_json::json!({ "jobs": jobs }))),
            Err(e) => call_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "failed to list cron jobs",
                "cron.list",
                Some(Value::String(format!("{e:#}"))),
            ),
        },
        "cron.add" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                name: String,
                schedule: String,
                kind: String,
                payload: Value,
                #[serde(default)]
                max_attempts: Option<i64>,
                #[serde(default)]
                backoff_seconds: Option<i64>,
                #[serde(default)]
                enabled: Option<bool>,
            }
            let params: Params = match parse_params(req.params, "cron.add") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state
                .scheduler
                .add_job(
                    params.name,
                    params.schedule,
                    params.kind,
                    params.payload,
                    params.max_attempts,
                    params.backoff_seconds,
                )
                .await
            {
                Ok(id) => {
                    if matches!(params.enabled, Some(false)) {
                        if let Err(e) = state.scheduler.pause_job(id).await {
                            return call_error_response(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "internal_error",
                                "job added but failed to disable",
                                "cron.add",
                                Some(Value::String(format!("{e:#}"))),
                            );
                        }
                    }
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({ "ok": true, "id": id })),
                    )
                }
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to add cron job",
                    "cron.add",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "cron.remove" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                id: i64,
            }
            let params: Params = match parse_params(req.params, "cron.remove") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state.scheduler.remove_job(params.id).await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to remove cron job",
                    "cron.remove",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "cron.run" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                id: i64,
            }
            let params: Params = match parse_params(req.params, "cron.run") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state.scheduler.run_job_now(params.id).await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to run cron job",
                    "cron.run",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "cron.pause" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                id: i64,
            }
            let params: Params = match parse_params(req.params, "cron.pause") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state.scheduler.pause_job(params.id).await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to pause cron job",
                    "cron.pause",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "cron.resume" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                id: i64,
            }
            let params: Params = match parse_params(req.params, "cron.resume") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state.scheduler.resume_job(params.id).await {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to resume cron job",
                    "cron.resume",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "cron.status" => match state.scheduler.state().await {
            Ok(scheduler) => (
                StatusCode::OK,
                Json(serde_json::json!({ "scheduler": scheduler })),
            ),
            Err(e) => call_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "failed to read scheduler status",
                "cron.status",
                Some(Value::String(format!("{e:#}"))),
            ),
        },
        "cron.runs" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                #[serde(default)]
                job_id: Option<i64>,
                #[serde(default = "default_runs_limit")]
                limit: usize,
            }
            fn default_runs_limit() -> usize {
                50
            }
            let params: Params = match parse_params(req.params, "cron.runs") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state.scheduler.list_runs(params.job_id, params.limit).await {
                Ok(runs) => (StatusCode::OK, Json(serde_json::json!({ "runs": runs }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to list cron runs",
                    "cron.runs",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "cron.deadLetters" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                #[serde(default = "default_dead_letters_limit")]
                limit: usize,
            }
            fn default_dead_letters_limit() -> usize {
                50
            }
            let params: Params = match parse_params(req.params, "cron.deadLetters") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match state.scheduler.list_dead_letters(params.limit).await {
                Ok(dead_letters) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "deadLetters": dead_letters })),
                ),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to list dead letters",
                    "cron.deadLetters",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "sessions.list" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                #[serde(default = "default_sessions_limit")]
                limit: usize,
            }
            fn default_sessions_limit() -> usize {
                100
            }
            let params: Params = match parse_params(req.params, "sessions.list") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match sessions::list_sessions(&state.db_path, params.limit).await {
                Ok(rows) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "sessions": rows })),
                ),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to list sessions",
                    "sessions.list",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "sessions.preview" | "sessions.show" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                id: String,
            }
            let params: Params = match parse_params(req.params, method_name.as_str()) {
                Ok(v) => v,
                Err(e) => return e,
            };
            match sessions::get_session(&state.db_path, &params.id).await {
                Ok(Some(session)) => (StatusCode::OK, Json(serde_json::json!(session))),
                Ok(None) => call_error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "session not found",
                    method_name.as_str(),
                    None,
                ),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to read session",
                    method_name.as_str(),
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "approvals.get" => match approvals::get_approvals(&state.db_path).await {
            Ok(approvals_state) => (StatusCode::OK, Json(serde_json::json!(approvals_state))),
            Err(e) => call_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "failed to get approvals",
                "approvals.get",
                Some(Value::String(format!("{e:#}"))),
            ),
        },
        "approvals.set" => {
            let entries = match parse_approvals_entries(req.params) {
                Ok(v) => v,
                Err(e) => {
                    return call_error_response(
                        StatusCode::BAD_REQUEST,
                        "invalid_params",
                        "invalid params",
                        "approvals.set",
                        Some(Value::String(e)),
                    );
                }
            };
            match approvals::set_approvals(&state.db_path, entries).await {
                Ok(count) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "ok": true, "count": count })),
                ),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to set approvals",
                    "approvals.set",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "approvals.allowlist.add" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                pattern: String,
                #[serde(default)]
                agent: Option<String>,
            }
            let params: Params = match parse_params(req.params, "approvals.allowlist.add") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match approvals::allowlist_add(&state.db_path, &params.pattern, params.agent.as_deref())
                .await
            {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to add allowlist entry",
                    "approvals.allowlist.add",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "approvals.allowlist.remove" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Params {
                pattern: String,
                #[serde(default)]
                agent: Option<String>,
            }
            let params: Params = match parse_params(req.params, "approvals.allowlist.remove") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match approvals::allowlist_remove(
                &state.db_path,
                &params.pattern,
                params.agent.as_deref(),
            )
            .await
            {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to remove allowlist entry",
                    "approvals.allowlist.remove",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "approvals.allowlist.list" => match approvals::get_approvals(&state.db_path).await {
            Ok(approvals_state) => (
                StatusCode::OK,
                Json(serde_json::json!({ "allowlist": approvals_state.allowlist })),
            ),
            Err(e) => call_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "failed to list allowlist",
                "approvals.allowlist.list",
                Some(Value::String(format!("{e:#}"))),
            ),
        },
        "pairing.list" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                #[serde(default)]
                channel: Option<String>,
                #[serde(default, alias = "account_id")]
                account_id: Option<String>,
                #[serde(default)]
                status: Option<String>,
                #[serde(default = "default_pairing_limit")]
                limit: usize,
            }
            fn default_pairing_limit() -> usize {
                100
            }
            let params: Params = match parse_params(req.params, "pairing.list") {
                Ok(v) => v,
                Err(e) => return e,
            };
            match pairing::list_pairing_requests(
                &state.db_path,
                params.channel.as_deref(),
                params.account_id.as_deref(),
                params.status.as_deref(),
                params.limit,
            )
            .await
            {
                Ok(rows) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "requests": rows })),
                ),
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to list pairing requests",
                    "pairing.list",
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "pairing.approve" | "pairing.reject" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                id: i64,
                #[serde(default)]
                actor: Option<String>,
                #[serde(default)]
                approved_by: Option<String>,
            }
            let params: Params = match parse_params(req.params, method_name.as_str()) {
                Ok(v) => v,
                Err(e) => return e,
            };
            let actor = params.actor.or(params.approved_by);
            let action = if method_name == "pairing.approve" {
                pairing::PairingAction::Approve
            } else {
                pairing::PairingAction::Reject
            };
            match pairing::apply_pairing_action(&state.db_path, params.id, action, actor.as_deref())
                .await
            {
                Ok(pairing::PairingActionOutcome::Updated(request)) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "ok": true, "request": request })),
                ),
                Ok(pairing::PairingActionOutcome::NotFound) => call_error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "pairing request not found",
                    method_name.as_str(),
                    None,
                ),
                Ok(pairing::PairingActionOutcome::InvalidState { current_status }) => {
                    call_error_response(
                        StatusCode::CONFLICT,
                        "invalid_state",
                        format!("pairing request is already {}", current_status),
                        method_name.as_str(),
                        Some(serde_json::json!({ "status": current_status })),
                    )
                }
                Err(e) => call_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "failed to apply pairing action",
                    method_name.as_str(),
                    Some(Value::String(format!("{e:#}"))),
                ),
            }
        }
        "plugins.get" | "plugins.list" => match plugins::load_registry() {
            Ok(reg) => (StatusCode::OK, Json(serde_json::json!(reg))),
            Err(e) => call_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "failed to load plugin registry",
                method_name.as_str(),
                Some(Value::String(format!("{e:#}"))),
            ),
        },
        "chat.send" => {
            let params: ChatRequest = match parse_params(req.params, "chat.send") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let (status, body) = run_chat_flow(state, params).await;
            (status, Json(body))
        }
        _ => call_error_response(
            StatusCode::NOT_FOUND,
            "unknown_method",
            format!("unsupported method: {}", method_name),
            method_name.as_str(),
            Some(serde_json::json!({ "supportedMethods": SUPPORTED_METHODS })),
        ),
    }
}
fn is_authorized(headers: &HeaderMap) -> bool {
    let required = std::env::var("MAGICMERLIN_API_KEY").ok();
    let Some(required) = required.filter(|s| !s.trim().is_empty()) else {
        return true;
    };

    let provided = headers
        .get("x-magicmerlin-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    provided == required
}

const CONTROL_UI_HTML: &str = r#"<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
  <title>MagicMerlin Control UI</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, sans-serif; max-width: 900px; margin: 40px auto; padding: 0 16px; }
    .row { display: flex; gap: 12px; }
    textarea { width: 100%; min-height: 90px; }
    pre { background: #111; color: #eee; padding: 12px; border-radius: 8px; white-space: pre-wrap; }
    button { padding: 10px 14px; }
    .muted { color: #666; font-size: 12px; }
  </style>
</head>
<body>
  <h1>MagicMerlin Control UI</h1>
  <p class=\"muted\">Local-only. Chat calls <code>POST /chat</code>. If <code>MAGICMERLIN_API_KEY</code> is set, include <code>x-magicmerlin-api-key</code> in requests.</p>

  <div class=\"row\">
    <textarea id=\"msg\" placeholder=\"Say something...\"></textarea>
  </div>
  <div class=\"row\" style=\"margin-top: 8px; align-items: center;\">
    <button id=\"send\">Send</button>
    <span class=\"muted\">sessionId: <span id=\"sid\">(new)</span></span>
  </div>

  <h3>Reply</h3>
  <pre id=\"out\">(no reply yet)</pre>

<script>
  let sessionId = null;
  const out = document.getElementById('out');
  const sid = document.getElementById('sid');

  async function send() {
    const message = document.getElementById('msg').value;
    out.textContent = '...';
    const res = await fetch('/chat', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ message, sessionId })
    });
    const json = await res.json();
    if (!res.ok) {
      out.textContent = JSON.stringify(json, null, 2);
      return;
    }
    sessionId = json.sessionId;
    sid.textContent = sessionId;
    out.textContent = json.reply;
  }

  document.getElementById('send').addEventListener('click', send);
</script>
</body>
</html>"#;

async fn http_index() -> Html<&'static str> {
    Html(CONTROL_UI_HTML)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChatRequest {
    message: String,
    session_id: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    peer_id: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatResponse {
    reply: String,
    session_id: String,
    provider: String,
    model: String,
}

async fn http_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let (status, body) = run_chat_flow(state, req).await;
    (status, Json(body))
}

async fn run_chat_flow(state: AppState, req: ChatRequest) -> (StatusCode, Value) {
    let session_id = req
        .session_id
        .or_else(|| {
            if let (Some(channel), Some(peer_id)) = (req.channel.as_deref(), req.peer_id.as_deref())
            {
                Some(pairing::resolve_dm_session_key(
                    pairing::DmScope::from_env(),
                    channel,
                    peer_id,
                    req.account_id.as_deref(),
                ))
            } else {
                None
            }
        })
        .unwrap_or_else(|| format!("chat:{}", uuid::Uuid::new_v4()));

    // Persist session metadata best-effort.
    let _ = sessions::upsert_session(
        &state.db_path,
        &session_id,
        Some("control_ui"),
        "active",
        Some(&serde_json::json!({"provider":"codex-cli"})),
    )
    .await;

    let model =
        std::env::var("MAGICMERLIN_CHAT_MODEL").unwrap_or_else(|_| "gpt-5.3-codex".to_string());
    let timeout_secs: u64 = std::env::var("MAGICMERLIN_CHAT_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let tmp = match tempfile::TempDir::new() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"error": format!("tempdir: {e:#}")}),
            );
        }
    };

    let out_file = tmp.path().join("last.txt");

    let mut cmd = tokio::process::Command::new("codex");
    cmd.arg("exec")
        .arg("--ephemeral")
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(tmp.path())
        .arg("-s")
        .arg("read-only")
        .arg("-m")
        .arg(&model)
        .arg("--output-last-message")
        .arg(&out_file)
        .arg(&req.message);

    let output = match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        cmd.output(),
    )
    .await
    {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"error": format!("codex exec failed: {e:#}")}),
            );
        }
        Err(_) => {
            return (
                StatusCode::GATEWAY_TIMEOUT,
                serde_json::json!({"error": format!("codex exec timed out after {timeout_secs}s")}),
            );
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return (
            StatusCode::BAD_GATEWAY,
            serde_json::json!({
                "error": "codex exec returned non-zero",
                "stderr": stderr,
            }),
        );
    }

    let reply = match std::fs::read_to_string(&out_file) {
        Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().last().unwrap_or("").to_string()
        }
    };

    (
        StatusCode::OK,
        serde_json::to_value(ChatResponse {
            reply,
            session_id,
            provider: "codex-cli".to_string(),
            model,
        })
        .unwrap_or_else(|_| serde_json::json!({"error":"serialize ChatResponse"})),
    )
}

async fn http_cron_list(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    match state.scheduler.list_jobs().await {
        Ok(jobs) => (StatusCode::OK, Json(serde_json::json!({ "jobs": jobs }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

async fn http_cron_run(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"ok": false, "error":"unauthorized"})),
        );
    }

    match state.scheduler.run_job_now(id).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": format!("{e:#}") })),
        ),
    }
}

async fn http_cron_pause(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"ok": false, "error":"unauthorized"})),
        );
    }

    match state.scheduler.pause_job(id).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": format!("{e:#}") })),
        ),
    }
}

async fn http_cron_resume(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"ok": false, "error":"unauthorized"})),
        );
    }

    match state.scheduler.resume_job(id).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": format!("{e:#}") })),
        ),
    }
}

async fn http_dead_letters(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let rows: Result<Vec<DeadLetter>, _> = state.scheduler.list_dead_letters(100).await;
    match rows {
        Ok(dead_letters) => (
            StatusCode::OK,
            Json(serde_json::json!({ "deadLetters": dead_letters })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Sessions HTTP handlers
// ---------------------------------------------------------------------------

async fn http_sessions_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    match sessions::list_sessions(&state.db_path, 100).await {
        Ok(rows) => (
            StatusCode::OK,
            Json(serde_json::json!({ "sessions": rows })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

async fn http_sessions_show(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    match sessions::get_session(&state.db_path, &id).await {
        Ok(Some(session)) => (StatusCode::OK, Json(serde_json::json!(session))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "session not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Approvals HTTP handler
// ---------------------------------------------------------------------------

async fn http_approvals_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    match approvals::get_approvals(&state.db_path).await {
        Ok(approvals_state) => (StatusCode::OK, Json(serde_json::json!(approvals_state))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Pairing HTTP handlers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairingListQuery {
    channel: Option<String>,
    #[serde(alias = "account_id")]
    account_id: Option<String>,
    status: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairingStateQuery {
    channel: Option<String>,
    peer_id: Option<String>,
    #[serde(alias = "account_id")]
    account_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PairingDecisionRequest {
    id: i64,
    #[serde(default)]
    actor: Option<String>,
    #[serde(default)]
    approved_by: Option<String>,
}

async fn http_pairing_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PairingListQuery>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let limit = query.limit.unwrap_or(100);
    match pairing::list_pairing_requests(
        &state.db_path,
        query.channel.as_deref(),
        query.account_id.as_deref(),
        query.status.as_deref(),
        limit,
    )
    .await
    {
        Ok(rows) => (
            StatusCode::OK,
            Json(serde_json::json!({ "requests": rows })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

async fn http_pairing_action(
    state: AppState,
    request: PairingDecisionRequest,
    action: pairing::PairingAction,
) -> (StatusCode, Json<Value>) {
    let actor = request.actor.or(request.approved_by);
    match pairing::apply_pairing_action(&state.db_path, request.id, action, actor.as_deref()).await
    {
        Ok(pairing::PairingActionOutcome::Updated(updated)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "request": updated })),
        ),
        Ok(pairing::PairingActionOutcome::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": "pairing request not found" })),
        ),
        Ok(pairing::PairingActionOutcome::InvalidState { current_status }) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "ok": false,
                "error": "pairing request is not pending",
                "status": current_status,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": format!("{e:#}") })),
        ),
    }
}

async fn http_pairing_approve(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PairingDecisionRequest>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    http_pairing_action(state, req, pairing::PairingAction::Approve).await
}

async fn http_pairing_reject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PairingDecisionRequest>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    http_pairing_action(state, req, pairing::PairingAction::Reject).await
}

async fn http_pairing_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PairingStateQuery>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let limit = query.limit.unwrap_or(100);
    match pairing::list_pairing_state(
        &state.db_path,
        query.channel.as_deref(),
        query.peer_id.as_deref(),
        query.account_id.as_deref(),
        limit,
    )
    .await
    {
        Ok(rows) => (StatusCode::OK, Json(serde_json::json!({ "state": rows }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Plugins HTTP handler
// ---------------------------------------------------------------------------

async fn http_plugins_list(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    match plugins::load_registry() {
        Ok(reg) => (StatusCode::OK, Json(serde_json::json!(reg))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e:#}") })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn build_test_state() -> AppState {
        let state_root =
            std::env::temp_dir().join(format!("magicmerlin-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&state_root).expect("state dir");
        std::env::set_var("OPENCLAW_STATE_DIR", &state_root);
        let providers = SnapshotBackedProviders::load().expect("providers");
        let hashes = providers.hashes().expect("hashes");
        let info = CompatInfo {
            compat_version: COMPAT_VERSION,
            fingerprint: hashes.fingerprint,
            snapshot_hashes: hashes.files,
        };
        let db_path = state_root.join("gateway-test.sqlite");
        let scheduler = Arc::new(Scheduler::new(db_path.clone()).await.expect("scheduler"));
        let cfg = ConfigManager::load(ConfigOptions::default()).expect("config");
        let (events, _) = broadcast::channel(32);
        AppState {
            providers,
            info,
            scheduler,
            db_path,
            config: Arc::new(Mutex::new(cfg)),
            auth: Arc::new(GatewayAuth::default()),
            events,
            run_queue: Arc::new(RunQueue::default()),
            started_at: Instant::now(),
            presence: Arc::new(Mutex::new(SystemPresence::default())),
        }
    }

    #[tokio::test]
    async fn routes_health_method() {
        let state = build_test_state().await;
        let value = dispatch_ws_method(&state, "test-client", "health", Value::Null)
            .await
            .expect("health result");
        assert_eq!(value.get("ok").and_then(Value::as_bool), Some(true));
    }

    #[tokio::test]
    async fn rejects_invalid_auth() {
        let mut state = build_test_state().await;
        state.auth = Arc::new(GatewayAuth {
            token: Some("t123".to_string()),
            password: None,
        });
        let ok = is_ws_authorized(
            &state,
            Some(&JsonRpcAuth {
                token: Some("bad".to_string()),
                password: None,
            }),
            None,
            None,
        );
        assert!(!ok);
    }
}

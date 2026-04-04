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
use magicmerlin_acp::{AcpRuntime, AcpxRequest, AgentHarnessConfig, AgentId};
use magicmerlin_agent::{
    AbortSignal, AgentEngine, AgentEngineConfig, InboundContext, SessionKey, SessionManager,
    ToolExecutionResult, ToolExecutor, ToolSchemaDescriptor,
};
use magicmerlin_agent_tools::{ProcessManager, ToolContext, ToolRegistry, register_default_tools};
use magicmerlin_auto_reply::{format_reply, parse_slash_command, Platform, SlashCommand};
use magicmerlin_compat::{
    providers::{SnapshotBackedProviders, StatusProvider, ToolRegistryProvider},
    COMPAT_VERSION,
};
use magicmerlin_config::{run_security_audit, ConfigManager, ConfigOptions, SecurityAuditContext};
use magicmerlin_gateway::{
    methods::SUPPORTED_METHODS,
    pairing,
    run_queue::{RunQueue, RunQueueConfig, RunStatus},
    ws::{WsServerConfig, WsServerState},
};
use magicmerlin_logging::{init_with as init_logging, LogLevel};
use magicmerlin_providers::{AuthProfiles, ModelRegistry, ProviderRouter};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use tracing::info;

mod approvals;
mod channel_loop;
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
                                    p.version.as_str(),
                                    if p.enabled { "enabled" } else { "disabled" }
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
    event_history: Arc<Mutex<Vec<GatewayEvent>>>,
    run_queue: Arc<RunQueue>,
    ws_state: Arc<WsServerState>,
    started_at: Instant,
    presence: Arc<Mutex<SystemPresence>>,
    acp: Arc<AcpRuntime>,
    agent_engine: Arc<AgentEngine>,
    agent_engines: HashMap<String, Arc<AgentEngine>>,
    tool_registry: Arc<ToolRegistry>,
    session_manager: Arc<SessionManager>,
    process_manager: ProcessManager,
    workspace_dir: PathBuf,
    port: u16,
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

async fn serve_http(
    bind: IpAddr,
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    db_path: PathBuf,
    config: Arc<Mutex<ConfigManager>>,
    auth: Arc<GatewayAuth>,
) -> Result<()> {
    let (state_dir, acp_config, agent_model, loaded_config) = {
        let guard = config.lock().await;
        let cfg = guard.config().clone();
        let model = cfg
            .agents
            .defaults
            .model
            .clone()
            .unwrap_or_else(|| "openai/gpt-4o".to_string());
        (
            guard.state_paths().state_dir.clone(),
            resolve_acp_harness_config(&cfg),
            model,
            cfg,
        )
    };
    let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);
    sessions::migrate_sessions(&db_path).await?;
    approvals::migrate_approvals(&db_path).await?;
    pairing::migrate_pairing(&db_path).await?;
    let (events, _) = broadcast::channel(256);
    let acp = Arc::new(AcpRuntime::new(&state_dir.join("acp"), acp_config)?);
    let ws_state = Arc::new(WsServerState::new(WsServerConfig {
        auth_bearer_token: auth.token.clone(),
    }));

    // --- Agent engine infrastructure ---
    let workspace_dir = state_dir.join("workspace");
    let _ = std::fs::create_dir_all(&workspace_dir);
    let agent_dir = state_dir.join("agents").join("merlin");
    let _ = std::fs::create_dir_all(&agent_dir);
    let sessions_dir = state_dir.join("sessions");
    let _ = std::fs::create_dir_all(&sessions_dir);
    let memory_dir = state_dir.join("memory");
    let _ = std::fs::create_dir_all(&memory_dir);

    let agent_db_path = state_dir.join("agent.sqlite");
    let storage = magicmerlin_storage::Storage::new(&agent_db_path)
        .map_err(|e| anyhow::anyhow!("storage init: {e}"))?;
    let session_manager = Arc::new(SessionManager::new(storage, &sessions_dir, &memory_dir)?);

    let model_registry = ModelRegistry::from_config(&loaded_config)
        .unwrap_or_else(|_| ModelRegistry::default());
    let auth_profiles = AuthProfiles::load_from_state_dir(&state_dir)
        .unwrap_or_default();
    let provider_router = Arc::new(ProviderRouter::with_defaults(
        model_registry,
        auth_profiles,
        None,
    ));

    let engine_config = AgentEngineConfig {
        model: agent_model.clone(),
        workspace_dir: workspace_dir.clone(),
        agent_dir,
        agent_name: "merlin".to_string(),
        channel: "gateway".to_string(),
        timezone: "UTC".to_string(),
        max_turns: 20,
        max_tool_rounds: 10,
        context_window: 120_000,
        token_budget: 100_000,
        compact_threshold_pct: 75,
        ..AgentEngineConfig::default()
    };

    let agent_engine = Arc::new(AgentEngine::new(
        provider_router.clone(),
        (*session_manager).clone(),
        engine_config,
    ));

    // Build named agent engines from config
    let mut agent_engines: HashMap<String, Arc<AgentEngine>> = HashMap::new();
    agent_engines.insert("merlin".to_string(), agent_engine.clone());
    for (name, nacfg) in &loaded_config.agents.named {
        let named_model = nacfg.model.clone().unwrap_or_else(|| agent_model.clone());
        let named_workspace = nacfg.workspace.as_ref().map(PathBuf::from).unwrap_or_else(|| workspace_dir.clone());
        let named_agent_dir = nacfg.agent_dir.as_ref().map(PathBuf::from).unwrap_or_else(|| state_dir.join("agents").join(name));
        let _ = std::fs::create_dir_all(&named_agent_dir);
        let named_config = AgentEngineConfig {
            model: named_model,
            workspace_dir: named_workspace,
            agent_dir: named_agent_dir,
            agent_name: name.clone(),
            channel: "gateway".to_string(),
            timezone: "UTC".to_string(),
            max_turns: 20, max_tool_rounds: 10, context_window: 120_000,
            token_budget: 100_000, compact_threshold_pct: 75,
            ..AgentEngineConfig::default()
        };
        agent_engines.insert(name.clone(), Arc::new(AgentEngine::new(
            provider_router.clone(), (*session_manager).clone(), named_config,
        )));
    }

    let mut tool_registry = ToolRegistry::new();
    register_default_tools(&mut tool_registry);
    let tool_registry = Arc::new(tool_registry);

    let process_manager = ProcessManager::new();

    let state = AppState {
        providers,
        info,
        scheduler,
        db_path,
        config,
        auth,
        events,
        event_history: Arc::new(Mutex::new(Vec::new())),
        run_queue: Arc::new(RunQueue::new(RunQueueConfig {
            max_depth_per_session: 5,
            default_timeout: Duration::from_secs(60),
        })),
        ws_state,
        started_at: Instant::now(),
        presence: Arc::new(Mutex::new(SystemPresence::default())),
        acp,
        agent_engine,
        agent_engines,
        tool_registry,
        session_manager,
        process_manager,
        workspace_dir,
        port,
    };
    let _ws_keepalive = state.ws_state.clone().spawn_keepalive();

    // Start Telegram channel loop (if configured)
    channel_loop::spawn_telegram_loop(state.clone()).await;

    let app = build_router(state);

    let addr = SocketAddr::from((bind, port));
    eprintln!("magicmerlin-gateway listening on http://{addr}");
    let pid_file = service::default_pid_file(&state_dir);
    let _ = service::remove_stale_pid_file(&pid_file);
    let _ = service::write_pid_file(&pid_file);
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        anyhow::anyhow!(
            "failed to bind gateway to {addr}: {err}. If the port is in use, choose another with --serve <port>."
        )
    })?;
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
    let (state_dir, acp_config, daemon_agent_model, daemon_loaded_config) = {
        let guard = config.lock().await;
        let cfg = guard.config().clone();
        let model = cfg
            .agents
            .defaults
            .model
            .clone()
            .unwrap_or_else(|| "openai/gpt-4o".to_string());
        (
            guard.state_paths().state_dir.clone(),
            resolve_acp_harness_config(&cfg),
            model,
            cfg,
        )
    };
    let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);
    sessions::migrate_sessions(&db_path).await?;
    approvals::migrate_approvals(&db_path).await?;
    pairing::migrate_pairing(&db_path).await?;
    let daemon_handle = scheduler.clone().spawn_daemon();

    let (events, _) = broadcast::channel(256);
    let acp = Arc::new(AcpRuntime::new(&state_dir.join("acp"), acp_config)?);
    let ws_state = Arc::new(WsServerState::new(WsServerConfig {
        auth_bearer_token: auth.token.clone(),
    }));

    // --- Agent engine infrastructure (daemon) ---
    let workspace_dir = state_dir.join("workspace");
    let _ = std::fs::create_dir_all(&workspace_dir);
    let agent_dir = state_dir.join("agents").join("merlin");
    let _ = std::fs::create_dir_all(&agent_dir);
    let sessions_dir = state_dir.join("sessions");
    let _ = std::fs::create_dir_all(&sessions_dir);
    let memory_dir = state_dir.join("memory");
    let _ = std::fs::create_dir_all(&memory_dir);

    let agent_db_path = state_dir.join("agent.sqlite");
    let storage = magicmerlin_storage::Storage::new(&agent_db_path)
        .map_err(|e| anyhow::anyhow!("storage init: {e}"))?;
    let session_manager = Arc::new(SessionManager::new(storage, &sessions_dir, &memory_dir)?);

    let model_registry = ModelRegistry::from_config(&daemon_loaded_config)
        .unwrap_or_else(|_| ModelRegistry::default());
    let auth_profiles = AuthProfiles::load_from_state_dir(&state_dir)
        .unwrap_or_default();
    let provider_router = Arc::new(ProviderRouter::with_defaults(
        model_registry,
        auth_profiles,
        None,
    ));

    let engine_config = AgentEngineConfig {
        model: daemon_agent_model.clone(),
        workspace_dir: workspace_dir.clone(),
        agent_dir,
        agent_name: "merlin".to_string(),
        channel: "gateway".to_string(),
        timezone: "UTC".to_string(),
        max_turns: 20,
        max_tool_rounds: 10,
        context_window: 120_000,
        token_budget: 100_000,
        compact_threshold_pct: 75,
        ..AgentEngineConfig::default()
    };

    let agent_engine = Arc::new(AgentEngine::new(
        provider_router.clone(),
        (*session_manager).clone(),
        engine_config,
    ));

    // Build named agent engines from config (daemon)
    let mut agent_engines: HashMap<String, Arc<AgentEngine>> = HashMap::new();
    agent_engines.insert("merlin".to_string(), agent_engine.clone());
    for (name, nacfg) in &daemon_loaded_config.agents.named {
        let named_model = nacfg.model.clone().unwrap_or_else(|| daemon_agent_model.clone());
        let named_workspace = nacfg.workspace.as_ref().map(PathBuf::from).unwrap_or_else(|| workspace_dir.clone());
        let named_agent_dir = nacfg.agent_dir.as_ref().map(PathBuf::from).unwrap_or_else(|| state_dir.join("agents").join(name));
        let _ = std::fs::create_dir_all(&named_agent_dir);
        let named_config = AgentEngineConfig {
            model: named_model,
            workspace_dir: named_workspace,
            agent_dir: named_agent_dir,
            agent_name: name.clone(),
            channel: "gateway".to_string(),
            timezone: "UTC".to_string(),
            max_turns: 20, max_tool_rounds: 10, context_window: 120_000,
            token_budget: 100_000, compact_threshold_pct: 75,
            ..AgentEngineConfig::default()
        };
        agent_engines.insert(name.clone(), Arc::new(AgentEngine::new(
            provider_router.clone(), (*session_manager).clone(), named_config,
        )));
    }

    let mut tool_registry = ToolRegistry::new();
    register_default_tools(&mut tool_registry);
    let tool_registry = Arc::new(tool_registry);

    let process_manager = ProcessManager::new();

    let state = AppState {
        providers,
        info,
        scheduler,
        db_path,
        config,
        auth,
        events,
        event_history: Arc::new(Mutex::new(Vec::new())),
        run_queue: Arc::new(RunQueue::new(RunQueueConfig {
            max_depth_per_session: 5,
            default_timeout: Duration::from_secs(60),
        })),
        ws_state,
        started_at: Instant::now(),
        presence: Arc::new(Mutex::new(SystemPresence::default())),
        acp,
        agent_engine,
        agent_engines,
        tool_registry,
        session_manager,
        process_manager,
        workspace_dir,
        port,
    };
    let _ws_keepalive = state.ws_state.clone().spawn_keepalive();

    // Start Telegram channel loop (if configured)
    channel_loop::spawn_telegram_loop(state.clone()).await;

    let app = build_router(state);

    let addr = SocketAddr::from((bind, port));
    eprintln!("magicmerlin-gateway (daemon) listening on http://{addr}");
    let pid_file = service::default_pid_file(&state_dir);
    let _ = service::remove_stale_pid_file(&pid_file);
    let _ = service::write_pid_file(&pid_file);
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        anyhow::anyhow!(
            "failed to bind gateway to {addr}: {err}. If the port is in use, choose another with --serve <port>."
        )
    })?;

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
        .route("/ui", get(http_index))
        .route("/chat", post(http_chat))
        .route("/methods", get(http_methods))
        .route("/call", post(http_call))
        .route("/ws", post(http_ws))
        .route("/events", get(http_events))
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
        .route("/acp/sessions", get(http_acp_sessions))
        .route("/security/audit", get(http_security_audit))
        .route("/api/v1/message", post(http_api_message))
        .route("/api/v1/sessions", get(http_api_sessions))
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
pub(crate) enum RpcError {
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

fn resolve_acp_harness_config(config: &magicmerlin_config::Config) -> AgentHarnessConfig {
    let mut harness = AgentHarnessConfig::default();
    let values = &config.acp.values;

    if let Some(max) = values
        .get("maxConcurrentSessions")
        .and_then(Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
    {
        harness.max_concurrent_sessions = max.max(1);
    }
    if let Some(ttl) = values.get("ttlSeconds").and_then(Value::as_u64) {
        harness.ttl_seconds = ttl.max(1);
    }
    if let Some(agents) = values.get("allowedAgents").and_then(Value::as_array) {
        let mut parsed = std::collections::BTreeSet::new();
        for agent in agents.iter().filter_map(Value::as_str) {
            parsed.insert(parse_agent_id(agent));
        }
        if !parsed.is_empty() {
            harness.allowed_agents = parsed;
        }
    }
    harness
}

fn parse_agent_id(value: &str) -> AgentId {
    match value {
        "claude-code" => AgentId::ClaudeCode,
        "codex" => AgentId::Codex,
        "opencode" => AgentId::OpenCode,
        "gemini" => AgentId::Gemini,
        "pi" => AgentId::Pi,
        other => AgentId::Custom(other.to_string()),
    }
}

fn build_security_context(config: &ConfigManager, auth: &GatewayAuth) -> SecurityAuditContext {
    let channels = &config.config().channels.values;
    let tools = &config.config().tools.values;
    let gateway = &config.config().gateway;

    let open_dm_policy = channels
        .get("dmPolicy")
        .and_then(Value::as_str)
        .is_some_and(|v| v.eq_ignore_ascii_case("open"));
    let public_bot = channels
        .get("publicBot")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let sandbox_configured = tools
        .get("sandbox")
        .and_then(Value::as_str)
        .is_some_and(|v| !v.trim().is_empty());

    let deny_lists = tools
        .get("denyLists")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let trusted_proxies = gateway
        .extra
        .get("trustedProxies")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    SecurityAuditContext {
        public_bot,
        open_dm_policy,
        sandbox_configured,
        gateway_token: auth.token.clone(),
        gateway_bind: gateway.bind.clone(),
        gateway_port: gateway.port,
        stale_high_token_sessions: 0,
        workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        tool_deny_lists: deny_lists,
        trusted_proxies,
    }
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

    let token_matches = state.auth.token.as_deref().map_or(true, |required| {
        auth.and_then(|a| a.token.as_deref()) == Some(required) || query_token == Some(required)
    });
    let password_matches = state.auth.password.as_deref().map_or(true, |required| {
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
            let mut presence = state.presence.lock().await.clone();
            presence.connected_clients = state.ws_state.connected_clients().await.len();
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
        "system-presence" => {
            let mut presence = state.presence.lock().await.clone();
            presence.connected_clients = state.ws_state.connected_clients().await.len();
            Ok(serde_json::to_value(presence).map_err(|e| RpcError::Internal(e.to_string()))?)
        }
        "system.event" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
                #[serde(default)]
                payload: Value,
                #[serde(default)]
                target_client: Option<String>,
            }
            let parsed: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: parsed.name.clone(),
                    params: parsed.payload,
                    target_client: parsed.target_client,
                },
            )
            .await;
            Ok(serde_json::json!({"ok": true, "event": parsed.name}))
        }
        "system.heartbeat" => {
            let mut presence = state.presence.lock().await;
            presence.last_heartbeat_at = chrono::Utc::now().timestamp();
            Ok(serde_json::json!({"ok": true, "lastHeartbeatAt": presence.last_heartbeat_at}))
        }
        "system.presence" => {
            let presence = state.presence.lock().await.clone();
            Ok(serde_json::to_value(presence).map_err(|e| RpcError::Internal(e.to_string()))?)
        }
        "system.restart" => Ok(serde_json::json!({
            "ok": true,
            "scheduled": true,
            "message": "restart requested (manual supervisor restart expected)"
        })),
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
        "config.validate" => {
            let cfg = state.config.lock().await;
            let ctx = build_security_context(&cfg, &state.auth);
            let report = run_security_audit(&ctx);
            Ok(serde_json::json!({
                "ok": true,
                "issues": report.issues,
            }))
        }
        "security.audit" => {
            let cfg = state.config.lock().await;
            let ctx = build_security_context(&cfg, &state.auth);
            let report = run_security_audit(&ctx);
            Ok(serde_json::to_value(report).map_err(|e| RpcError::Internal(e.to_string()))?)
        }
        "approvals.list" => {
            let data = approvals::get_approvals(&state.db_path)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::to_value(data).map_err(|e| RpcError::Internal(e.to_string()))?)
        }
        "approvals.approve" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                code: String,
                #[serde(default)]
                agent: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            approvals::set_approvals(
                &state.db_path,
                vec![approvals::ApprovalFileEntry {
                    agent: p.agent.clone(),
                    key: p.code.clone(),
                    value: "allow".to_string(),
                }],
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(
                serde_json::json!({ "ok": true, "code": p.code, "decision": "approved", "agent": p.agent }),
            )
        }
        "approvals.deny" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                code: String,
                #[serde(default)]
                agent: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            approvals::set_approvals(
                &state.db_path,
                vec![approvals::ApprovalFileEntry {
                    agent: p.agent.clone(),
                    key: p.code.clone(),
                    value: "deny".to_string(),
                }],
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(
                serde_json::json!({ "ok": true, "code": p.code, "decision": "denied", "agent": p.agent }),
            )
        }
        "approvals.pending" => {
            let data = approvals::get_approvals(&state.db_path)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let pending: Vec<_> = data
                .approvals
                .iter()
                .filter(|a| a.value != "allow" && a.value != "deny")
                .collect();
            Ok(serde_json::json!({ "pending": pending, "count": pending.len() }))
        }
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
        "acp.sessions.list" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: Option<String>,
            }
            let parsed: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let sessions = if let Some(thread_id) = parsed.thread_id {
                state.acp.sessions_for_thread(&thread_id).await
            } else {
                state.acp.list_sessions().await
            };
            Ok(serde_json::json!({ "sessions": sessions }))
        }
        "acp.spawn" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                agent: String,
                command: String,
                #[serde(default)]
                args: Vec<String>,
            }
            let parsed: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let session = state
                .acp
                .dispatch_acpx(AcpxRequest {
                    thread_id: parsed.thread_id,
                    agent: parse_agent_id(&parsed.agent),
                    command: parsed.command,
                    args: parsed.args,
                })
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "session": session }))
        }
        "acp.cleanup" => {
            let removed = state
                .acp
                .cleanup_expired()
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "removed": removed }))
        }

        // ── Pass 7: Memory methods ──────────────────────────────
        "memory.search" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                query: String,
                #[serde(default = "default_mem_limit")]
                limit: usize,
                #[serde(default)]
                agent: Option<String>,
            }
            fn default_mem_limit() -> usize {
                20
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let agent = p.agent.as_deref().unwrap_or("merlin");
            let cfg = state.config.lock().await;
            let mem_dir = cfg
                .state_paths()
                .state_dir
                .join("agents")
                .join(agent)
                .join("memory");
            drop(cfg);
            let matches = search_memory_files(&mem_dir, &p.query, p.limit).await;
            Ok(serde_json::json!({ "matches": matches, "query": p.query, "count": matches.len() }))
        }
        "memory.get" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                key: String,
                #[serde(default)]
                agent: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let agent = p.agent.as_deref().unwrap_or("merlin");
            let cfg = state.config.lock().await;
            let mem_dir = cfg
                .state_paths()
                .state_dir
                .join("agents")
                .join(agent)
                .join("memory");
            drop(cfg);
            let file_path = mem_dir.join(&p.key);
            let content = tokio::fs::read_to_string(&file_path).await.ok();
            Ok(serde_json::json!({ "key": p.key, "value": content }))
        }
        "memory.list" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default = "default_mem_list_limit")]
                limit: usize,
                #[serde(default)]
                agent: Option<String>,
                #[serde(default)]
                prefix: Option<String>,
            }
            fn default_mem_list_limit() -> usize {
                50
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let agent = p.agent.as_deref().unwrap_or("merlin");
            let cfg = state.config.lock().await;
            let mem_dir = cfg
                .state_paths()
                .state_dir
                .join("agents")
                .join(agent)
                .join("memory");
            drop(cfg);
            let files = list_memory_files(&mem_dir, p.prefix.as_deref(), p.limit).await;
            Ok(serde_json::json!({ "files": files, "count": files.len() }))
        }

        // ── Pass 7: Models methods ──────────────────────────────
        "models.list" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                #[serde(default)]
                provider: Option<String>,
            }
            let _p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let model = cfg
                .config()
                .agents
                .defaults
                .model
                .as_deref()
                .unwrap_or("anthropic/claude-sonnet-4-6");
            let provider = cfg
                .config()
                .agents
                .defaults
                .extra
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("anthropic");
            let models_json = serde_json::json!({
                "defaultModel": model,
                "defaultProvider": provider,
                "available": [
                    {"id":"anthropic/claude-opus-4-6","provider":"anthropic","name":"Claude Opus 4.6"},
                    {"id":"anthropic/claude-sonnet-4-6","provider":"anthropic","name":"Claude Sonnet 4.6"},
                    {"id":"openai/gpt-5.2","provider":"openai","name":"GPT-5.2"},
                    {"id":"openai/o3","provider":"openai","name":"o3"},
                    {"id":"google/gemini-2.5-pro","provider":"google","name":"Gemini 2.5 Pro"},
                    {"id":"xai/grok-3","provider":"xai","name":"Grok 3"},
                    {"id":"deepseek/deepseek-r2","provider":"deepseek","name":"DeepSeek R2"},
                    {"id":"mistral/mistral-large","provider":"mistral","name":"Mistral Large"},
                ],
            });
            Ok(models_json)
        }
        "models.set" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                model: String,
                #[serde(default)]
                agent: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            cfg.set("agents.defaults.model", &p.model)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "model": p.model, "agent": p.agent }))
        }
        "models.test" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default)]
                model: Option<String>,
                #[serde(default)]
                provider: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let model = p.model.unwrap_or_else(|| {
                cfg.config()
                    .agents
                    .defaults
                    .model
                    .clone()
                    .unwrap_or_else(|| "anthropic/claude-sonnet-4-6".to_string())
            });
            let provider = p.provider.unwrap_or_else(|| {
                cfg.config()
                    .agents
                    .defaults
                    .extra
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("anthropic")
                    .to_string()
            });
            let api_key_env = match provider.as_str() {
                "anthropic" => "ANTHROPIC_API_KEY",
                "openai" => "OPENAI_API_KEY",
                "google" => "GOOGLE_API_KEY",
                _ => "ANTHROPIC_API_KEY",
            };
            drop(cfg);
            let has_key = std::env::var(api_key_env)
                .ok()
                .is_some_and(|k| !k.is_empty());
            let start = Instant::now();
            let (reachable, latency_ms) = if has_key {
                let test_url = match provider.as_str() {
                    "anthropic" => "https://api.anthropic.com/v1/messages",
                    "openai" => "https://api.openai.com/v1/models",
                    "google" => "https://generativelanguage.googleapis.com/v1/models",
                    _ => "https://api.anthropic.com/v1/messages",
                };
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .map_err(|e| RpcError::Internal(e.to_string()))?;
                let resp = client.head(test_url).send().await;
                let latency = start.elapsed().as_millis() as u64;
                (resp.is_ok(), latency)
            } else {
                (false, 0)
            };
            Ok(serde_json::json!({
                "ok": true,
                "model": model,
                "provider": provider,
                "apiKeyConfigured": has_key,
                "reachable": reachable,
                "latencyMs": latency_ms,
            }))
        }
        "models.status" => {
            let cfg = state.config.lock().await;
            Ok(serde_json::json!({
                "defaultModel": cfg.config().agents.defaults.model,
                "defaultProvider": cfg.config().agents.defaults.extra.get("provider"),
                "configured": true,
            }))
        }

        // ── Channels methods ──────────────────────────────────────
        "channels.list" => {
            let cfg = state.config.lock().await;
            let channels_cfg = &cfg.config().channels.values;
            let platforms = [
                "telegram", "discord", "slack", "whatsapp", "signal", "imessage", "line", "web",
            ];
            let type_map: std::collections::HashMap<&str, &str> = [
                ("telegram", "polling"),
                ("discord", "gateway"),
                ("slack", "events"),
                ("whatsapp", "web"),
                ("signal", "cli"),
                ("imessage", "jxa"),
                ("line", "api"),
                ("web", "webhook"),
            ]
            .into_iter()
            .collect();
            let channels: Vec<Value> = platforms
                .iter()
                .map(|p| {
                    let has_config = channels_cfg.get(*p).is_some_and(|v| !v.is_null());
                    let has_token = channels_cfg
                        .get(&format!("{p}Token"))
                        .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                        || channels_cfg
                            .get(&format!("{p}BotToken"))
                            .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()));
                    let status = if has_config || has_token {
                        "configured"
                    } else {
                        "available"
                    };
                    serde_json::json!({
                        "name": p, "status": status, "type": type_map.get(p).unwrap_or(&"unknown"),
                        "configured": has_config || has_token,
                    })
                })
                .collect();
            Ok(serde_json::json!({ "channels": channels }))
        }
        "channels.status" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default)]
                channel: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let channels_cfg = &cfg.config().channels.values;
            let platforms: Vec<&str> = if let Some(ref ch) = p.channel {
                vec![ch.as_str()]
            } else {
                vec![
                    "telegram", "discord", "slack", "whatsapp", "signal", "imessage", "line", "web",
                ]
            };
            let statuses: Vec<Value> = platforms
                .iter()
                .map(|name| {
                    let configured = channels_cfg.get(*name).is_some_and(|v| !v.is_null())
                        || channels_cfg
                            .get(&format!("{name}Token"))
                            .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                        || channels_cfg
                            .get(&format!("{name}BotToken"))
                            .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()));
                    let status = if configured {
                        "configured"
                    } else {
                        "unconfigured"
                    };
                    serde_json::json!({
                        "name": name, "status": status, "connected": false,
                        "lastMessage": null, "lastError": null,
                    })
                })
                .collect();
            Ok(serde_json::json!({ "channels": statuses }))
        }
        "channels.login" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                channel: String,
                #[serde(default)]
                token: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            // Persist token to config if provided
            if let Some(ref token) = p.token {
                let key = format!("channels.{}Token", p.channel);
                let mut cfg = state.config.lock().await;
                cfg.set(&key, token)
                    .map_err(|e| RpcError::Internal(e.to_string()))?;
                cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            }
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "channels.login".to_string(),
                    params: serde_json::json!({"channel": p.channel}),
                    target_client: Some(client_id.to_string()),
                },
            )
            .await;
            Ok(serde_json::json!({ "ok": true, "channel": p.channel, "status": "login_initiated" }))
        }
        "channels.logout" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                channel: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "channels.logout".to_string(),
                    params: serde_json::json!({"channel": p.channel}),
                    target_client: Some(client_id.to_string()),
                },
            )
            .await;
            Ok(serde_json::json!({ "ok": true, "channel": p.channel, "status": "logged_out" }))
        }
        "channels.restart" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                channel: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "channels.restart".to_string(),
                    params: serde_json::json!({"channel": p.channel}),
                    target_client: Some(client_id.to_string()),
                },
            )
            .await;
            Ok(serde_json::json!({ "ok": true, "channel": p.channel, "status": "restarting" }))
        }
        "channels.send" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                channel: String,
                target: String,
                message: String,
                #[serde(default)]
                reply_to: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let message_id = uuid::Uuid::new_v4().to_string();
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "channels.outbound".to_string(),
                    params: serde_json::json!({
                        "channel": p.channel, "target": p.target,
                        "message": p.message, "messageId": message_id,
                        "replyTo": p.reply_to,
                    }),
                    target_client: None,
                },
            )
            .await;
            Ok(
                serde_json::json!({ "ok": true, "channel": p.channel, "target": p.target, "messageId": message_id }),
            )
        }
        "channels.react" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                channel: String,
                message_id: String,
                emoji: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            emit_gateway_event(state, GatewayEvent {
                method: "channels.react".to_string(),
                params: serde_json::json!({"channel": p.channel, "messageId": p.message_id, "emoji": p.emoji}),
                target_client: None,
            }).await;
            Ok(
                serde_json::json!({ "ok": true, "channel": p.channel, "messageId": p.message_id, "emoji": p.emoji }),
            )
        }
        "channels.delete" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                channel: String,
                message_id: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "channels.delete".to_string(),
                    params: serde_json::json!({"channel": p.channel, "messageId": p.message_id}),
                    target_client: None,
                },
            )
            .await;
            Ok(
                serde_json::json!({ "ok": true, "channel": p.channel, "messageId": p.message_id, "deleted": true }),
            )
        }

        // ── Pass 7: Hooks methods ───────────────────────────────
        "hooks.list" => {
            let cfg = state.config.lock().await;
            let hooks = cfg.get("hooks");
            Ok(serde_json::json!({ "hooks": hooks }))
        }
        "hooks.add" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                url: String,
                #[serde(default)]
                events: Option<Vec<String>>,
                #[serde(default)]
                name: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "url": p.url, "events": p.events, "name": p.name }))
        }
        "hooks.remove" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                url: Option<String>,
                #[serde(default)]
                id: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "url": p.url, "id": p.id }))
        }
        "hooks.test" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                url: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            // Perform a test webhook delivery
            let client = reqwest::Client::new();
            let test_payload = serde_json::json!({
                "event": "test",
                "timestamp": chrono::Utc::now().timestamp(),
                "source": "magicmerlin-gateway",
            });
            let result = client.post(&p.url).json(&test_payload).send().await;
            match result {
                Ok(resp) => Ok(serde_json::json!({
                    "ok": true,
                    "url": p.url,
                    "statusCode": resp.status().as_u16(),
                    "reachable": resp.status().is_success(),
                })),
                Err(e) => Ok(serde_json::json!({
                    "ok": false,
                    "url": p.url,
                    "error": e.to_string(),
                    "reachable": false,
                })),
            }
        }

        // ── Pass 7: Logs methods ────────────────────────────────
        "logs.tail" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default = "default_tail_lines")]
                lines: usize,
                #[serde(default)]
                level: Option<String>,
                #[serde(default)]
                component: Option<String>,
            }
            fn default_tail_lines() -> usize {
                100
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let log_dir = cfg.state_paths().logs_dir.clone();
            drop(cfg);
            let entries = tail_log_file(
                &log_dir,
                p.lines,
                p.level.as_deref(),
                p.component.as_deref(),
            )
            .await;
            Ok(serde_json::json!({ "entries": entries, "count": entries.len() }))
        }
        "logs.query" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default)]
                query: Option<String>,
                #[serde(default)]
                level: Option<String>,
                #[serde(default = "default_query_limit")]
                limit: usize,
            }
            fn default_query_limit() -> usize {
                200
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let log_dir = cfg.state_paths().logs_dir.clone();
            drop(cfg);
            let entries =
                query_log_file(&log_dir, p.query.as_deref(), p.level.as_deref(), p.limit).await;
            Ok(serde_json::json!({ "entries": entries, "count": entries.len() }))
        }

        // ── Pass 7: Run queue methods ───────────────────────────
        "run.list" => {
            let runs = state.run_queue.list_runs().await;
            Ok(serde_json::json!({ "runs": runs }))
        }
        "run.status" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                run_id: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let status = state.run_queue.get_run_status(&p.run_id).await;
            Ok(serde_json::json!({ "runId": p.run_id, "status": status }))
        }

        // ── Pass 7: Agents management methods ───────────────────
        "agents.list" => {
            let cfg = state.config.lock().await;
            let default_model = &cfg.config().agents.defaults.model;
            let mut agents_list = vec![
                serde_json::json!({"name":"merlin","model":default_model,"status":"active","description":"Default agent"}),
            ];
            for (name, nacfg) in &cfg.config().agents.named {
                if name == "merlin" { continue; }
                agents_list.push(serde_json::json!({
                    "name": name,
                    "model": nacfg.model.as_deref().unwrap_or("(default)"),
                    "status": if state.agent_engines.contains_key(name) { "active" } else { "configured" },
                    "description": nacfg.description.as_deref().unwrap_or(""),
                }));
            }
            Ok(serde_json::json!({ "agents": agents_list }))
        }
        "agents.get" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let model = &cfg.config().agents.defaults.model;
            Ok(serde_json::json!({
                "agent": {"name": p.name, "model": model, "status": "active"},
            }))
        }
        "agents.add" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
                #[serde(default)]
                model: Option<String>,
                #[serde(default)]
                description: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            let agent_config = serde_json::json!({
                "model": p.model, "description": p.description,
            });
            let path = format!("agents.{}", p.name);
            cfg.set(
                &path,
                &serde_json::to_string(&agent_config).unwrap_or_default(),
            )
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(
                serde_json::json!({ "ok": true, "agent": p.name, "model": p.model, "description": p.description }),
            )
        }
        "agents.remove" => {
            #[derive(Deserialize)]
            struct Params {
                name: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            let path = format!("agents.{}", p.name);
            let _ = cfg.unset(&path);
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "removed": p.name }))
        }
        "agents.config" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
                #[serde(default)]
                key: Option<String>,
                #[serde(default)]
                value: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            if let (Some(key), Some(value)) = (&p.key, &p.value) {
                let path = format!("agents.{}.{}", p.name, key);
                let mut cfg = state.config.lock().await;
                cfg.set(&path, value)
                    .map_err(|e| RpcError::Internal(e.to_string()))?;
                cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
                Ok(serde_json::json!({ "ok": true, "agent": p.name, "key": key, "value": value }))
            } else {
                let cfg = state.config.lock().await;
                let path = format!("agents.{}", p.name);
                let config_val = cfg.get(&path);
                Ok(serde_json::json!({ "agent": p.name, "config": config_val }))
            }
        }

        // ── Pass 7: Skills methods ──────────────────────────────
        "skills.list" => {
            let reg = plugins::load_registry().map_err(|e| RpcError::Internal(e.to_string()))?;
            let skills: Vec<Value> = reg.plugins.iter()
                .map(|p| serde_json::json!({"name": p.name, "version": p.version, "source": p.source, "enabled": p.enabled}))
                .collect();
            Ok(serde_json::json!({ "skills": skills, "count": skills.len() }))
        }
        "skills.get" => {
            #[derive(Deserialize)]
            struct Params {
                name: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let reg = plugins::load_registry().map_err(|e| RpcError::Internal(e.to_string()))?;
            let skill = reg.plugins.iter()
                .find(|plug| plug.name == p.name)
                .map(|plug| serde_json::json!({"name": plug.name, "version": plug.version, "description": plug.description, "source": plug.source, "enabled": plug.enabled}));
            Ok(serde_json::json!({ "skill": skill }))
        }

        // ── Directory methods ────────────────────────────────────
        "directory.search" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                query: String,
                #[serde(default)]
                channel: Option<String>,
                #[serde(default = "default_dir_limit")]
                limit: usize,
            }
            fn default_dir_limit() -> usize {
                25
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            // Search pairing DB for matching contacts
            let all = pairing::list_pairing_state(
                &state.db_path,
                p.channel.as_deref(),
                None,
                None,
                p.limit,
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            let query_lower = p.query.to_lowercase();
            let results: Vec<&_> = all
                .iter()
                .filter(|entry| {
                    serde_json::to_string(entry)
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&query_lower)
                })
                .take(p.limit)
                .collect();
            Ok(
                serde_json::json!({ "results": results, "query": p.query, "channel": p.channel, "count": results.len() }),
            )
        }
        "directory.get" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                id: String,
                #[serde(default)]
                channel: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            // Look up contact by peer_id in pairing DB
            let entries = pairing::list_pairing_state(
                &state.db_path,
                p.channel.as_deref(),
                Some(&p.id),
                None,
                1,
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            let contact = entries.into_iter().next();
            Ok(serde_json::json!({ "id": p.id, "contact": contact, "channel": p.channel }))
        }

        // ── Nodes methods ────────────────────────────────────────
        "nodes.list" => {
            let cfg = state.config.lock().await;
            let nodes_val = cfg.get("nodes");
            let nodes: Vec<Value> = match nodes_val {
                Some(Value::Array(arr)) => arr,
                Some(Value::Object(map)) => map
                    .into_iter()
                    .map(|(id, mut v)| {
                        if let Some(obj) = v.as_object_mut() {
                            obj.entry("id").or_insert(Value::String(id));
                        }
                        v
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let count = nodes.len();
            Ok(serde_json::json!({ "nodes": nodes, "count": count }))
        }
        "nodes.describe" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                id: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.id).await;
            match url {
                Some(base_url) => {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(10))
                        .build()
                        .map_err(|e| RpcError::Internal(e.to_string()))?;
                    match client.get(format!("{base_url}/api/describe")).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            let body: Value = resp.json().await.unwrap_or(Value::Null);
                            Ok(serde_json::json!({ "id": p.id, "url": base_url, "node": body }))
                        }
                        Ok(resp) => Ok(
                            serde_json::json!({ "id": p.id, "url": base_url, "error": format!("HTTP {}", resp.status()) }),
                        ),
                        Err(e) => Ok(
                            serde_json::json!({ "id": p.id, "url": base_url, "error": e.to_string() }),
                        ),
                    }
                }
                None => Err(RpcError::InvalidParams(format!("node not found: {}", p.id))),
            }
        }
        "nodes.run" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                id: String,
                command: String,
                #[serde(default)]
                args: Vec<String>,
                #[serde(default)]
                cwd: Option<String>,
                #[serde(default)]
                timeout_ms: Option<u64>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.id)
                .await
                .ok_or_else(|| RpcError::InvalidParams(format!("node not found: {}", p.id)))?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    p.timeout_ms.unwrap_or(30_000) / 1000 + 5,
                ))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let payload = serde_json::json!({ "command": p.command, "args": p.args, "cwd": p.cwd, "timeoutMs": p.timeout_ms });
            match client
                .post(format!("{url}/api/run"))
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let body: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(body)
                }
                Err(e) => Err(RpcError::Internal(format!("node request failed: {e}"))),
            }
        }
        "nodes.invoke" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                id: String,
                #[serde(alias = "method", alias = "invokeCommand")]
                invoke_command: String,
                #[serde(default, alias = "invokeParamsJson")]
                params: Value,
                #[serde(default, alias = "invokeTimeoutMs")]
                timeout_ms: Option<u64>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.id)
                .await
                .ok_or_else(|| RpcError::InvalidParams(format!("node not found: {}", p.id)))?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    p.timeout_ms.unwrap_or(30_000) / 1000 + 5,
                ))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let payload = serde_json::json!({ "command": p.invoke_command, "params": p.params, "timeoutMs": p.timeout_ms });
            match client
                .post(format!("{url}/api/invoke"))
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let body: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(body)
                }
                Err(e) => Err(RpcError::Internal(format!("node invoke failed: {e}"))),
            }
        }
        "nodes.notify" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(alias = "id")]
                node: String,
                title: String,
                body: String,
                #[serde(default)]
                priority: Option<String>,
                #[serde(default)]
                sound: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.node)
                .await
                .ok_or_else(|| RpcError::InvalidParams(format!("node not found: {}", p.node)))?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let payload = serde_json::json!({ "title": p.title, "body": p.body, "priority": p.priority, "sound": p.sound });
            match client
                .post(format!("{url}/api/notify"))
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let body: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(serde_json::json!({ "ok": status < 400, "node": p.node, "response": body }))
                }
                Err(e) => Err(RpcError::Internal(format!("node notify failed: {e}"))),
            }
        }
        "nodes.location_get" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(alias = "id")]
                node: String,
                #[serde(default)]
                desired_accuracy: Option<String>,
                #[serde(default)]
                location_timeout_ms: Option<u64>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.node)
                .await
                .ok_or_else(|| RpcError::InvalidParams(format!("node not found: {}", p.node)))?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    p.location_timeout_ms.unwrap_or(15_000) / 1000 + 5,
                ))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let mut req_url = format!("{url}/api/location");
            if let Some(acc) = &p.desired_accuracy {
                req_url.push_str(&format!("?accuracy={acc}"));
            }
            match client.get(&req_url).send().await {
                Ok(resp) => {
                    let body: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(body)
                }
                Err(e) => Err(RpcError::Internal(format!("node location_get failed: {e}"))),
            }
        }
        "nodes.screen_record" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(alias = "id")]
                node: String,
                duration_ms: u64,
                #[serde(default)]
                screen_index: Option<u32>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.node)
                .await
                .ok_or_else(|| RpcError::InvalidParams(format!("node not found: {}", p.node)))?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(p.duration_ms / 1000 + 30))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let payload =
                serde_json::json!({ "durationMs": p.duration_ms, "screenIndex": p.screen_index });
            match client
                .post(format!("{url}/api/screen/record"))
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let body: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(body)
                }
                Err(e) => Err(RpcError::Internal(format!(
                    "node screen_record failed: {e}"
                ))),
            }
        }
        "nodes.camera_snap" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(alias = "id")]
                node: String,
                #[serde(default)]
                facing: Option<String>,
                #[serde(default)]
                max_width: Option<u32>,
                #[serde(default)]
                quality: Option<u32>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let url = resolve_node_url(state, &p.node)
                .await
                .ok_or_else(|| RpcError::InvalidParams(format!("node not found: {}", p.node)))?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let payload = serde_json::json!({ "facing": p.facing, "maxWidth": p.max_width, "quality": p.quality });
            match client
                .post(format!("{url}/api/camera/snap"))
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let body: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(body)
                }
                Err(e) => Err(RpcError::Internal(format!("node camera_snap failed: {e}"))),
            }
        }

        // ── Sandbox methods ──────────────────────────────────────
        "sandbox.list" => {
            let output = tokio::process::Command::new("docker")
                .args([
                    "ps",
                    "--filter",
                    "label=magicmerlin.sandbox=true",
                    "--format",
                    "{{json .}}",
                ])
                .output()
                .await;
            match output {
                Ok(o) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let sandboxes: Vec<Value> = stdout
                        .lines()
                        .filter_map(|line| serde_json::from_str(line).ok())
                        .collect();
                    let count = sandboxes.len();
                    Ok(serde_json::json!({ "sandboxes": sandboxes, "count": count }))
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    Ok(serde_json::json!({ "sandboxes": [], "count": 0, "error": stderr.trim() }))
                }
                Err(_) => {
                    Ok(serde_json::json!({ "sandboxes": [], "count": 0, "dockerAvailable": false }))
                }
            }
        }
        "sandbox.start" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
                #[serde(default)]
                image: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let image = p.image.as_deref().unwrap_or("ubuntu:latest");
            let output = tokio::process::Command::new("docker")
                .args([
                    "run",
                    "-d",
                    "--name",
                    &p.name,
                    "--label",
                    "magicmerlin.sandbox=true",
                    image,
                    "sleep",
                    "infinity",
                ])
                .output()
                .await
                .map_err(|e| RpcError::Internal(format!("docker run failed: {e}")))?;
            if output.status.success() {
                let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(
                    serde_json::json!({ "ok": true, "name": p.name, "containerId": container_id, "image": image, "status": "running" }),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                Err(RpcError::Internal(format!("docker run failed: {stderr}")))
            }
        }
        "sandbox.stop" => {
            #[derive(Deserialize)]
            struct Params {
                name: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let output = tokio::process::Command::new("docker")
                .args(["rm", "-f", &p.name])
                .output()
                .await
                .map_err(|e| RpcError::Internal(format!("docker rm failed: {e}")))?;
            Ok(
                serde_json::json!({ "ok": output.status.success(), "name": p.name, "status": "stopped" }),
            )
        }
        "sandbox.status" => {
            let output = tokio::process::Command::new("docker")
                .args([
                    "ps",
                    "-a",
                    "--filter",
                    "label=magicmerlin.sandbox=true",
                    "--format",
                    "{{json .}}",
                ])
                .output()
                .await;
            match output {
                Ok(o) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let sandboxes: Vec<Value> = stdout
                        .lines()
                        .filter_map(|line| serde_json::from_str(line).ok())
                        .collect();
                    let running = sandboxes
                        .iter()
                        .filter(|s| s.get("State").and_then(Value::as_str) == Some("running"))
                        .count();
                    Ok(serde_json::json!({ "sandboxes": sandboxes, "running": running }))
                }
                _ => Ok(
                    serde_json::json!({ "sandboxes": [], "running": 0, "dockerAvailable": false }),
                ),
            }
        }
        "sandbox.exec" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                name: String,
                command: String,
                #[serde(default)]
                args: Vec<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cmd_args = vec!["exec".to_string(), p.name.clone(), p.command.clone()];
            cmd_args.extend(p.args.clone());
            let output = tokio::process::Command::new("docker")
                .args(&cmd_args)
                .output()
                .await
                .map_err(|e| RpcError::Internal(format!("docker exec failed: {e}")))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(serde_json::json!({
                "ok": output.status.success(),
                "exitCode": output.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "sandbox": p.name,
                "command": p.command,
            }))
        }

        // ── Browser methods ──────────────────────────────────────
        "browser.start" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                #[serde(default)]
                profile: Option<String>,
                #[serde(default)]
                headless: Option<bool>,
            }
            let _p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let chrome_path = cfg
                .get("browser.chromePath")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(find_chrome_binary);
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            // Check if already running
            let check = reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            if let Ok(resp) = check
                .get(format!("http://127.0.0.1:{debug_port}/json/version"))
                .send()
                .await
            {
                if resp.status().is_success() {
                    let info: Value = resp.json().await.unwrap_or(Value::Null);
                    return Ok(
                        serde_json::json!({ "ok": true, "status": "already_running", "debugPort": debug_port, "info": info }),
                    );
                }
            }
            let headless_flag = if _p.headless.unwrap_or(true) {
                "--headless=new"
            } else {
                "--no-first-run"
            };
            let child = tokio::process::Command::new(&chrome_path)
                .args([
                    headless_flag,
                    &format!("--remote-debugging-port={debug_port}"),
                    "--disable-gpu",
                    "--no-sandbox",
                ])
                .spawn();
            match child {
                Ok(c) => Ok(
                    serde_json::json!({ "ok": true, "status": "started", "pid": c.id(), "debugPort": debug_port }),
                ),
                Err(e) => Err(RpcError::Internal(format!(
                    "failed to start Chrome at {chrome_path}: {e}"
                ))),
            }
        }
        "browser.stop" => {
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let _ = client
                .get(format!("http://127.0.0.1:{debug_port}/json/close"))
                .send()
                .await;
            // Also try pkill
            let _ = tokio::process::Command::new("pkill")
                .args(["-f", &format!("remote-debugging-port={debug_port}")])
                .output()
                .await;
            Ok(serde_json::json!({ "ok": true, "status": "stopped" }))
        }
        "browser.status" => {
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            match client
                .get(format!("http://127.0.0.1:{debug_port}/json/list"))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let tabs: Vec<Value> = resp.json().await.unwrap_or_default();
                    Ok(
                        serde_json::json!({ "running": true, "tabs": tabs.len(), "debugPort": debug_port, "tabList": tabs }),
                    )
                }
                _ => {
                    Ok(serde_json::json!({ "running": false, "tabs": 0, "debugPort": debug_port }))
                }
            }
        }
        "browser.tabs" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                #[serde(default)]
                profile: Option<String>,
            }
            let _p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            match client
                .get(format!("http://127.0.0.1:{debug_port}/json/list"))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let tabs: Vec<Value> = resp.json().await.unwrap_or_default();
                    let mapped: Vec<Value> = tabs
                        .iter()
                        .map(|t| {
                            serde_json::json!({
                                "id": t.get("id"), "title": t.get("title"),
                                "url": t.get("url"), "type": t.get("type"),
                            })
                        })
                        .collect();
                    Ok(serde_json::json!({ "tabs": mapped }))
                }
                _ => Err(RpcError::Internal(
                    "browser not running — use browser.start first".to_string(),
                )),
            }
        }
        "browser.open" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                url: String,
                #[serde(default)]
                profile: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let encoded_url = urlencoding::encode(&p.url);
            match client
                .get(format!(
                    "http://127.0.0.1:{debug_port}/json/new?{encoded_url}"
                ))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let tab: Value = resp.json().await.unwrap_or(Value::Null);
                    Ok(
                        serde_json::json!({ "ok": true, "targetId": tab.get("id"), "url": p.url, "tab": tab }),
                    )
                }
                Ok(resp) => Err(RpcError::Internal(format!(
                    "failed to open tab: HTTP {}",
                    resp.status()
                ))),
                Err(e) => Err(RpcError::Internal(format!("browser not reachable: {e}"))),
            }
        }
        "browser.navigate" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                url: String,
                #[serde(default)]
                target_id: Option<String>,
                #[serde(default)]
                tab_id: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let target = p.target_id.or(p.tab_id);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            // Navigate by activating the target URL
            let endpoint = if let Some(ref tid) = target {
                format!("http://127.0.0.1:{debug_port}/json/activate/{tid}")
            } else {
                format!("http://127.0.0.1:{debug_port}/json/list")
            };
            let _ = client.get(&endpoint).send().await;
            Ok(serde_json::json!({ "ok": true, "url": p.url, "targetId": target }))
        }
        "browser.screenshot" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default)]
                target_id: Option<String>,
                #[serde(default)]
                tab_id: Option<String>,
                #[serde(default)]
                full_page: Option<bool>,
                #[serde(default, alias = "type")]
                format: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            // Screenshots require CDP WebSocket — return guidance
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            match client
                .get(format!("http://127.0.0.1:{debug_port}/json/list"))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let tabs: Vec<Value> = resp.json().await.unwrap_or_default();
                    let target = p.target_id.or(p.tab_id);
                    let ws_url = tabs
                        .iter()
                        .find(|t| {
                            target.is_none()
                                || t.get("id").and_then(Value::as_str) == target.as_deref()
                        })
                        .and_then(|t| t.get("webSocketDebuggerUrl").and_then(Value::as_str));
                    Ok(serde_json::json!({
                        "ok": true, "targetId": target, "fullPage": p.full_page,
                        "format": p.format.as_deref().unwrap_or("png"),
                        "webSocketDebuggerUrl": ws_url,
                        "note": "use CDP Page.captureScreenshot via the webSocketDebuggerUrl",
                    }))
                }
                _ => Err(RpcError::Internal("browser not running".to_string())),
            }
        }
        "browser.act" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                #[serde(default)]
                target_id: Option<String>,
                #[serde(alias = "action")]
                request: Value,
                #[serde(default)]
                profile: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            match client
                .get(format!("http://127.0.0.1:{debug_port}/json/list"))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let tabs: Vec<Value> = resp.json().await.unwrap_or_default();
                    let ws_url = tabs
                        .iter()
                        .find(|t| {
                            p.target_id.is_none()
                                || t.get("id").and_then(Value::as_str) == p.target_id.as_deref()
                        })
                        .and_then(|t| t.get("webSocketDebuggerUrl").and_then(Value::as_str));
                    Ok(serde_json::json!({
                        "ok": true, "targetId": p.target_id, "request": p.request,
                        "webSocketDebuggerUrl": ws_url,
                        "note": "dispatch CDP commands via the webSocketDebuggerUrl",
                    }))
                }
                _ => Err(RpcError::Internal("browser not running".to_string())),
            }
        }
        "browser.snapshot" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                #[serde(default)]
                target_id: Option<String>,
                #[serde(default)]
                profile: Option<String>,
                #[serde(default)]
                refs: Option<bool>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let debug_port = cfg
                .get("browser.debugPort")
                .and_then(|v| v.as_u64())
                .unwrap_or(9222);
            drop(cfg);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            match client
                .get(format!("http://127.0.0.1:{debug_port}/json/list"))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let tabs: Vec<Value> = resp.json().await.unwrap_or_default();
                    let tab = tabs
                        .iter()
                        .find(|t| {
                            p.target_id.is_none()
                                || t.get("id").and_then(Value::as_str) == p.target_id.as_deref()
                        })
                        .cloned();
                    Ok(serde_json::json!({
                        "ok": true,
                        "url": tab.as_ref().and_then(|t| t.get("url")),
                        "title": tab.as_ref().and_then(|t| t.get("title")),
                        "targetId": tab.as_ref().and_then(|t| t.get("id")),
                        "webSocketDebuggerUrl": tab.as_ref().and_then(|t| t.get("webSocketDebuggerUrl")),
                        "note": "use CDP Accessibility.getFullAXTree via webSocketDebuggerUrl for a11y snapshot",
                    }))
                }
                _ => Err(RpcError::Internal("browser not running".to_string())),
            }
        }

        // ── Extended session methods ─────────────────────────────
        "sessions.history" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                id: String,
                #[serde(default = "default_history_limit")]
                limit: usize,
                #[serde(default)]
                include_tools: Option<bool>,
            }
            fn default_history_limit() -> usize {
                50
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let session = sessions::get_session(&state.db_path, &p.id)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let cfg = state.config.lock().await;
            let state_dir = cfg.state_paths().state_dir.clone();
            drop(cfg);
            let transcript_path = state_dir.join("sessions").join(format!("{}.jsonl", p.id));
            let history =
                read_transcript_tail(&transcript_path, p.limit, p.include_tools.unwrap_or(true))
                    .await;
            Ok(
                serde_json::json!({ "sessionId": p.id, "session": session, "history": history, "count": history.len() }),
            )
        }
        "sessions.yield" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default)]
                session_id: Option<String>,
                #[serde(default)]
                message: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            if let Some(ref sid) = p.session_id {
                sessions::upsert_session(
                    &state.db_path, sid, None, "yielded",
                    Some(&serde_json::json!({"yieldMessage": p.message, "yieldedAt": chrono::Utc::now().timestamp()})),
                ).await.map_err(|e| RpcError::Internal(e.to_string()))?;
            }
            Ok(serde_json::json!({ "ok": true, "sessionId": p.session_id, "status": "yielded" }))
        }
        "sessions.export" => {
            let all = sessions::list_sessions(&state.db_path, 500)
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "sessions": all, "exportedAt": chrono::Utc::now().timestamp() }))
        }

        // ── Pass 7: Extended config methods ─────────────────────
        "config.list" => {
            let cfg = state.config.lock().await;
            let raw = cfg.raw_json();
            Ok(serde_json::json!({ "config": raw }))
        }
        "config.export" => {
            let cfg = state.config.lock().await;
            let raw = cfg.raw_json();
            Ok(serde_json::json!({ "config": raw, "exportedAt": chrono::Utc::now().timestamp() }))
        }
        "config.import" => {
            #[derive(Deserialize)]
            struct Params {
                config: Value,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            cfg.import_json(p.config)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true }))
        }

        // ── Pass 7: Extended system methods ─────────────────────
        "system.info" => {
            let presence = state.presence.lock().await.clone();
            Ok(serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": state.started_at.elapsed().as_secs(),
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "pid": std::process::id(),
                "presence": presence,
            }))
        }
        "system.env" => Ok(serde_json::json!({
            "MAGICMERLIN_STATE_DIR": std::env::var("MAGICMERLIN_STATE_DIR").ok(),
            "MAGICMERLIN_API_KEY": std::env::var("MAGICMERLIN_API_KEY").ok().map(|_| "***"),
            "MAGICMERLIN_DB_PATH": std::env::var("MAGICMERLIN_DB_PATH").ok(),
            "MAGICMERLIN_CONFIG_PATH": std::env::var("MAGICMERLIN_CONFIG_PATH").ok(),
        })),

        // ── Plugins install ──────────────────────────────────────
        "plugins.install" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                source: String,
                #[serde(default)]
                name: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            let plugins_dir = cfg.state_paths().state_dir.join("plugins");
            drop(cfg);
            let _ = tokio::fs::create_dir_all(&plugins_dir).await;
            let plugin_name = p.name.unwrap_or_else(|| {
                p.source
                    .rsplit('/')
                    .next()
                    .unwrap_or("plugin")
                    .trim_end_matches(".git")
                    .to_string()
            });
            let dest = plugins_dir.join(&plugin_name);
            let output = if p.source.starts_with("http") || p.source.ends_with(".git") {
                tokio::process::Command::new("git")
                    .args([
                        "clone",
                        "--depth",
                        "1",
                        &p.source,
                        dest.to_string_lossy().as_ref(),
                    ])
                    .output()
                    .await
            } else {
                // Local path — symlink
                #[cfg(unix)]
                {
                    tokio::process::Command::new("ln")
                        .args(["-sf", &p.source, dest.to_string_lossy().as_ref()])
                        .output()
                        .await
                }
                #[cfg(not(unix))]
                {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "symlink not supported",
                    ))
                }
            };
            match output {
                Ok(o) if o.status.success() => Ok(
                    serde_json::json!({ "ok": true, "source": p.source, "name": plugin_name, "path": dest.to_string_lossy(), "status": "installed" }),
                ),
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    Err(RpcError::Internal(format!("install failed: {stderr}")))
                }
                Err(e) => Err(RpcError::Internal(format!("install failed: {e}"))),
            }
        }

        // ── Subagents methods ───────────────────────────────────
        "subagents.list" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[allow(dead_code)]
            struct Params {
                #[serde(default)]
                recent_minutes: Option<u64>,
            }
            let _p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let acp_sessions = state.acp.list_sessions().await;
            Ok(serde_json::json!({ "subagents": acp_sessions, "count": acp_sessions.len() }))
        }
        "subagents.steer" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                target: String,
                message: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            sessions::upsert_session(
                &state.db_path, &p.target, None, "active",
                Some(&serde_json::json!({"steerMessage": p.message, "steeredAt": chrono::Utc::now().timestamp()})),
            ).await.map_err(|e| RpcError::Internal(e.to_string()))?;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "subagents.steer".to_string(),
                    params: serde_json::json!({"target": p.target, "message": p.message}),
                    target_client: None,
                },
            )
            .await;
            Ok(serde_json::json!({ "ok": true, "target": p.target }))
        }
        "subagents.kill" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                target: String,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            sessions::upsert_session(
                &state.db_path,
                &p.target,
                None,
                "killed",
                Some(&serde_json::json!({"killedAt": chrono::Utc::now().timestamp()})),
            )
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "subagents.kill".to_string(),
                    params: serde_json::json!({"target": p.target}),
                    target_client: None,
                },
            )
            .await;
            Ok(serde_json::json!({ "ok": true, "target": p.target, "status": "killed" }))
        }

        // ── Gateway control aliases ─────────────────────────────
        "gateway.status" => {
            let scheduler_state = state
                .scheduler
                .state()
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            let mut presence = state.presence.lock().await.clone();
            presence.connected_clients = state.ws_state.connected_clients().await.len();
            let config = state.config.lock().await;
            let model = config.config().agents.defaults.model.clone();
            drop(config);
            let session_count = sessions::list_sessions(&state.db_path, 1000)
                .await
                .map(|s| s.len())
                .unwrap_or(0);
            Ok(serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "uptime": state.started_at.elapsed().as_secs(),
                "pid": std::process::id(),
                "model": model,
                "agents": ["merlin"],
                "sessions": session_count,
                "scheduler": scheduler_state,
                "presence": presence,
            }))
        }
        "gateway.restart" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(default)]
                reason: Option<String>,
                #[serde(default)]
                delay_ms: Option<u64>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let delay = p.delay_ms.unwrap_or(500);
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "gateway.restart".to_string(),
                    params: serde_json::json!({"reason": p.reason, "delayMs": delay}),
                    target_client: None,
                },
            )
            .await;
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(delay)).await;
                std::process::exit(0);
            });
            Ok(serde_json::json!({ "ok": true, "restarting_in_ms": delay, "reason": p.reason }))
        }
        "gateway.config.get" => {
            #[derive(Deserialize)]
            struct Params {
                #[serde(default)]
                path: Option<String>,
            }
            let p: Params = serde_json::from_value(if params.is_null() {
                serde_json::json!({})
            } else {
                params
            })
            .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let cfg = state.config.lock().await;
            match p.path {
                Some(path) => Ok(serde_json::json!({ "value": cfg.get(&path), "path": path })),
                None => Ok(serde_json::json!({ "config": cfg.raw_json() })),
            }
        }
        "gateway.config.patch" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                raw: Value,
                #[serde(default)]
                note: Option<String>,
            }
            let p: Params = serde_json::from_value(params)
                .map_err(|e| RpcError::InvalidParams(e.to_string()))?;
            let mut cfg = state.config.lock().await;
            cfg.import_json(p.raw)
                .map_err(|e| RpcError::Internal(e.to_string()))?;
            cfg.save().map_err(|e| RpcError::Internal(e.to_string()))?;
            Ok(serde_json::json!({ "ok": true, "note": p.note }))
        }

        _ => Err(RpcError::MethodNotFound(method.to_string())),
    }
}

/// Bridge that adapts `ToolRegistry` + `ToolContext` into the `ToolExecutor` trait
/// expected by `AgentEngine::run_turn_with_options`.
struct RegistryToolExecutor {
    registry: Arc<ToolRegistry>,
    ctx: ToolContext,
}

#[async_trait::async_trait]
impl ToolExecutor for RegistryToolExecutor {
    async fn execute_tool(
        &self,
        tool_call: &magicmerlin_providers::types::ToolCall,
    ) -> std::result::Result<ToolExecutionResult, magicmerlin_agent::AgentError> {
        match self.registry.execute(&tool_call.name, tool_call.arguments.clone(), &self.ctx).await {
            Ok(result) => {
                let content = serde_json::to_string(&result.value).unwrap_or_default();
                if result.ok {
                    Ok(ToolExecutionResult::ok(tool_call.id.clone(), content))
                } else {
                    Ok(ToolExecutionResult::err(tool_call.id.clone(), content))
                }
            }
            Err(e) => Ok(ToolExecutionResult::err(
                tool_call.id.clone(),
                e.to_string(),
            )),
        }
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
        agent: Option<String>,
    }

    let parsed: Params =
        serde_json::from_value(params).map_err(|e| RpcError::InvalidParams(e.to_string()))?;
    let session_id = parsed.session_id.clone();
    let message = parsed.message.clone();
    let agent_name = parsed.agent.clone().unwrap_or_else(|| "merlin".to_string());

    // Handle slash commands locally (unchanged)
    if let Some(command) = parse_slash_command(&message) {
        let reply = match command {
            SlashCommand::Status => "session is active".to_string(),
            SlashCommand::Compact => {
                let sm = state.session_manager.clone();
                let key = magicmerlin_agent::SessionKey(session_id.clone());
                match sm.load_or_create(key, &agent_name) {
                    Ok(mut sess) => {
                        // Force compact (threshold = 0)
                        match sm.compact_now(&mut sess) {
                            Ok(c) => format!(
                                "Compacted: {} → {} messages, {} → {} tokens, {} memories extracted",
                                c.messages_before, c.messages_after,
                                c.tokens_before, c.tokens_after,
                                c.memory_candidates_extracted,
                            ),
                            Err(e) => format!("compaction failed: {e}"),
                        }
                    }
                    Err(e) => format!("failed to load session: {e}"),
                }
            }
            SlashCommand::Reasoning { on } => format!("reasoning mode: {:?}", on),
            SlashCommand::Model { name } => {
                format!("model {}", name.unwrap_or_else(|| "unchanged".to_string()))
            }
            SlashCommand::Reset => "session reset requested".to_string(),
            SlashCommand::Help { .. } => {
                "/status /compact /reasoning /model /reset /help".to_string()
            }
            _ => format!("command received: {:?}", command),
        };
        return Ok(
            serde_json::json!({"ok": true, "reply": reply, "sessionId": session_id, "kind":"command"}),
        );
    }

    // Queue management (unchanged)
    let timeout = Duration::from_secs(parsed.timeout_seconds.unwrap_or(60));
    let queue_timeout = Duration::from_secs(30);
    let run_id = format!("run:{}:{}", session_id, uuid::Uuid::new_v4());
    state
        .run_queue
        .enqueue(&session_id, &run_id, Some(timeout))
        .await
        .map_err(RpcError::InvalidParams)?;
    state
        .run_queue
        .wait_turn(&session_id, &run_id, queue_timeout)
        .await
        .map_err(RpcError::Internal)?;

    let lock = state.run_queue.session_lock(&session_id).await;
    let _guard = lock.lock().await;
    let mut abort_rx = state.run_queue.register_abort(&session_id).await;

    emit_gateway_event(
        state,
        GatewayEvent {
            method: "agent.partial".to_string(),
            params: serde_json::json!({"sessionId": session_id, "status":"queued"}),
            target_client: Some(client_id.to_string()),
        },
    )
    .await;

    // --- Real agent loop ---
    let session_id_for_run = session_id.clone();
    let message_for_run = message.clone();
    let engine = state
        .agent_engines
        .get(&agent_name)
        .cloned()
        .unwrap_or_else(|| state.agent_engine.clone());
    let agent_name_for_ctx = agent_name.clone();
    let sm = state.session_manager.clone();
    let registry = state.tool_registry.clone();

    // Build ToolContext for this turn
    let tool_ctx = {
        let guard = state.config.lock().await;
        let cfg = guard.config().clone();
        let sp = guard.state_paths().clone();
        ToolContext {
            agent_name: agent_name_for_ctx,
            workspace_dir: state.workspace_dir.clone(),
            state_paths: sp,
            config: cfg,
            delivery: None,
            process_manager: state.process_manager.clone(),
            node_configs: vec![],
            browser_manager: None,
            canvas_server: None,
            tts_client: None,
            understanding_client: None,
        }
    };

    // Build tool schemas from registry
    let tool_schemas: Vec<ToolSchemaDescriptor> = registry
        .schemas()
        .into_iter()
        .filter_map(|s| {
            let name = s.get("name")?.as_str()?.to_string();
            let description = s.get("description")?.as_str()?.to_string();
            let parameters = s.get("parameters").cloned().unwrap_or(serde_json::json!({}));
            Some(ToolSchemaDescriptor {
                name,
                description,
                parameters,
            })
        })
        .collect();

    let tool_executor = RegistryToolExecutor {
        registry,
        ctx: tool_ctx,
    };

    // Build abort signal wired to run_queue abort channel
    let abort_signal = AbortSignal::new();

    let run_fut = async {
        emit_gateway_event(
            state,
            GatewayEvent {
                method: "agent.partial".to_string(),
                params: serde_json::json!({"sessionId": session_id_for_run, "status":"running"}),
                target_client: Some(client_id.to_string()),
            },
        )
        .await;

        // Upsert gateway session record
        sessions::upsert_session(
            &state.db_path,
            &session_id_for_run,
            Some("gateway"),
            "active",
            Some(&serde_json::json!({ "lastInput": message_for_run })),
        )
        .await
        .map_err(|e| RpcError::Internal(e.to_string()))?;

        // Load or create agent session
        let session_key = SessionKey::agent_main(&agent_name);
        let mut session = sm
            .load_or_create(session_key, &agent_name)
            .map_err(|e| RpcError::Internal(format!("session load: {e}")))?;

        let inbound = InboundContext::default();

        // Run the real agent turn
        let reply = engine
            .run_turn_with_options(
                &mut session,
                &message_for_run,
                &tool_executor,
                &inbound,
                &tool_schemas,
                Some(&abort_signal),
            )
            .await
            .map_err(|e| RpcError::Internal(format!("agent turn: {e}")))?;

        Ok::<String, RpcError>(reply.text)
    };

    // Race against abort signal and timeout
    let abort_signal_cancel = abort_signal.clone();
    let result = tokio::select! {
        changed = abort_rx.changed() => {
            if changed.is_ok() && *abort_rx.borrow() {
                abort_signal_cancel.cancel();
                Err(RpcError::Internal("aborted".to_string()))
            } else {
                Err(RpcError::Internal("abort channel closed".to_string()))
            }
        }
        timed = tokio::time::timeout(timeout, run_fut) => {
            match timed {
                Ok(reply) => reply,
                Err(_) => {
                    abort_signal_cancel.cancel();
                    Err(RpcError::Internal("run timed out".to_string()))
                }
            }
        }
    };

    state.run_queue.clear_abort(&session_id).await;
    match result {
        Ok(reply) => {
            state
                .run_queue
                .complete(&session_id, &run_id, RunStatus::Completed, None)
                .await;
            let formatted = format_reply(Platform::Telegram, &reply);
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "agent.partial".to_string(),
                    params: serde_json::json!({"sessionId": session_id, "status":"completed", "text": reply, "chunks": formatted}),
                    target_client: Some(client_id.to_string()),
                },
            )
            .await;
            Ok(serde_json::json!({"ok": true, "reply": reply, "sessionId": session_id}))
        }
        Err(err) => {
            let status = if err.to_string().contains("timed out") {
                RunStatus::TimedOut
            } else if err.to_string().contains("aborted") {
                RunStatus::Aborted
            } else {
                RunStatus::Failed
            };
            state
                .run_queue
                .complete(&session_id, &run_id, status, Some(err.to_string()))
                .await;
            emit_gateway_event(
                state,
                GatewayEvent {
                    method: "agent.partial".to_string(),
                    params: serde_json::json!({"sessionId": session_id, "status":"failed", "error": err.to_string()}),
                    target_client: Some(client_id.to_string()),
                },
            )
            .await;
            Err(err)
        }
    }
}

const EVENT_HISTORY_LIMIT: usize = 500;

async fn emit_gateway_event(state: &AppState, event: GatewayEvent) {
    let _ = state.events.send(event.clone());
    let mut history = state.event_history.lock().await;
    history.push(event);
    if history.len() > EVENT_HISTORY_LIMIT {
        let overflow = history.len() - EVENT_HISTORY_LIMIT;
        history.drain(0..overflow);
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

        // ── Delegate all other methods to WS dispatch ──────────
        "memory.search"
        | "memory.get"
        | "memory.list"
        | "models.list"
        | "models.set"
        | "models.test"
        | "models.status"
        | "channels.list"
        | "channels.status"
        | "channels.login"
        | "channels.logout"
        | "channels.restart"
        | "channels.send"
        | "channels.react"
        | "channels.delete"
        | "hooks.list"
        | "hooks.add"
        | "hooks.remove"
        | "hooks.test"
        | "logs.tail"
        | "logs.query"
        | "run.list"
        | "run.status"
        | "agents.list"
        | "agents.get"
        | "agents.add"
        | "agents.remove"
        | "agents.config"
        | "skills.list"
        | "skills.get"
        | "directory.search"
        | "directory.get"
        | "nodes.list"
        | "nodes.describe"
        | "nodes.run"
        | "nodes.invoke"
        | "nodes.notify"
        | "nodes.location_get"
        | "nodes.screen_record"
        | "nodes.camera_snap"
        | "sandbox.list"
        | "sandbox.start"
        | "sandbox.stop"
        | "sandbox.status"
        | "sandbox.exec"
        | "browser.start"
        | "browser.stop"
        | "browser.status"
        | "browser.navigate"
        | "browser.screenshot"
        | "browser.act"
        | "browser.snapshot"
        | "browser.tabs"
        | "browser.open"
        | "sessions.history"
        | "sessions.export"
        | "sessions.yield"
        | "config.list"
        | "config.export"
        | "config.import"
        | "system.info"
        | "system.env"
        | "system.restart"
        | "plugins.install"
        | "subagents.list"
        | "subagents.steer"
        | "subagents.kill"
        | "gateway.status"
        | "gateway.restart"
        | "gateway.config.get"
        | "gateway.config.patch"
        | "approvals.pending" => {
            match dispatch_ws_method(&state, "http-call", method_name.as_str(), req.params).await {
                Ok(result) => (StatusCode::OK, Json(result)),
                Err(rpc_err) => {
                    let (code_str, status_code) = match &rpc_err {
                        RpcError::InvalidParams(_) => ("invalid_params", StatusCode::BAD_REQUEST),
                        RpcError::MethodNotFound(_) => ("method_not_found", StatusCode::NOT_FOUND),
                        RpcError::Internal(_) => {
                            ("internal_error", StatusCode::INTERNAL_SERVER_ERROR)
                        }
                        RpcError::Unauthorized => ("unauthorized", StatusCode::UNAUTHORIZED),
                    };
                    call_error_response(
                        status_code,
                        code_str,
                        rpc_err.to_string(),
                        method_name.as_str(),
                        None,
                    )
                }
            }
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

    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| {
            raw.strip_prefix("Bearer ")
                .or_else(|| raw.strip_prefix("bearer "))
        })
        .map(str::trim)
        .unwrap_or("");

    let provided = headers
        .get("x-magicmerlin-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    provided == required || bearer == required
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApiMessageRequest {
    session_id: String,
    message: String,
    #[serde(default)]
    timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiSessionsQuery {
    #[serde(default = "default_api_sessions_limit")]
    limit: usize,
}

fn default_api_sessions_limit() -> usize {
    100
}

async fn http_api_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ApiMessageRequest>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "unauthorized"})),
        );
    }

    let params = serde_json::json!({
        "sessionId": req.session_id,
        "message": req.message,
        "timeoutSeconds": req.timeout_seconds,
    });

    match run_agent_turn(&state, "http-api", params).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": err.to_string(), "code": err.code()})),
        ),
    }
}

async fn http_api_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ApiSessionsQuery>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "unauthorized"})),
        );
    }

    match sessions::list_sessions(&state.db_path, query.limit).await {
        Ok(rows) => (
            StatusCode::OK,
            Json(serde_json::json!({ "sessions": rows })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("failed to list sessions: {err:#}")})),
        ),
    }
}

const CONTROL_UI_HTML: &str = include_str!("../static/index.html");

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventsQuery {
    since: Option<usize>,
    limit: Option<usize>,
}

async fn http_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let since = query.since.unwrap_or(0);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let history = state.event_history.lock().await;
    let start = since.min(history.len());
    let items: Vec<GatewayEvent> = history.iter().skip(start).take(limit).cloned().collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "events": items,
            "nextCursor": start + items.len(),
            "total": history.len(),
        })),
    )
}

async fn http_acp_sessions(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let sessions = state.acp.list_sessions().await;
    (
        StatusCode::OK,
        Json(serde_json::json!({ "sessions": sessions })),
    )
}

async fn http_security_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"unauthorized"})),
        );
    }

    let cfg = state.config.lock().await;
    let ctx = build_security_context(&cfg, &state.auth);
    let report = run_security_audit(&ctx);
    (StatusCode::OK, Json(serde_json::json!(report)))
}

// ── Sprint 6: Helper functions ──────────────────────────────────────

/// Resolve a node's base URL from config by its ID/name.
async fn resolve_node_url(state: &AppState, node_id: &str) -> Option<String> {
    let cfg = state.config.lock().await;
    let nodes_val = cfg.get("nodes");
    match nodes_val {
        Some(Value::Array(arr)) => {
            for node in &arr {
                if node.get("id").and_then(Value::as_str) == Some(node_id)
                    || node.get("name").and_then(Value::as_str) == Some(node_id)
                {
                    return node
                        .get("url")
                        .and_then(Value::as_str)
                        .map(|s| s.trim_end_matches('/').to_string());
                }
            }
            None
        }
        Some(Value::Object(map)) => {
            if let Some(node) = map.get(node_id) {
                return node
                    .get("url")
                    .and_then(Value::as_str)
                    .map(|s| s.trim_end_matches('/').to_string());
            }
            None
        }
        _ => None,
    }
}

/// Find a Chrome/Chromium binary on the system.
fn find_chrome_binary() -> String {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return c.to_string();
        }
    }
    // Fall back to PATH lookup
    "google-chrome".to_string()
}

/// Read the tail of a JSONL transcript file.
async fn read_transcript_tail(
    path: &std::path::Path,
    limit: usize,
    include_tools: bool,
) -> Vec<Value> {
    let Ok(content) = tokio::fs::read_to_string(path).await else {
        return Vec::new();
    };
    let all_lines: Vec<&str> = content.lines().collect();
    let start = all_lines.len().saturating_sub(limit * 2); // read extra in case we filter
    let mut results = Vec::new();
    for line in &all_lines[start..] {
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            if !include_tools {
                let role = entry.get("role").and_then(Value::as_str).unwrap_or("");
                if role == "tool" || role == "tool_result" {
                    continue;
                }
            }
            results.push(entry);
            if results.len() >= limit {
                break;
            }
        }
    }
    results
}

// ── Pass 7: Helper functions for new gateway methods ────────────────

/// Search memory files for a query string (case-insensitive substring match).
async fn search_memory_files(mem_dir: &std::path::Path, query: &str, limit: usize) -> Vec<Value> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(mem_dir).await else {
        return results;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if results.len() >= limit {
            break;
        }
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|e| e == "md" || e == "txt" || e == "json")
        {
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if content.to_lowercase().contains(&query_lower) {
                    let filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let snippet = content
                        .lines()
                        .find(|line| line.to_lowercase().contains(&query_lower))
                        .unwrap_or("")
                        .to_string();
                    results.push(serde_json::json!({
                        "file": filename,
                        "snippet": snippet,
                        "path": path.to_string_lossy(),
                    }));
                }
            }
        }
    }
    results
}

/// List memory files with optional prefix filter.
async fn list_memory_files(
    mem_dir: &std::path::Path,
    prefix: Option<&str>,
    limit: usize,
) -> Vec<Value> {
    let mut files = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(mem_dir).await else {
        return files;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if files.len() >= limit {
            break;
        }
        let path = entry.path();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if let Some(pfx) = prefix {
            if !filename.starts_with(pfx) {
                continue;
            }
        }
        if let Ok(meta) = entry.metadata().await {
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            files.push(serde_json::json!({
                "name": filename,
                "size": meta.len(),
                "modifiedAt": modified,
            }));
        }
    }
    files
}

/// Tail the most recent log file, returning the last N lines.
async fn tail_log_file(
    log_dir: &std::path::Path,
    lines: usize,
    level: Option<&str>,
    component: Option<&str>,
) -> Vec<Value> {
    // Find the most recent .log file
    let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    if let Ok(mut entries) = tokio::fs::read_dir(log_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "log" || e == "jsonl") {
                if let Ok(meta) = entry.metadata().await {
                    let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                    if latest.as_ref().map_or(true, |(_, best)| modified > *best) {
                        latest = Some((path, modified));
                    }
                }
            }
        }
    }
    let Some((log_path, _)) = latest else {
        return Vec::new();
    };
    let Ok(content) = tokio::fs::read_to_string(&log_path).await else {
        return Vec::new();
    };
    let all_lines: Vec<&str> = content.lines().collect();
    let start = all_lines.len().saturating_sub(lines);
    all_lines[start..]
        .iter()
        .filter(|line| {
            if let Some(lvl) = level {
                if !line.to_lowercase().contains(&lvl.to_lowercase()) {
                    return false;
                }
            }
            if let Some(comp) = component {
                if !line.contains(comp) {
                    return false;
                }
            }
            true
        })
        .map(|line| serde_json::json!({"line": line}))
        .collect()
}

/// Query log files with a text search and optional level filter.
async fn query_log_file(
    log_dir: &std::path::Path,
    query: Option<&str>,
    level: Option<&str>,
    limit: usize,
) -> Vec<Value> {
    let mut results = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(log_dir).await else {
        return results;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if results.len() >= limit {
            break;
        }
        let path = entry.path();
        if !path.extension().is_some_and(|e| e == "log" || e == "jsonl") {
            continue;
        }
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            for line in content.lines() {
                if results.len() >= limit {
                    break;
                }
                let matches_query =
                    query.map_or(true, |q| line.to_lowercase().contains(&q.to_lowercase()));
                let matches_level =
                    level.map_or(true, |l| line.to_lowercase().contains(&l.to_lowercase()));
                if matches_query && matches_level {
                    results.push(serde_json::json!({
                        "line": line,
                        "file": path.file_name().unwrap_or_default().to_string_lossy(),
                    }));
                }
            }
        }
    }
    results
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
        let acp = Arc::new(
            AcpRuntime::new(&state_root.join("acp"), AgentHarnessConfig::default()).expect("acp"),
        );
        let ws_state = Arc::new(WsServerState::new(WsServerConfig::default()));

        let workspace_dir = state_root.join("workspace");
        let _ = std::fs::create_dir_all(&workspace_dir);
        let sessions_dir = state_root.join("sessions");
        let _ = std::fs::create_dir_all(&sessions_dir);
        let memory_dir = state_root.join("memory");
        let _ = std::fs::create_dir_all(&memory_dir);
        let agent_dir = state_root.join("agents").join("merlin");
        let _ = std::fs::create_dir_all(&agent_dir);

        let agent_db_path = state_root.join("agent.sqlite");
        let storage =
            magicmerlin_storage::Storage::new(&agent_db_path).expect("storage");
        let session_manager =
            Arc::new(SessionManager::new(storage, &sessions_dir, &memory_dir).expect("sm"));

        let provider_router = Arc::new(ProviderRouter::new(ModelRegistry::default()));
        let engine_config = AgentEngineConfig {
            workspace_dir: workspace_dir.clone(),
            agent_dir,
            agent_name: "merlin".to_string(),
            channel: "test".to_string(),
            ..AgentEngineConfig::default()
        };
        let agent_engine = Arc::new(AgentEngine::new(
            provider_router,
            (*session_manager).clone(),
            engine_config,
        ));
        let mut agent_engines: HashMap<String, Arc<AgentEngine>> = HashMap::new();
        agent_engines.insert("merlin".to_string(), agent_engine.clone());
        let mut tool_registry = ToolRegistry::new();
        register_default_tools(&mut tool_registry);
        let tool_registry = Arc::new(tool_registry);

        AppState {
            providers,
            info,
            scheduler,
            db_path,
            config: Arc::new(Mutex::new(cfg)),
            auth: Arc::new(GatewayAuth::default()),
            events,
            event_history: Arc::new(Mutex::new(Vec::new())),
            run_queue: Arc::new(RunQueue::default()),
            ws_state,
            started_at: Instant::now(),
            presence: Arc::new(Mutex::new(SystemPresence::default())),
            acp,
            agent_engine,
            agent_engines,
            tool_registry,
            session_manager,
            process_manager: ProcessManager::new(),
            workspace_dir,
            port: 0,
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

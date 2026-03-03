use std::{
    fs,
    io::Read as _,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use magicmerlin_compat::{
    providers::{SnapshotBackedProviders, StatusProvider, ToolRegistryProvider},
    COMPAT_VERSION,
};
use serde::Serialize;

mod approvals;
mod plugins;
mod scheduler;
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

    // CLI subcommands.
    if let Some(cmd) = args.command {
        let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);

        match cmd {
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
            serve_http_with_daemon(args.bind, port, providers, info, db_path).await?;
        } else {
            serve_http(args.bind, port, providers, info, db_path).await?;
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
}

async fn serve_http(
    bind: IpAddr,
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    db_path: PathBuf,
) -> Result<()> {
    let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);
    sessions::migrate_sessions(&db_path).await?;
    approvals::migrate_approvals(&db_path).await?;
    let state = AppState {
        providers,
        info,
        scheduler,
        db_path,
    };

    let app = build_router(state);

    let addr = SocketAddr::from((bind, port));
    eprintln!("magicmerlin-gateway listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_http_with_daemon(
    bind: IpAddr,
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    db_path: PathBuf,
) -> Result<()> {
    let scheduler = Arc::new(Scheduler::new(db_path.clone()).await?);
    sessions::migrate_sessions(&db_path).await?;
    approvals::migrate_approvals(&db_path).await?;
    let daemon_handle = scheduler.clone().spawn_daemon();

    let state = AppState {
        providers,
        info,
        scheduler,
        db_path,
    };

    let app = build_router(state);

    let addr = SocketAddr::from((bind, port));
    eprintln!("magicmerlin-gateway (daemon) listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run server in foreground; scheduler runs in background.
    axum::serve(listener, app).await?;

    // If server stops, stop scheduler task too.
    daemon_handle.abort();
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
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
        .route("/plugins", get(http_plugins_list))
        .with_state(state)
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

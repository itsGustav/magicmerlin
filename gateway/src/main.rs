use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Result;
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

mod scheduler;
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

        /// Cron expression (UTC). Example: "*/5 * * * * *" (every 5 seconds)
        #[arg(long)]
        schedule: String,

        /// Kind: http_get | discord_webhook
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

    /// Remove a job by id
    Remove { id: i64 },

    /// Trigger a job once, immediately
    Run { id: i64 },

    /// Pause a job (disable)
    Pause { id: i64 },

    /// Resume a job (enable)
    Resume { id: i64 },

    /// List dead-lettered job failures
    DeadLetters {
        #[arg(long, default_value_t = 50)]
        limit: usize,

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
                    CronCommand::Remove { id } => {
                        scheduler.remove_job(id).await?;
                        println!("ok");
                    }
                    CronCommand::Run { id } => {
                        scheduler.run_job_now(id).await?;
                        println!("ok");
                    }
                    CronCommand::Pause { id } => {
                        scheduler.pause_job(id).await?;
                        println!("ok");
                    }
                    CronCommand::Resume { id } => {
                        scheduler.resume_job(id).await?;
                        println!("ok");
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
        "No action provided. Try: status --json, cron list --json, --print-compat, or --serve 8080"
    );
    Ok(())
}

#[derive(Clone)]
struct AppState {
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    scheduler: Arc<Scheduler>,
}

async fn serve_http(
    bind: IpAddr,
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
    db_path: PathBuf,
) -> Result<()> {
    let scheduler = Arc::new(Scheduler::new(db_path).await?);
    let state = AppState {
        providers,
        info,
        scheduler,
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
    let scheduler = Arc::new(Scheduler::new(db_path).await?);
    let daemon_handle = scheduler.clone().spawn_daemon();

    let state = AppState {
        providers,
        info,
        scheduler,
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

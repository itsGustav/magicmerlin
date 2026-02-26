use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
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
use scheduler::{default_db_path, Scheduler};

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
    },

    /// Remove a job by id
    Remove { id: i64 },

    /// Trigger a job once, immediately
    Run { id: i64 },
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

    // CLI subcommands.
    if let Some(cmd) = args.command {
        let scheduler = Arc::new(Scheduler::new(default_db_path()).await?);

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
                                    "{}\t{}\t{}\t{}\t{}\t{:?}",
                                    j.id, j.name, j.kind, j.enabled, j.schedule, j.next_run_at
                                );
                            }
                        }
                    }
                    CronCommand::Add {
                        name,
                        schedule,
                        kind,
                        payload,
                    } => {
                        let payload_json: serde_json::Value = serde_json::from_str(&payload)?;
                        let id = scheduler
                            .add_job(name, schedule, kind, payload_json)
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
                }
                return Ok(());
            }
        }
    }

    // Back-compat: --serve
    if let Some(port) = args.serve {
        if args.daemon {
            serve_http_with_daemon(port, providers, info).await?;
        } else {
            serve_http(port, providers, info).await?;
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

async fn serve_http(port: u16, providers: SnapshotBackedProviders, info: CompatInfo) -> Result<()> {
    let scheduler = Arc::new(Scheduler::new(default_db_path()).await?);
    let state = AppState {
        providers,
        info,
        scheduler,
    };

    let app = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("magicmerlin-gateway listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_http_with_daemon(
    port: u16,
    providers: SnapshotBackedProviders,
    info: CompatInfo,
) -> Result<()> {
    let scheduler = Arc::new(Scheduler::new(default_db_path()).await?);
    let daemon_handle = scheduler.clone().spawn_daemon();

    let state = AppState {
        providers,
        info,
        scheduler,
    };

    let app = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
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
        .route(
            "/cron",
            get({
                let state = state.clone();
                move || async move {
                    match state.scheduler.list_jobs().await {
                        Ok(jobs) => Json(serde_json::json!({ "jobs": jobs })),
                        Err(e) => Json(serde_json::json!({ "error": format!("{e:#}") })),
                    }
                }
            }),
        )
        .route("/cron/run/:id", post(run_cron_job))
        .with_state(state)
}

async fn run_cron_job(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match state.scheduler.run_job_now(id).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": format!("{e:#}") })),
    }
}

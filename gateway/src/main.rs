use std::net::SocketAddr;

use anyhow::Result;
use axum::{routing::get, Json, Router};
use clap::Parser;
use magicmerlin_compat::{
  providers::{SnapshotBackedProviders, StatusProvider, ToolRegistryProvider},
  COMPAT_VERSION,
};
use serde::Serialize;

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

  /// Emit JSON output for --print-compat.
  #[arg(long)]
  json: bool,
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

  if let Some(port) = args.serve {
    serve_http(port, providers, info).await?;
    return Ok(());
  }

  // Default behavior: be explicit (no silent daemon).
  eprintln!("No action provided. Try --print-compat or --serve 8080");
  Ok(())
}

async fn serve_http(port: u16, providers: SnapshotBackedProviders, info: CompatInfo) -> Result<()> {
  #[derive(Clone)]
  struct AppState {
    providers: SnapshotBackedProviders,
    info: CompatInfo,
  }

  let state = AppState { providers, info };

  let app = Router::new()
    .route("/health", get({
      let state = state.clone();
      move || async move {
        Json(serde_json::json!({
          "status": "ok",
          "compatVersion": state.info.compat_version,
          "fingerprint": state.info.fingerprint,
        }))
      }
    }))
    .route("/status", get({
      let state = state.clone();
      move || async move {
        Json(serde_json::json!({
          "compat": {
            "compatVersion": state.info.compat_version,
            "fingerprint": state.info.fingerprint,
          },
          "openclawStatus": state.providers.openclaw_status_json(),
        }))
      }
    }))
    .route("/tools", get({
      let state = state.clone();
      move || async move {
        Json(serde_json::json!({
          "tools": state.providers.tool_names(),
        }))
      }
    }))
    .route("/snapshots", get({
      let state = state.clone();
      move || async move { Json(state.info.clone()) }
    }));

  let addr = SocketAddr::from(([127, 0, 0, 1], port));
  eprintln!("magicmerlin-gateway listening on http://{addr}");
  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;
  Ok(())
}

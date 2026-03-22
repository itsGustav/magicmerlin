//! Slack Events API webhook handler (alternative to Socket Mode).
//!
//! Handles URL verification challenges and event callbacks via HTTP POST.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use tokio::sync::mpsc;

use super::normalize::normalize_slack_event;
use crate::framework::InboundMessage;

/// Shared state for the webhook handler.
#[derive(Clone)]
pub struct SlackWebhookState {
    pub tx: mpsc::Sender<InboundMessage>,
    pub bot_user_id: Option<String>,
    pub signing_secret: Option<String>,
}

/// Build an axum router for Slack Events API webhooks.
pub fn router(state: Arc<SlackWebhookState>) -> Router {
    Router::new()
        .route("/slack/events", post(handle_events))
        .with_state(state)
}

async fn handle_events(
    State(state): State<Arc<SlackWebhookState>>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let event_type = payload["type"].as_str().unwrap_or("");

    match event_type {
        // URL verification challenge — Slack sends this when you first set up the webhook URL
        "url_verification" => {
            let challenge = payload["challenge"].as_str().unwrap_or("").to_string();
            (
                StatusCode::OK,
                Json(serde_json::json!({"challenge": challenge})),
            )
        }
        // Normal event callback
        "event_callback" => {
            if let Some(event) = payload.get("event") {
                if let Some(inbound) = normalize_slack_event(event, state.bot_user_id.as_deref()) {
                    let _ = state.tx.send(inbound).await;
                }
            }
            (StatusCode::OK, Json(serde_json::json!({"ok": true})))
        }
        _ => (StatusCode::OK, Json(serde_json::json!({"ok": true}))),
    }
}

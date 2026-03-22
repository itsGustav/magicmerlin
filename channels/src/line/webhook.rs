//! LINE webhook handler — receives POST from LINE platform.
//!
//! Validates X-Line-Signature (HMAC-SHA256) and dispatches events.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::framework::InboundMessage;
use super::normalize::normalize_line_event;

type HmacSha256 = Hmac<Sha256>;

/// Shared state for the LINE webhook handler.
#[derive(Clone)]
pub struct LineWebhookState {
    pub tx: mpsc::Sender<InboundMessage>,
    pub channel_secret: String,
}

/// Build an axum router for LINE webhook events.
pub fn router(state: Arc<LineWebhookState>) -> Router {
    Router::new()
        .route("/webhook/line", post(handle_webhook))
        .with_state(state)
}

/// Validate X-Line-Signature using HMAC-SHA256 of the raw body with the channel secret.
pub fn verify_signature(channel_secret: &str, body: &[u8], signature: &str) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(channel_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        mac.finalize().into_bytes(),
    );
    expected == signature
}

async fn handle_webhook(
    State(state): State<Arc<LineWebhookState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // Validate signature
    let signature = match headers.get("x-line-signature").and_then(|v| v.to_str().ok()) {
        Some(sig) => sig,
        None => return StatusCode::UNAUTHORIZED,
    };

    if !verify_signature(&state.channel_secret, &body, signature) {
        return StatusCode::UNAUTHORIZED;
    }

    // Parse payload
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // Process events array
    if let Some(events) = payload["events"].as_array() {
        for event in events {
            if let Some(inbound) = normalize_line_event(event) {
                let _ = state.tx.send(inbound).await;
            }
        }
    }

    StatusCode::OK
}

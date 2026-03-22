use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};

use super::runtime::TelegramChannel;
use super::types::TelegramUpdate;

/// Builds a minimal axum webhook router for Telegram updates.
pub fn router(channel: Arc<TelegramChannel>) -> Router {
    Router::new()
        .route("/telegram/:account", post(handle_webhook))
        .with_state(channel)
}

async fn handle_webhook(
    State(channel): State<Arc<TelegramChannel>>,
    Path(account): Path<String>,
    headers: HeaderMap,
    Json(update): Json<TelegramUpdate>,
) -> StatusCode {
    let secret = headers
        .get("x-telegram-bot-api-secret-token")
        .and_then(|value| value.to_str().ok());
    match channel
        .handle_webhook_update(&account, secret, update)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::UNAUTHORIZED,
    }
}

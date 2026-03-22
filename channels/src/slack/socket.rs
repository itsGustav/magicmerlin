//! Slack Socket Mode — WebSocket event loop.
//!
//! Connects to `wss://wss-primary.slack.com/link/...` via apps.connections.open,
//! receives enveloped events, ACKs each one, and dispatches normalized messages.

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::framework::InboundMessage;
use super::normalize::normalize_slack_event;

/// Errors from the Socket Mode connection.
#[derive(Debug, thiserror::Error)]
pub enum SocketError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("bad handshake response")]
    BadHandshake,
    #[error("connection closed")]
    Closed,
}

type Result<T> = std::result::Result<T, SocketError>;

/// Socket Mode driver that maintains a WebSocket connection to Slack.
pub struct SlackSocketMode {
    app_token: String,
    bot_user_id: Option<String>,
    http: reqwest::Client,
}

impl SlackSocketMode {
    pub fn new(app_token: String, bot_user_id: Option<String>) -> Self {
        Self {
            app_token,
            bot_user_id,
            http: reqwest::Client::new(),
        }
    }

    /// Obtain a WSS URL via `apps.connections.open`.
    async fn get_wss_url(&self) -> Result<String> {
        let resp: Value = self
            .http
            .post("https://slack.com/api/apps.connections.open")
            .bearer_auth(&self.app_token)
            .json(&serde_json::json!({}))
            .send()
            .await?
            .json()
            .await?;

        resp["url"]
            .as_str()
            .map(ToString::to_string)
            .ok_or(SocketError::BadHandshake)
    }

    /// Connect to Slack Socket Mode and dispatch normalized messages.
    ///
    /// This runs until the connection is closed or an error occurs.
    /// The caller should loop + reconnect on `SocketError::Closed`.
    pub async fn connect(&self, tx: mpsc::Sender<InboundMessage>) -> Result<()> {
        let wss_url = self.get_wss_url().await?;
        tracing::info!(url = %wss_url, "Connecting to Slack Socket Mode");

        let (ws_stream, _) = tokio_tungstenite::connect_async(&wss_url).await?;
        let (mut write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            let msg = msg?;
            let text = match msg {
                tokio_tungstenite::tungstenite::Message::Text(t) => t,
                tokio_tungstenite::tungstenite::Message::Close(_) => {
                    tracing::info!("Slack Socket Mode connection closed by server");
                    return Err(SocketError::Closed);
                }
                tokio_tungstenite::tungstenite::Message::Ping(data) => {
                    write
                        .send(tokio_tungstenite::tungstenite::Message::Pong(data))
                        .await?;
                    continue;
                }
                _ => continue,
            };

            let envelope: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // ACK the envelope
            if let Some(envelope_id) = envelope["envelope_id"].as_str() {
                let ack = serde_json::json!({"envelope_id": envelope_id});
                write
                    .send(tokio_tungstenite::tungstenite::Message::Text(
                        ack.to_string().into(),
                    ))
                    .await?;
            }

            // Handle different envelope types
            let envelope_type = envelope["type"].as_str().unwrap_or("");
            match envelope_type {
                "hello" => {
                    tracing::info!("Slack Socket Mode hello received");
                }
                "disconnect" => {
                    tracing::info!(
                        reason = envelope["reason"].as_str().unwrap_or("unknown"),
                        "Slack Socket Mode disconnect requested"
                    );
                    return Err(SocketError::Closed);
                }
                "events_api" => {
                    if let Some(event) = envelope.get("payload").and_then(|p| p.get("event")) {
                        if let Some(inbound) =
                            normalize_slack_event(event, self.bot_user_id.as_deref())
                        {
                            let _ = tx.send(inbound).await;
                        }
                    }
                }
                "interactive" | "slash_commands" => {
                    // Could be extended for interactive messages / slash commands
                    tracing::debug!(envelope_type, "Received non-event envelope");
                }
                _ => {}
            }
        }

        Err(SocketError::Closed)
    }

    /// Run the Socket Mode loop with automatic reconnection.
    pub async fn run_forever(self, tx: mpsc::Sender<InboundMessage>) {
        loop {
            match self.connect(tx.clone()).await {
                Ok(()) => {}
                Err(SocketError::Closed) => {
                    tracing::info!("Reconnecting Slack Socket Mode in 2s...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Slack Socket Mode error, reconnecting in 5s...");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}

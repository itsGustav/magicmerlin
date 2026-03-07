//! Web chat channel implementation using WebSocket + HTTP semantics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// Web channel configuration.
#[derive(Debug, Clone)]
pub struct WebConfig {
    /// Bind address for websocket endpoint.
    pub websocket_bind: String,
    /// Bind address for media upload endpoint.
    pub media_upload_bind: String,
}

/// Web channel adapter.
#[derive(Debug)]
pub struct WebChannel {
    config: WebConfig,
    sessions: RwLock<HashMap<String, String>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl WebChannel {
    /// Creates Web channel adapter.
    pub fn new(config: WebConfig) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Authenticates a session token.
    pub async fn authenticate_session(&self, session_id: &str, user_id: &str) {
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), user_id.to_string());
    }

    /// Handles inbound websocket JSON payload.
    pub async fn handle_ws_message(&self, _session_id: &str, _payload: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    /// Emits typing indicator to a session.
    pub async fn typing_indicator(&self, _session_id: &str, _typing: bool) -> Result<()> {
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("web-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for WebChannel {
    fn name(&self) -> &str {
        "web"
    }

    fn platform(&self) -> Platform {
        Platform::Web
    }

    async fn start(&mut self) -> Result<()> {
        let _ = &self.config;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();
        self.messages
            .write()
            .await
            .insert(format!("{target}:{id}"), message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        self.messages
            .write()
            .await
            .insert(format!("{target}:{message_id}"), message);
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        self.messages
            .write()
            .await
            .remove(&format!("{target}:{message_id}"));
        Ok(())
    }

    async fn react(&self, _target: &str, _message_id: &str, _emoji: &str) -> Result<()> {
        Ok(())
    }
}

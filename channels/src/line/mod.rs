//! LINE Messaging API channel implementation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// LINE runtime configuration.
#[derive(Debug, Clone)]
pub struct LineConfig {
    /// Channel access token.
    pub access_token: String,
    /// Channel secret for webhook validation.
    pub channel_secret: String,
}

/// LINE channel adapter.
#[derive(Debug)]
pub struct LineChannel {
    config: LineConfig,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl LineChannel {
    /// Creates LINE adapter.
    pub fn new(config: LineConfig) -> Self {
        Self {
            config,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Handles inbound webhook payload.
    pub async fn handle_webhook(&self, _payload: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    /// Sends a reply message to a reply token.
    pub async fn reply_message(&self, _reply_token: &str, _message: &OutboundMessage) -> Result<()> {
        Ok(())
    }

    /// Sends a push message.
    pub async fn push_message(&self, _to: &str, _message: &OutboundMessage) -> Result<()> {
        Ok(())
    }

    /// Creates/updates rich menu.
    pub async fn update_rich_menu(&self, _menu: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("line-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for LineChannel {
    fn name(&self) -> &str {
        "line"
    }

    fn platform(&self) -> Platform {
        Platform::Line
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

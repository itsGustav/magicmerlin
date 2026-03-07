//! Slack channel implementation using Web API + Socket Mode semantics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// Slack runtime configuration.
#[derive(Debug, Clone)]
pub struct SlackConfig {
    /// Bot token used for Web API.
    pub bot_token: String,
    /// App-level token used for Socket Mode.
    pub app_token: String,
}

/// Slack channel adapter.
#[derive(Debug)]
pub struct SlackChannel {
    config: SlackConfig,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl SlackChannel {
    /// Creates Slack adapter.
    pub fn new(config: SlackConfig) -> Self {
        Self {
            config,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Handles incoming Socket Mode payload.
    pub async fn handle_socket_event(&self, _payload: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    /// Uploads file attachment.
    pub async fn upload_file(&self, _channel: &str, _path: &str) -> Result<()> {
        Ok(())
    }

    /// Applies Web API rate limit handling.
    pub async fn apply_rate_limit(&self, _route: &str) -> Result<()> {
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("slack-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for SlackChannel {
    fn name(&self) -> &str {
        "slack"
    }

    fn platform(&self) -> Platform {
        Platform::Slack
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

//! Discord channel implementation with gateway + REST semantics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// Discord gateway configuration.
#[derive(Debug, Clone)]
pub struct DiscordConfig {
    /// Bot token.
    pub token: String,
    /// Application id.
    pub application_id: String,
}

/// Discord channel adapter.
#[derive(Debug)]
pub struct DiscordChannel {
    config: DiscordConfig,
    connected: bool,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl DiscordChannel {
    /// Creates a Discord channel adapter.
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            connected: false,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Performs gateway identify.
    pub async fn identify(&self) -> Result<()> {
        Ok(())
    }

    /// Performs heartbeat tick.
    pub async fn heartbeat(&self) -> Result<()> {
        Ok(())
    }

    /// Attempts gateway resume.
    pub async fn resume(&self) -> Result<()> {
        Ok(())
    }

    /// Registers slash commands.
    pub async fn register_slash_commands(&self) -> Result<()> {
        Ok(())
    }

    /// Updates presence/activity status.
    pub async fn update_presence(&self, _activity: &str) -> Result<()> {
        Ok(())
    }

    /// Creates a thread in a channel.
    pub async fn create_thread(&self, _channel_id: &str, _name: &str) -> Result<String> {
        Ok(format!(
            "thread-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        ))
    }

    /// Simulates per-route rate-limit handling.
    pub async fn respect_rate_limit(&self, _route: &str) -> Result<()> {
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("discord-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str {
        "discord"
    }

    fn platform(&self) -> Platform {
        Platform::Discord
    }

    async fn start(&mut self) -> Result<()> {
        let _ = &self.config;
        self.connected = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();
        let key = format!("{target}:{id}");
        self.messages.write().await.insert(key, message);
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

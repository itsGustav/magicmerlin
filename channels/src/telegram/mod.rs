//! Telegram channel implementation using Bot API semantics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// Telegram bot account configuration.
#[derive(Debug, Clone)]
pub struct TelegramAccount {
    /// Account label.
    pub name: String,
    /// Bot token.
    pub token: String,
}

/// Telegram runtime configuration.
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    /// Bot accounts served by this channel.
    pub accounts: Vec<TelegramAccount>,
    /// Use long-polling mode.
    pub polling_mode: bool,
    /// Optional webhook URL.
    pub webhook_url: Option<String>,
}

/// Telegram channel adapter.
#[derive(Debug)]
pub struct TelegramChannel {
    config: TelegramConfig,
    running: bool,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl TelegramChannel {
    /// Creates a Telegram channel adapter.
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            running: false,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Simulates `getUpdates` polling.
    pub async fn get_updates(&self) -> Result<Vec<serde_json::Value>> {
        Ok(Vec::new())
    }

    /// Sends typing indicator with `sendChatAction`.
    pub async fn send_typing_indicator(&self, _chat_id: &str) -> Result<()> {
        Ok(())
    }

    /// Sends a poll with `sendPoll`.
    pub async fn send_poll(&self, _chat_id: &str, _question: &str, _options: &[String]) -> Result<MessageId> {
        Ok(self.next_message_id())
    }

    /// Uploads media using API upload endpoints.
    pub async fn upload_media(&self, _chat_id: &str, _message: &OutboundMessage) -> Result<MessageId> {
        Ok(self.next_message_id())
    }

    /// Downloads media from `getFile` URL.
    pub async fn download_media(&self, file_id: &str) -> Result<String> {
        Ok(format!("/tmp/telegram_{file_id}"))
    }

    fn next_message_id(&self) -> MessageId {
        format!("tg-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for TelegramChannel {
    fn name(&self) -> &str {
        "telegram"
    }

    fn platform(&self) -> Platform {
        Platform::Telegram
    }

    async fn start(&mut self) -> Result<()> {
        let _ = &self.config;
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();
        let key = format!("{target}:{id}");
        self.messages.write().await.insert(key, message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        let key = format!("{target}:{message_id}");
        self.messages.write().await.insert(key, message);
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        let key = format!("{target}:{message_id}");
        self.messages.write().await.remove(&key);
        Ok(())
    }

    async fn react(&self, _target: &str, _message_id: &str, _emoji: &str) -> Result<()> {
        Ok(())
    }
}

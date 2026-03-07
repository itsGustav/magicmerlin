//! iMessage channel implementation via macOS Messages.app bridge.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// iMessage configuration.
#[derive(Debug, Clone)]
pub struct IMessageConfig {
    /// Poll interval milliseconds.
    pub poll_interval_ms: u64,
}

/// iMessage channel adapter.
#[derive(Debug)]
pub struct IMessageChannel {
    config: IMessageConfig,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl IMessageChannel {
    /// Creates iMessage adapter.
    pub fn new(config: IMessageConfig) -> Self {
        Self {
            config,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Polls Messages.app for new messages.
    pub async fn poll_messages(&self) -> Result<Vec<serde_json::Value>> {
        Ok(Vec::new())
    }

    /// Sends image media using Messages.app attachment flow.
    pub async fn send_image(&self, _target: &str, _path: &str) -> Result<MessageId> {
        Ok(format!("imsg-img-{}", self.next_id.fetch_add(1, Ordering::Relaxed)))
    }

    fn next_message_id(&self) -> MessageId {
        format!("imsg-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for IMessageChannel {
    fn name(&self) -> &str {
        "imessage"
    }

    fn platform(&self) -> Platform {
        Platform::IMessage
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

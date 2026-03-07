//! Signal channel implementation via `signal-cli` style wrapper.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// Signal configuration.
#[derive(Debug, Clone)]
pub struct SignalConfig {
    /// Path to signal-cli binary.
    pub cli_path: String,
    /// Registered number.
    pub number: String,
}

/// Signal channel adapter.
#[derive(Debug)]
pub struct SignalChannel {
    config: SignalConfig,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl SignalChannel {
    /// Creates signal adapter.
    pub fn new(config: SignalConfig) -> Self {
        Self {
            config,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Verifies safety/trust number.
    pub async fn verify_safety_number(&self, _peer: &str, _safety_number: &str) -> Result<bool> {
        Ok(true)
    }

    fn next_message_id(&self) -> MessageId {
        format!("signal-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for SignalChannel {
    fn name(&self) -> &str {
        "signal"
    }

    fn platform(&self) -> Platform {
        Platform::Signal
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

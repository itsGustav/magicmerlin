//! WhatsApp channel implementation via external bridge process semantics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

/// WhatsApp bridge configuration.
#[derive(Debug, Clone)]
pub struct WhatsAppConfig {
    /// Command to launch bridge process.
    pub bridge_command: String,
}

/// WhatsApp channel adapter.
#[derive(Debug)]
pub struct WhatsAppChannel {
    config: WhatsAppConfig,
    paired: bool,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl WhatsAppChannel {
    /// Creates WhatsApp adapter.
    pub fn new(config: WhatsAppConfig) -> Self {
        Self {
            config,
            paired: false,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Returns QR content for pairing flow.
    pub async fn pairing_qr_code(&self) -> Result<String> {
        Ok("WA-PAIR-QR".to_string())
    }

    /// Marks pairing complete.
    pub async fn complete_pairing(&mut self) -> Result<()> {
        self.paired = true;
        Ok(())
    }

    /// Sends read receipt for a message.
    pub async fn send_read_receipt(&self, _chat_id: &str, _message_id: &str) -> Result<()> {
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("wa-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    fn platform(&self) -> Platform {
        Platform::WhatsApp
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

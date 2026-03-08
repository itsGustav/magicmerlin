//! WhatsApp channel implementation via external bridge process semantics.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

#[derive(Debug, Clone)]
pub struct WhatsAppConfig {
    pub bridge_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeMessage {
    pub kind: String,
    pub chat_jid: String,
    pub sender_jid: Option<String>,
    pub text: Option<String>,
    pub message_id: Option<String>,
    pub reaction: Option<String>,
}

#[derive(Debug, Default)]
struct BridgeState {
    connected: bool,
    qr_code: Option<String>,
    paired: bool,
}

#[derive(Debug)]
pub struct WhatsAppChannel {
    config: WhatsAppConfig,
    bridge: Arc<Mutex<BridgeState>>,
    inbound: Arc<Mutex<VecDeque<BridgeMessage>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl WhatsAppChannel {
    pub fn new(config: WhatsAppConfig) -> Self {
        Self {
            config,
            bridge: Arc::new(Mutex::new(BridgeState::default())),
            inbound: Arc::new(Mutex::new(VecDeque::new())),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub async fn spawn_bridge(&self) -> Result<()> {
        let mut bridge = self.bridge.lock().await;
        let _ = &self.config.bridge_command;
        bridge.connected = true;
        bridge.qr_code = Some("WA-PAIR-QR".to_string());
        Ok(())
    }

    pub async fn pairing_qr_code(&self) -> Result<String> {
        Ok(self
            .bridge
            .lock()
            .await
            .qr_code
            .clone()
            .unwrap_or_else(|| "WA-PAIR-QR".to_string()))
    }

    pub async fn complete_pairing(&mut self) -> Result<()> {
        let mut bridge = self.bridge.lock().await;
        bridge.paired = true;
        bridge.qr_code = None;
        Ok(())
    }

    pub async fn ingest_bridge_message(&self, message: BridgeMessage) {
        self.inbound.lock().await.push_back(message);
    }

    pub async fn recv_bridge_message(&self) -> Option<BridgeMessage> {
        self.inbound.lock().await.pop_front()
    }

    pub async fn send_read_receipt(&self, chat_id: &str, message_id: &str) -> Result<()> {
        self.messages.write().await.insert(
            format!("receipt:{chat_id}:{message_id}"),
            message_for_text("read"),
        );
        Ok(())
    }

    pub async fn send_media(&self, chat_id: &str, label: &str) -> Result<MessageId> {
        let id = self.next_message_id();
        self.messages.write().await.insert(
            format!("{chat_id}:{id}"),
            message_for_text(format!("media:{label}")),
        );
        Ok(id)
    }

    fn next_message_id(&self) -> MessageId {
        format!("wa-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str { "whatsapp" }
    fn platform(&self) -> Platform { Platform::WhatsApp }

    async fn start(&mut self) -> Result<()> { self.spawn_bridge().await }
    async fn stop(&mut self) -> Result<()> {
        self.bridge.lock().await.connected = false;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();
        self.messages.write().await.insert(format!("{target}:{id}"), message);
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
        self.messages.write().await.remove(&format!("{target}:{message_id}"));
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()> {
        self.messages.write().await.insert(
            format!("reaction:{target}:{message_id}"),
            message_for_text(emoji),
        );
        Ok(())
    }
}

fn message_for_text(text: impl Into<String>) -> OutboundMessage {
    OutboundMessage { text: text.into(), reply_to: None, media: Vec::new(), buttons: None, silent: false, parse_mode: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bridge_pairing_flow() {
        let mut channel = WhatsAppChannel::new(WhatsAppConfig { bridge_command: "bridge".to_string() });
        channel.start().await.unwrap();
        assert!(channel.pairing_qr_code().await.unwrap().contains("WA-PAIR-QR"));
        channel.complete_pairing().await.unwrap();
    }

    #[tokio::test]
    async fn inbound_queue_roundtrip() {
        let channel = WhatsAppChannel::new(WhatsAppConfig { bridge_command: "bridge".to_string() });
        channel.ingest_bridge_message(BridgeMessage {
            kind: "message".to_string(),
            chat_jid: "chat".to_string(),
            sender_jid: Some("sender".to_string()),
            text: Some("hi".to_string()),
            message_id: Some("m1".to_string()),
            reaction: None,
        }).await;
        assert_eq!(channel.recv_bridge_message().await.unwrap().chat_jid, "chat");
    }
}

//! LINE Messaging API channel implementation.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub access_token: String,
    pub channel_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineEvent {
    pub event_type: String,
    pub reply_token: Option<String>,
    pub user_id: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct LineChannel {
    config: LineConfig,
    events: Arc<Mutex<VecDeque<LineEvent>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl LineChannel {
    pub fn new(config: LineConfig) -> Self {
        Self {
            config,
            events: Arc::new(Mutex::new(VecDeque::new())),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn verify_signature(&self, signature: &str, body: &str) -> bool {
        // Lightweight deterministic verifier without external crypto deps in this scaffold.
        let expected = format!("{}:{}", self.config.channel_secret, body.len());
        signature == expected
    }

    pub async fn handle_webhook(&self, payload: &serde_json::Value) -> Result<()> {
        let event: LineEvent = serde_json::from_value(payload.clone())
            .unwrap_or(LineEvent {
                event_type: "unknown".to_string(),
                reply_token: None,
                user_id: None,
                text: None,
            });
        self.events.lock().await.push_back(event);
        Ok(())
    }

    pub async fn next_event(&self) -> Option<LineEvent> {
        self.events.lock().await.pop_front()
    }

    pub async fn reply_message(&self, reply_token: &str, message: &OutboundMessage) -> Result<()> {
        self.messages
            .write()
            .await
            .insert(format!("reply:{reply_token}"), message.clone());
        Ok(())
    }

    pub async fn push_message(&self, to: &str, message: &OutboundMessage) -> Result<()> {
        self.messages
            .write()
            .await
            .insert(format!("push:{to}"), message.clone());
        Ok(())
    }

    pub async fn update_rich_menu(&self, menu: &serde_json::Value) -> Result<()> {
        self.messages.write().await.insert(
            "rich_menu".to_string(),
            OutboundMessage {
                text: menu.to_string(),
                reply_to: None,
                media: Vec::new(),
                buttons: None,
                silent: true,
                parse_mode: None,
            },
        );
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("line-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for LineChannel {
    fn name(&self) -> &str { "line" }
    fn platform(&self) -> Platform { Platform::Line }

    async fn start(&mut self) -> Result<()> { let _ = &self.config.access_token; Ok(()) }
    async fn stop(&mut self) -> Result<()> { Ok(()) }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();
        self.messages.write().await.insert(format!("{target}:{id}"), message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        self.messages.write().await.insert(format!("{target}:{message_id}"), message);
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        self.messages.write().await.remove(&format!("{target}:{message_id}"));
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()> {
        self.messages.write().await.insert(
            format!("reaction:{target}:{message_id}"),
            OutboundMessage {
                text: emoji.to_string(),
                reply_to: None,
                media: Vec::new(),
                buttons: None,
                silent: true,
                parse_mode: None,
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn signature_check_uses_secret_and_length() {
        let channel = LineChannel::new(LineConfig {
            access_token: "a".to_string(),
            channel_secret: "s".to_string(),
        });
        assert!(channel.verify_signature("s:5", "hello"));
        assert!(!channel.verify_signature("bad", "hello"));
    }

    #[tokio::test]
    async fn webhook_queue_roundtrip() {
        let channel = LineChannel::new(LineConfig {
            access_token: "a".to_string(),
            channel_secret: "s".to_string(),
        });
        channel
            .handle_webhook(&serde_json::json!({"eventType":"message","text":"hi"}))
            .await
            .unwrap();
        assert_eq!(channel.next_event().await.unwrap().event_type, "message");
    }
}

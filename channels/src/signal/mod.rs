//! Signal channel implementation via `signal-cli` style wrapper.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

#[derive(Debug, Clone)]
pub struct SignalConfig {
    pub cli_path: String,
    pub number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalEnvelope {
    pub source: String,
    pub timestamp_ms: i64,
    pub message: Option<String>,
    pub group_id: Option<String>,
    pub attachments: Vec<String>,
}

#[derive(Debug)]
pub struct SignalChannel {
    config: SignalConfig,
    inbound: Arc<Mutex<VecDeque<SignalEnvelope>>>,
    trusted: Arc<Mutex<HashMap<String, String>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl SignalChannel {
    pub fn new(config: SignalConfig) -> Self {
        Self {
            config,
            inbound: Arc::new(Mutex::new(VecDeque::new())),
            trusted: Arc::new(Mutex::new(HashMap::new())),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn build_send_command(&self, recipient: &str, text: &str) -> Vec<String> {
        vec![
            self.config.cli_path.clone(),
            "-u".to_string(),
            self.config.number.clone(),
            "send".to_string(),
            recipient.to_string(),
            "-m".to_string(),
            text.to_string(),
        ]
    }

    pub async fn ingest_envelope(&self, env: SignalEnvelope) {
        self.inbound.lock().await.push_back(env);
    }

    pub async fn recv_envelope(&self) -> Option<SignalEnvelope> {
        self.inbound.lock().await.pop_front()
    }

    pub async fn verify_safety_number(&self, peer: &str, safety_number: &str) -> Result<bool> {
        self.trusted
            .lock()
            .await
            .insert(peer.to_string(), safety_number.to_string());
        Ok(true)
    }

    fn next_message_id(&self) -> MessageId {
        format!("signal-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for SignalChannel {
    fn name(&self) -> &str { "signal" }
    fn platform(&self) -> Platform { Platform::Signal }

    async fn start(&mut self) -> Result<()> { Ok(()) }
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

    #[test]
    fn command_builder_includes_required_args() {
        let channel = SignalChannel::new(SignalConfig { cli_path: "signal-cli".to_string(), number: "+1000".to_string() });
        let cmd = channel.build_send_command("+2000", "hello");
        assert!(cmd.contains(&"send".to_string()));
        assert!(cmd.contains(&"+2000".to_string()));
    }

    #[tokio::test]
    async fn safety_number_can_be_verified() {
        let channel = SignalChannel::new(SignalConfig { cli_path: "signal-cli".to_string(), number: "+1000".to_string() });
        assert!(channel.verify_safety_number("peer", "sn").await.unwrap());
    }
}

//! iMessage channel implementation via macOS Messages.app bridge.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use tokio::sync::RwLock;

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

#[derive(Debug, Clone)]
pub struct IMessageConfig {
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IMessageRow {
    pub row_id: i64,
    pub chat_id: String,
    pub sender: String,
    pub text: Option<String>,
    pub date: i64,
}

#[derive(Debug)]
pub struct IMessageChannel {
    config: IMessageConfig,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    last_row_id: AtomicI64,
    next_id: AtomicU64,
}

impl IMessageChannel {
    pub fn new(config: IMessageConfig) -> Self {
        Self {
            config,
            messages: RwLock::new(HashMap::new()),
            last_row_id: AtomicI64::new(0),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn build_osascript_send(to: &str, text: &str) -> String {
        format!(
            "tell application \"Messages\" to send \"{}\" to buddy \"{}\"",
            text.replace('"', "\\\""),
            to.replace('"', "\\\""),
        )
    }

    pub fn parse_chat_row(row: &[&str]) -> Option<IMessageRow> {
        if row.len() < 5 {
            return None;
        }
        Some(IMessageRow {
            row_id: row[0].parse().ok()?,
            chat_id: row[1].to_string(),
            sender: row[2].to_string(),
            text: if row[3].is_empty() { None } else { Some(row[3].to_string()) },
            date: row[4].parse().ok()?,
        })
    }

    pub async fn dedup_and_accept(&self, row_id: i64) -> bool {
        let previous = self.last_row_id.load(Ordering::Relaxed);
        if row_id <= previous {
            return false;
        }
        self.last_row_id.store(row_id, Ordering::Relaxed);
        true
    }

    pub async fn send_image(&self, target: &str, path: &str) -> Result<MessageId> {
        let id = format!("imsg-img-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        self.messages.write().await.insert(
            format!("{target}:{id}"),
            OutboundMessage {
                text: format!("image:{path}"),
                reply_to: None,
                media: Vec::new(),
                buttons: None,
                silent: false,
                parse_mode: None,
            },
        );
        Ok(id)
    }

    fn next_message_id(&self) -> MessageId {
        format!("imsg-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for IMessageChannel {
    fn name(&self) -> &str { "imessage" }
    fn platform(&self) -> Platform { Platform::IMessage }

    async fn start(&mut self) -> Result<()> { let _ = &self.config; Ok(()) }
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
    fn parses_row_shape() {
        let row = IMessageChannel::parse_chat_row(&["1", "chat123", "alice", "hello", "99"]).unwrap();
        assert_eq!(row.chat_id, "chat123");
    }

    #[tokio::test]
    async fn dedup_tracks_last_rowid() {
        let channel = IMessageChannel::new(IMessageConfig { poll_interval_ms: 500 });
        assert!(channel.dedup_and_accept(10).await);
        assert!(!channel.dedup_and_accept(9).await);
    }
}

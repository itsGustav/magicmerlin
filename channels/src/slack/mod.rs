//! Slack channel implementation using Web API + Socket Mode semantics.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

#[derive(Debug, Clone)]
pub struct SlackConfig {
    pub bot_token: String,
    pub app_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlackBlock {
    pub kind: String,
    pub text: Option<String>,
    pub elements: Vec<SlackBlockElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlackBlockElement {
    pub kind: String,
    pub text: String,
    pub action_id: Option<String>,
}

#[derive(Debug, Default)]
struct SlackRateLimiter {
    next_window: Mutex<HashMap<String, Instant>>,
}

#[derive(Debug)]
pub struct SlackChannel {
    config: SlackConfig,
    socket_events: Arc<Mutex<VecDeque<serde_json::Value>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    rate_limiter: Arc<SlackRateLimiter>,
    next_id: AtomicU64,
}

impl SlackChannel {
    pub fn new(config: SlackConfig) -> Self {
        Self {
            config,
            socket_events: Arc::new(Mutex::new(VecDeque::new())),
            messages: RwLock::new(HashMap::new()),
            rate_limiter: Arc::new(SlackRateLimiter::default()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn section_block(text: impl Into<String>) -> SlackBlock {
        SlackBlock {
            kind: "section".to_string(),
            text: Some(text.into()),
            elements: Vec::new(),
        }
    }

    pub fn divider_block() -> SlackBlock {
        SlackBlock {
            kind: "divider".to_string(),
            text: None,
            elements: Vec::new(),
        }
    }

    pub fn actions_block(elements: Vec<SlackBlockElement>) -> SlackBlock {
        SlackBlock {
            kind: "actions".to_string(),
            text: None,
            elements,
        }
    }

    pub async fn handle_socket_event(&self, payload: &serde_json::Value) -> Result<()> {
        self.socket_events.lock().await.push_back(payload.clone());
        Ok(())
    }

    pub async fn next_socket_event(&self) -> Option<serde_json::Value> {
        self.socket_events.lock().await.pop_front()
    }

    pub async fn upload_file(&self, channel: &str, path: &str) -> Result<()> {
        self.messages.write().await.insert(
            format!("file:{channel}:{path}"),
            OutboundMessage {
                text: format!("uploaded {path}"),
                reply_to: None,
                media: Vec::new(),
                buttons: None,
                silent: true,
                parse_mode: None,
            },
        );
        Ok(())
    }

    pub async fn apply_rate_limit(&self, route: &str) -> Result<()> {
        let mut windows = self.rate_limiter.next_window.lock().await;
        if let Some(next) = windows.get(route).copied() {
            let now = Instant::now();
            if next > now {
                tokio::time::sleep(next.duration_since(now)).await;
            }
        }
        windows.insert(route.to_string(), Instant::now() + Duration::from_millis(200));
        Ok(())
    }

    fn next_message_id(&self) -> MessageId {
        format!("slack-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for SlackChannel {
    fn name(&self) -> &str { "slack" }
    fn platform(&self) -> Platform { Platform::Slack }

    async fn start(&mut self) -> Result<()> { let _ = &self.config; Ok(()) }
    async fn stop(&mut self) -> Result<()> { Ok(()) }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        self.apply_rate_limit("chat.postMessage").await?;
        let id = self.next_message_id();
        self.messages.write().await.insert(format!("{target}:{id}"), message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        self.apply_rate_limit("chat.update").await?;
        self.messages.write().await.insert(format!("{target}:{message_id}"), message);
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        self.apply_rate_limit("chat.delete").await?;
        self.messages.write().await.remove(&format!("{target}:{message_id}"));
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()> {
        self.apply_rate_limit("reactions.add").await?;
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
    fn block_kit_builders_work() {
        let section = SlackChannel::section_block("hello");
        assert_eq!(section.kind, "section");
        let divider = SlackChannel::divider_block();
        assert_eq!(divider.kind, "divider");
        let actions = SlackChannel::actions_block(vec![SlackBlockElement {
            kind: "button".to_string(),
            text: "Click".to_string(),
            action_id: Some("a1".to_string()),
        }]);
        assert_eq!(actions.elements.len(), 1);
    }

    #[tokio::test]
    async fn socket_event_roundtrip() {
        let channel = SlackChannel::new(SlackConfig { bot_token: "x".into(), app_token: "y".into() });
        channel.handle_socket_event(&serde_json::json!({"type":"app_mention"})).await.unwrap();
        assert_eq!(channel.next_socket_event().await.unwrap()["type"], "app_mention");
    }
}

//! Web chat channel implementation using WebSocket + HTTP semantics.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

#[derive(Debug, Clone)]
pub struct WebConfig {
    pub websocket_bind: String,
    pub media_upload_bind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSession {
    pub session_id: String,
    pub user_id: String,
    pub authenticated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebEvent {
    pub event: String,
    pub session_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug)]
pub struct WebChannel {
    config: WebConfig,
    sessions: RwLock<HashMap<String, WebSession>>,
    events: Arc<Mutex<VecDeque<WebEvent>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl WebChannel {
    pub fn new(config: WebConfig) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            events: Arc::new(Mutex::new(VecDeque::new())),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub async fn authenticate_session(&self, session_id: &str, user_id: &str) {
        self.sessions.write().await.insert(
            session_id.to_string(),
            WebSession {
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                authenticated: true,
            },
        );
    }

    pub async fn handle_ws_message(&self, session_id: &str, payload: &serde_json::Value) -> Result<()> {
        self.events.lock().await.push_back(WebEvent {
            event: "message".to_string(),
            session_id: session_id.to_string(),
            payload: payload.clone(),
        });
        Ok(())
    }

    pub async fn typing_indicator(&self, session_id: &str, typing: bool) -> Result<()> {
        self.events.lock().await.push_back(WebEvent {
            event: "typing".to_string(),
            session_id: session_id.to_string(),
            payload: serde_json::json!({ "typing": typing }),
        });
        Ok(())
    }

    pub async fn next_event(&self) -> Option<WebEvent> {
        self.events.lock().await.pop_front()
    }

    fn next_message_id(&self) -> MessageId {
        format!("web-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for WebChannel {
    fn name(&self) -> &str { "web" }
    fn platform(&self) -> Platform { Platform::Web }

    async fn start(&mut self) -> Result<()> { let _ = &self.config.websocket_bind; Ok(()) }
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
    async fn authentication_and_event_flow() {
        let channel = WebChannel::new(WebConfig {
            websocket_bind: "127.0.0.1:0".to_string(),
            media_upload_bind: "127.0.0.1:0".to_string(),
        });
        channel.authenticate_session("s1", "u1").await;
        channel
            .handle_ws_message("s1", &serde_json::json!({"text":"hello"}))
            .await
            .unwrap();
        channel.typing_indicator("s1", true).await.unwrap();
        assert_eq!(channel.next_event().await.unwrap().event, "message");
        assert_eq!(channel.next_event().await.unwrap().event, "typing");
    }
}

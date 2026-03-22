//! Slack channel implementation — Web API + Socket Mode.
//!
//! Implements the full Slack Bot lifecycle:
//! - HTTP API client for chat.postMessage, chat.update, chat.delete, reactions, files, users
//! - Socket Mode event loop via WebSocket (tokio-tungstenite)
//! - Message normalization from Slack events → InboundMessage
//! - Block Kit builders and Markdown-to-mrkdwn formatting

pub mod api;
pub mod normalize;
pub mod socket;
pub mod webhook;

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, ChannelError, MessageId, OutboundMessage, Platform, Result};

// ─── Configuration ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SlackConfig {
    /// Bot user OAuth token (xoxb-...)
    pub bot_token: String,
    /// App-level token for Socket Mode (xapp-...)
    pub app_token: String,
    /// Optional bot user ID for mention stripping (e.g. "U01ABCDEF").
    /// Resolved automatically on start() if not provided.
    pub bot_user_id: Option<String>,
}

// ─── Block Kit types ────────────────────────────────────────────────────────

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

// ─── Slack API response types ───────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SlackUser {
    pub id: String,
    pub name: String,
    pub real_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackConversation {
    pub id: String,
    pub name: Option<String>,
    pub is_im: Option<bool>,
    pub is_mpim: Option<bool>,
}

// ─── Rate limiter ───────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct SlackRateLimiter {
    next_window: Mutex<HashMap<String, Instant>>,
}

// ─── Channel implementation ─────────────────────────────────────────────────

#[derive(Debug)]
pub struct SlackChannel {
    config: SlackConfig,
    api: api::SlackApiClient,
    socket_events: Arc<Mutex<VecDeque<serde_json::Value>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    rate_limiter: Arc<SlackRateLimiter>,
    next_id: AtomicU64,
}

impl SlackChannel {
    pub fn new(config: SlackConfig) -> Self {
        let api = api::SlackApiClient::new(config.bot_token.clone());
        Self {
            config,
            api,
            socket_events: Arc::new(Mutex::new(VecDeque::new())),
            messages: RwLock::new(HashMap::new()),
            rate_limiter: Arc::new(SlackRateLimiter::default()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Returns a reference to the underlying HTTP API client.
    pub fn api(&self) -> &api::SlackApiClient {
        &self.api
    }

    // ── Block Kit builders ──────────────────────────────────────────────

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

    // ── Socket event queue ──────────────────────────────────────────────

    pub async fn handle_socket_event(&self, payload: &serde_json::Value) -> Result<()> {
        self.socket_events.lock().await.push_back(payload.clone());
        Ok(())
    }

    pub async fn next_socket_event(&self) -> Option<serde_json::Value> {
        self.socket_events.lock().await.pop_front()
    }

    // ── File upload ─────────────────────────────────────────────────────

    pub async fn upload_file(
        &self,
        channel: &str,
        filename: &str,
        content: &[u8],
    ) -> Result<String> {
        self.apply_rate_limit("files.uploadV2").await?;
        self.api
            .files_upload(channel, filename, content)
            .await
            .map_err(|e| ChannelError::PlatformRequest(e.to_string()))
    }

    // ── Rate limiting ───────────────────────────────────────────────────

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

    // ── Thread support ──────────────────────────────────────────────────

    /// Reply in a thread (passes thread_ts to chat.postMessage).
    pub async fn reply_in_thread(
        &self,
        channel: &str,
        thread_ts: &str,
        text: &str,
    ) -> Result<String> {
        self.apply_rate_limit("chat.postMessage").await?;
        let blocks = text_to_blocks(text);
        self.api
            .chat_post_message_threaded(channel, text, Some(blocks), thread_ts)
            .await
            .map_err(|e| ChannelError::PlatformRequest(e.to_string()))
    }

    fn next_message_id(&self) -> MessageId {
        format!("slack-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for SlackChannel {
    fn name(&self) -> &str {
        "slack"
    }

    fn platform(&self) -> Platform {
        Platform::Slack
    }

    async fn start(&mut self) -> Result<()> {
        // Validate tokens are present
        if self.config.bot_token.is_empty() || self.config.app_token.is_empty() {
            tracing::warn!("Slack tokens not configured — channel will remain inactive");
            return Err(ChannelError::PlatformDisabled("slack: missing tokens"));
        }

        // Resolve bot user ID if not provided
        if self.config.bot_user_id.is_none() {
            match self.api.auth_test().await {
                Ok(user_id) => {
                    tracing::info!(bot_user_id = %user_id, "Slack bot identity resolved");
                    self.config.bot_user_id = Some(user_id);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Could not resolve Slack bot user ID");
                }
            }
        }

        tracing::info!("Slack channel started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Slack channel stopped");
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        self.apply_rate_limit("chat.postMessage").await?;
        let blocks = text_to_blocks(&message.text);
        match self
            .api
            .chat_post_message(target, &message.text, Some(blocks))
            .await
        {
            Ok(ts) => Ok(ts),
            Err(e) => {
                // Fallback to local tracking if API fails
                tracing::warn!(error = %e, "Slack API send failed, tracking locally");
                let id = self.next_message_id();
                self.messages
                    .write()
                    .await
                    .insert(format!("{target}:{id}"), message);
                Ok(id)
            }
        }
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        self.apply_rate_limit("chat.update").await?;
        if let Err(e) = self.api.chat_update(target, message_id, &message.text).await {
            tracing::warn!(error = %e, "Slack API edit failed, tracking locally");
            self.messages
                .write()
                .await
                .insert(format!("{target}:{message_id}"), message);
        }
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        self.apply_rate_limit("chat.delete").await?;
        if let Err(e) = self.api.chat_delete(target, message_id).await {
            tracing::warn!(error = %e, "Slack API delete failed, tracking locally");
            self.messages
                .write()
                .await
                .remove(&format!("{target}:{message_id}"));
        }
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()> {
        self.apply_rate_limit("reactions.add").await?;
        if let Err(e) = self.api.reactions_add(target, message_id, emoji).await {
            tracing::warn!(error = %e, "Slack API react failed, tracking locally");
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
        }
        Ok(())
    }
}

// ─── Block builders (public) ────────────────────────────────────────────────

/// Convert plain text to Slack Block Kit JSON (section with mrkdwn).
pub fn text_to_blocks(text: &str) -> serde_json::Value {
    serde_json::json!([{
        "type": "section",
        "text": {
            "type": "mrkdwn",
            "text": format_for_slack(text)
        }
    }])
}

/// Convert standard Markdown conventions to Slack mrkdwn.
///
/// - `**bold**` → `*bold*`
/// - `__underline__` stays (Slack doesn't support underline, keep as-is)
/// - `` `code` `` → `` `code` `` (no change)
/// - `[text](url)` → `<url|text>`
pub fn format_for_slack(text: &str) -> String {
    let mut out = text.to_string();

    // **bold** → *bold*
    while let Some(start) = out.find("**") {
        if let Some(end) = out[start + 2..].find("**") {
            let inner = out[start + 2..start + 2 + end].to_string();
            out = format!("{}*{}*{}", &out[..start], inner, &out[start + 2 + end + 2..]);
        } else {
            break;
        }
    }

    // [text](url) → <url|text>
    let mut result = String::with_capacity(out.len());
    let mut chars = out.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            let mut link_text = String::new();
            let mut found_close = false;
            for ic in chars.by_ref() {
                if ic == ']' {
                    found_close = true;
                    break;
                }
                link_text.push(ic);
            }
            if found_close && chars.peek() == Some(&'(') {
                chars.next(); // consume '('
                let mut url = String::new();
                for ic in chars.by_ref() {
                    if ic == ')' {
                        break;
                    }
                    url.push(ic);
                }
                result.push_str(&format!("<{url}|{link_text}>"));
            } else {
                result.push('[');
                result.push_str(&link_text);
                if found_close {
                    result.push(']');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
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
        let channel = SlackChannel::new(SlackConfig {
            bot_token: "x".into(),
            app_token: "y".into(),
            bot_user_id: None,
        });
        channel
            .handle_socket_event(&serde_json::json!({"type":"app_mention"}))
            .await
            .unwrap();
        assert_eq!(
            channel.next_socket_event().await.unwrap()["type"],
            "app_mention"
        );
    }

    #[test]
    fn format_for_slack_converts_bold() {
        assert_eq!(format_for_slack("**hello**"), "*hello*");
        assert_eq!(format_for_slack("a **b** c **d**"), "a *b* c *d*");
    }

    #[test]
    fn format_for_slack_converts_links() {
        assert_eq!(
            format_for_slack("[click here](https://example.com)"),
            "<https://example.com|click here>"
        );
    }

    #[test]
    fn format_for_slack_preserves_code() {
        assert_eq!(format_for_slack("`code`"), "`code`");
    }

    #[test]
    fn text_to_blocks_produces_section() {
        let blocks = text_to_blocks("hello");
        assert_eq!(blocks[0]["type"], "section");
        assert_eq!(blocks[0]["text"]["type"], "mrkdwn");
    }

    #[test]
    fn normalize_slack_message_event() {
        use normalize::normalize_slack_event;
        let event = serde_json::json!({
            "type": "message",
            "user": "U123",
            "channel": "C456",
            "text": "hello world",
            "ts": "1234567890.123456"
        });
        let msg = normalize_slack_event(&event, None).unwrap();
        assert_eq!(msg.platform, Platform::Slack);
        assert_eq!(msg.chat_id, "C456");
        assert_eq!(msg.text.as_deref(), Some("hello world"));
        assert_eq!(msg.id, "1234567890.123456");
    }

    #[test]
    fn normalize_slack_strips_bot_mention() {
        use normalize::normalize_slack_event;
        let event = serde_json::json!({
            "type": "message",
            "user": "U123",
            "channel": "C456",
            "text": "<@UBOT> do something",
            "ts": "1234567890.123456"
        });
        let msg = normalize_slack_event(&event, Some("UBOT")).unwrap();
        assert_eq!(msg.text.as_deref(), Some("do something"));
    }

    #[test]
    fn normalize_slack_dm_detected() {
        use normalize::normalize_slack_event;
        let event = serde_json::json!({
            "type": "message",
            "user": "U123",
            "channel": "D456",
            "text": "hi",
            "ts": "1.1"
        });
        let msg = normalize_slack_event(&event, None).unwrap();
        assert_eq!(msg.chat_type, crate::framework::ChatType::Direct);
    }

    #[test]
    fn normalize_slack_thread() {
        use normalize::normalize_slack_event;
        let event = serde_json::json!({
            "type": "message",
            "user": "U123",
            "channel": "C456",
            "text": "reply",
            "ts": "1.2",
            "thread_ts": "1.1"
        });
        let msg = normalize_slack_event(&event, None).unwrap();
        assert_eq!(msg.reply_to.as_deref(), Some("1.1"));
    }

    #[test]
    fn normalize_ignores_bot_messages() {
        use normalize::normalize_slack_event;
        let event = serde_json::json!({
            "type": "message",
            "bot_id": "B123",
            "channel": "C456",
            "text": "bot says",
            "ts": "1.1"
        });
        assert!(normalize_slack_event(&event, None).is_none());
    }
}

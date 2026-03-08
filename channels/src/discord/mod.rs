//! Discord channel implementation with gateway + REST semantics.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

pub const DISCORD_MAX_MESSAGE_LEN: usize = 2000;

/// Discord gateway configuration.
#[derive(Debug, Clone)]
pub struct DiscordConfig {
    /// Bot token.
    pub token: String,
    /// Application id.
    pub application_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordHello {
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordSession {
    pub session_id: String,
    pub sequence: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<u32>,
    pub fields: Vec<DiscordEmbedField>,
    pub footer: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordEmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

impl DiscordEmbed {
    pub fn builder() -> DiscordEmbedBuilder {
        DiscordEmbedBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct DiscordEmbedBuilder {
    title: Option<String>,
    description: Option<String>,
    color: Option<u32>,
    fields: Vec<DiscordEmbedField>,
    footer: Option<String>,
    thumbnail_url: Option<String>,
}

impl DiscordEmbedBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn field(mut self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        self.fields.push(DiscordEmbedField {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail_url = Some(url.into());
        self
    }

    pub fn build(self) -> DiscordEmbed {
        DiscordEmbed {
            title: self.title,
            description: self.description,
            color: self.color,
            fields: self.fields,
            footer: self.footer,
            thumbnail_url: self.thumbnail_url,
        }
    }
}

#[derive(Debug, Clone)]
struct RateLimitBucket {
    reset_at: Instant,
    remaining: u32,
}

#[derive(Debug, Default)]
struct RateLimiter {
    buckets: Mutex<HashMap<String, RateLimitBucket>>,
}

impl RateLimiter {
    async fn apply_headers(
        &self,
        route: &str,
        limit: Option<u32>,
        remaining: Option<u32>,
        reset_after_seconds: Option<f64>,
    ) {
        let mut buckets = self.buckets.lock().await;
        let reset_after = reset_after_seconds.unwrap_or(0.0).max(0.0);
        buckets.insert(
            route.to_string(),
            RateLimitBucket {
                reset_at: Instant::now() + Duration::from_secs_f64(reset_after),
                remaining: remaining.or(limit).unwrap_or(1),
            },
        );
    }

    async fn wait_route(&self, route: &str) {
        let wait = {
            let mut buckets = self.buckets.lock().await;
            if let Some(bucket) = buckets.get_mut(route) {
                if bucket.remaining > 0 {
                    bucket.remaining -= 1;
                    None
                } else {
                    let now = Instant::now();
                    if bucket.reset_at > now {
                        Some(bucket.reset_at.duration_since(now))
                    } else {
                        bucket.remaining = 0;
                        None
                    }
                }
            } else {
                None
            }
        };

        if let Some(wait) = wait {
            tokio::time::sleep(wait).await;
        }
    }
}

/// Discord channel adapter.
#[derive(Debug)]
pub struct DiscordChannel {
    config: DiscordConfig,
    connected: bool,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    session: Arc<Mutex<Option<DiscordSession>>>,
    last_heartbeat: Arc<Mutex<Option<Instant>>>,
    intents: Arc<Mutex<Vec<String>>>,
    rate_limiter: Arc<RateLimiter>,
    presence: Arc<Mutex<String>>,
    slash_commands: Arc<Mutex<BTreeMap<String, serde_json::Value>>>,
    inbound_queue: Arc<Mutex<VecDeque<serde_json::Value>>>,
    next_id: AtomicU64,
}

impl DiscordChannel {
    /// Creates a Discord channel adapter.
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            connected: false,
            messages: RwLock::new(HashMap::new()),
            session: Arc::new(Mutex::new(None)),
            last_heartbeat: Arc::new(Mutex::new(None)),
            intents: Arc::new(Mutex::new(vec![
                "GUILDS".to_string(),
                "GUILD_MESSAGES".to_string(),
                "MESSAGE_CONTENT".to_string(),
                "DIRECT_MESSAGES".to_string(),
            ])),
            rate_limiter: Arc::new(RateLimiter::default()),
            presence: Arc::new(Mutex::new("online".to_string())),
            slash_commands: Arc::new(Mutex::new(BTreeMap::new())),
            inbound_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: AtomicU64::new(1),
        }
    }

    /// Performs gateway identify.
    pub async fn identify(&self) -> Result<serde_json::Value> {
        let intents = self.intents.lock().await.clone();
        Ok(serde_json::json!({
            "op": 2,
            "d": {
                "token": "***",
                "intents": intents,
                "properties": {
                    "$os": std::env::consts::OS,
                    "$browser": "magicmerlin",
                    "$device": "magicmerlin"
                }
            }
        }))
    }

    /// Performs heartbeat tick.
    pub async fn heartbeat(&self) -> Result<()> {
        *self.last_heartbeat.lock().await = Some(Instant::now());
        Ok(())
    }

    /// Attempts gateway resume.
    pub async fn resume(&self) -> Result<serde_json::Value> {
        let session = self.session.lock().await.clone();
        Ok(match session {
            Some(session) => serde_json::json!({
                "op": 6,
                "d": {
                    "token": "***",
                    "session_id": session.session_id,
                    "seq": session.sequence,
                }
            }),
            None => serde_json::json!({"resume": false}),
        })
    }

    pub async fn on_gateway_hello(&self, hello: DiscordHello) {
        let mut queue = self.inbound_queue.lock().await;
        queue.push_back(serde_json::json!({
            "event": "HELLO",
            "heartbeatIntervalMs": hello.heartbeat_interval_ms,
        }));
    }

    pub async fn on_gateway_dispatch(&self, sequence: u64, session_id: Option<String>) {
        let mut session = self.session.lock().await;
        let updated = match session.as_mut() {
            Some(existing) => {
                existing.sequence = Some(sequence);
                if let Some(new_id) = session_id {
                    existing.session_id = new_id;
                }
                existing.clone()
            }
            None => DiscordSession {
                session_id: session_id.unwrap_or_else(|| format!("sess-{}", sequence)),
                sequence: Some(sequence),
            },
        };
        *session = Some(updated);
    }

    /// Registers slash commands.
    pub async fn register_slash_commands(
        &self,
        command_name: &str,
        command: serde_json::Value,
    ) -> Result<()> {
        self.slash_commands
            .lock()
            .await
            .insert(command_name.to_string(), command);
        Ok(())
    }

    /// Updates presence/activity status.
    pub async fn update_presence(&self, activity: &str) -> Result<()> {
        *self.presence.lock().await = activity.to_string();
        Ok(())
    }

    /// Creates a thread in a channel.
    pub async fn create_thread(&self, channel_id: &str, name: &str) -> Result<String> {
        self.rate_limiter.wait_route("POST:/threads").await;
        Ok(format!("thread-{channel_id}-{name}-{}", self.next_id.fetch_add(1, Ordering::Relaxed)))
    }

    /// Simulates per-route rate-limit handling.
    pub async fn respect_rate_limit(
        &self,
        route: &str,
        limit: Option<u32>,
        remaining: Option<u32>,
        reset_after_seconds: Option<f64>,
    ) -> Result<()> {
        self.rate_limiter
            .apply_headers(route, limit, remaining, reset_after_seconds)
            .await;
        self.rate_limiter.wait_route(route).await;
        Ok(())
    }

    pub fn split_message(text: &str) -> Vec<String> {
        if text.len() <= DISCORD_MAX_MESSAGE_LEN {
            return vec![text.to_string()];
        }

        let mut parts = Vec::new();
        let mut current = String::new();

        for line in text.lines() {
            let candidate = if current.is_empty() {
                line.to_string()
            } else {
                format!("{current}\n{line}")
            };

            if candidate.len() <= DISCORD_MAX_MESSAGE_LEN {
                current = candidate;
            } else {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
                if line.len() > DISCORD_MAX_MESSAGE_LEN {
                    let mut start = 0usize;
                    while start < line.len() {
                        let end = (start + DISCORD_MAX_MESSAGE_LEN).min(line.len());
                        parts.push(line[start..end].to_string());
                        start = end;
                    }
                } else {
                    current.push_str(line);
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            parts.push(String::new());
        }

        parts
    }

    fn next_message_id(&self) -> MessageId {
        format!("discord-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str {
        "discord"
    }

    fn platform(&self) -> Platform {
        Platform::Discord
    }

    async fn start(&mut self) -> Result<()> {
        let _ = &self.config;
        self.connected = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let chunks = Self::split_message(&message.text);
        let mut last = None;
        for chunk in chunks {
            let id = self.next_message_id();
            self.messages
                .write()
                .await
                .insert(format!("{target}:{id}"), OutboundMessage { text: chunk, ..message.clone() });
            last = Some(id);
        }

        Ok(last.unwrap_or_else(|| self.next_message_id()))
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

    fn build_channel() -> DiscordChannel {
        DiscordChannel::new(DiscordConfig {
            token: "token".to_string(),
            application_id: "app".to_string(),
        })
    }

    #[test]
    fn split_respects_discord_limit() {
        let text = "x".repeat(DISCORD_MAX_MESSAGE_LEN * 2 + 10);
        let parts = DiscordChannel::split_message(&text);
        assert!(parts.len() >= 2);
        assert!(parts.iter().all(|part| part.len() <= DISCORD_MAX_MESSAGE_LEN));
    }

    #[tokio::test]
    async fn embed_builder_creates_payload() {
        let embed = DiscordEmbed::builder()
            .title("Build")
            .description("Description")
            .field("A", "B", true)
            .color(0x112233)
            .footer("footer")
            .thumbnail("https://example.com")
            .build();
        assert_eq!(embed.fields.len(), 1);
        assert_eq!(embed.color, Some(0x112233));
    }

    #[tokio::test]
    async fn resume_payload_includes_session_state() {
        let channel = build_channel();
        channel.on_gateway_dispatch(42, Some("s1".to_string())).await;
        let resume = channel.resume().await.unwrap();
        assert_eq!(resume["op"], 6);
        assert_eq!(resume["d"]["session_id"], "s1");
    }

    #[tokio::test]
    async fn rate_limit_waits_when_bucket_empty() {
        let channel = build_channel();
        channel
            .respect_rate_limit("POST:/messages", Some(1), Some(0), Some(0.2))
            .await
            .unwrap();
        let now = Instant::now();
        channel
            .respect_rate_limit("POST:/messages", Some(1), Some(0), Some(0.2))
            .await
            .unwrap();
        assert!(now.elapsed() >= Duration::from_millis(180));
    }
}

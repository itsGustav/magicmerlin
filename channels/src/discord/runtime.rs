use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, ChatType, MessageId, OutboundMessage, Platform, Result};

use super::types::{
    session_scope, DiscordApiError, DiscordAttachment, DiscordConfig, DiscordEmbed,
    DiscordGatewayState, DiscordHealth, DiscordHello, DiscordInteraction,
    DiscordInteractionResponse, DiscordMessage, DiscordPresence, DiscordProcessedEvent,
    DiscordResponseKind, DiscordSession, DiscordThread, DISCORD_MAX_MESSAGE_LEN, DurationHolder,
};

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

pub type DiscordResult<T> = std::result::Result<T, DiscordApiError>;

#[derive(Debug)]
pub struct DiscordChannel {
    config: DiscordConfig,
    connected: AtomicBool,
    messages: RwLock<HashMap<String, DiscordMessage>>,
    history: RwLock<HashMap<String, Vec<MessageId>>>,
    reactions: RwLock<HashMap<String, Vec<String>>>,
    threads: RwLock<HashMap<String, DiscordThread>>,
    session: Arc<Mutex<Option<DiscordSession>>>,
    health: Arc<RwLock<DiscordHealth>>,
    last_heartbeat: Arc<Mutex<Option<Instant>>>,
    intents: Arc<Mutex<Vec<String>>>,
    rate_limiter: Arc<RateLimiter>,
    presence: Arc<Mutex<DiscordPresence>>,
    slash_commands: Arc<Mutex<BTreeMap<String, serde_json::Value>>>,
    interactions: Arc<Mutex<VecDeque<DiscordInteraction>>>,
    interaction_responses: Arc<Mutex<Vec<DiscordInteractionResponse>>>,
    processed_events: Arc<Mutex<Vec<DiscordProcessedEvent>>>,
    next_id: AtomicU64,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            connected: AtomicBool::new(false),
            messages: RwLock::new(HashMap::new()),
            history: RwLock::new(HashMap::new()),
            reactions: RwLock::new(HashMap::new()),
            threads: RwLock::new(HashMap::new()),
            session: Arc::new(Mutex::new(None)),
            health: Arc::new(RwLock::new(DiscordHealth {
                state: DiscordGatewayState::Disconnected,
                last_sequence: None,
                heartbeat_interval: None,
                last_error: None,
            })),
            last_heartbeat: Arc::new(Mutex::new(None)),
            intents: Arc::new(Mutex::new(vec![
                "GUILDS".to_string(),
                "GUILD_MESSAGES".to_string(),
                "MESSAGE_CONTENT".to_string(),
                "DIRECT_MESSAGES".to_string(),
            ])),
            rate_limiter: Arc::new(RateLimiter::default()),
            presence: Arc::new(Mutex::new(DiscordPresence {
                status: "online".to_string(),
                activity: "starting".to_string(),
            })),
            slash_commands: Arc::new(Mutex::new(BTreeMap::new())),
            interactions: Arc::new(Mutex::new(VecDeque::new())),
            interaction_responses: Arc::new(Mutex::new(Vec::new())),
            processed_events: Arc::new(Mutex::new(Vec::new())),
            next_id: AtomicU64::new(1),
        }
    }

    pub async fn identify(&self) -> Result<serde_json::Value> {
        let intents = self.intents.lock().await.clone();
        self.health.write().await.state = DiscordGatewayState::Identified;
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

    pub async fn heartbeat(&self) -> Result<()> {
        *self.last_heartbeat.lock().await = Some(Instant::now());
        Ok(())
    }

    pub async fn resume(&self) -> Result<serde_json::Value> {
        self.health.write().await.state = DiscordGatewayState::Resuming;
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

    pub async fn reconnect(&self) -> Result<()> {
        self.health.write().await.state = DiscordGatewayState::Connecting;
        Ok(())
    }

    pub async fn on_gateway_hello(&self, hello: DiscordHello) {
        let mut health = self.health.write().await;
        health.heartbeat_interval = Some(DurationHolder::from(Duration::from_millis(
            hello.heartbeat_interval_ms,
        )));
        health.state = DiscordGatewayState::Connecting;
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
                resume_gateway_url: None,
            },
        };
        *session = Some(updated.clone());

        let mut health = self.health.write().await;
        health.last_sequence = updated.sequence;
        health.state = DiscordGatewayState::Ready;
    }

    pub async fn register_slash_command(
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

    pub async fn registered_commands(&self) -> Vec<String> {
        self.slash_commands.lock().await.keys().cloned().collect()
    }

    pub async fn queue_interaction(&self, interaction: DiscordInteraction) {
        self.interactions.lock().await.push_back(interaction);
    }

    pub async fn process_next_interaction(&self) -> Result<Option<DiscordProcessedEvent>> {
        let Some(interaction) = self.interactions.lock().await.pop_front() else {
            return Ok(None);
        };
        let scope = session_scope(
            if interaction.guild_id.is_some() {
                ChatType::Group
            } else {
                ChatType::Direct
            },
            interaction.guild_id.as_deref(),
            &interaction.channel_id,
            interaction.thread_id.as_deref(),
            &interaction.user_id,
        );
        let event = DiscordProcessedEvent {
            kind: format!("slash:{}", interaction.command_name),
            channel_id: interaction.channel_id,
            guild_id: interaction.guild_id,
            thread_id: interaction.thread_id,
            session_scope: scope,
        };
        self.processed_events.lock().await.push(event.clone());
        Ok(Some(event))
    }

    pub async fn processed_events(&self) -> Vec<DiscordProcessedEvent> {
        self.processed_events.lock().await.clone()
    }

    pub async fn defer_interaction(&self, interaction_id: &str) -> Result<()> {
        self.interaction_responses.lock().await.push(DiscordInteractionResponse {
            interaction_id: interaction_id.to_string(),
            kind: DiscordResponseKind::Deferred,
            content: String::new(),
        });
        Ok(())
    }

    pub async fn respond_to_interaction(&self, interaction_id: &str, content: &str) -> Result<()> {
        self.interaction_responses.lock().await.push(DiscordInteractionResponse {
            interaction_id: interaction_id.to_string(),
            kind: DiscordResponseKind::Immediate,
            content: content.to_string(),
        });
        Ok(())
    }

    pub async fn followup_interaction(&self, interaction_id: &str, content: &str) -> Result<()> {
        self.interaction_responses.lock().await.push(DiscordInteractionResponse {
            interaction_id: interaction_id.to_string(),
            kind: DiscordResponseKind::Followup,
            content: content.to_string(),
        });
        Ok(())
    }

    pub async fn interaction_responses(&self) -> Vec<DiscordInteractionResponse> {
        self.interaction_responses.lock().await.clone()
    }

    pub async fn update_presence(&self, activity: &str) -> Result<()> {
        let mut presence = self.presence.lock().await;
        presence.activity = activity.to_string();
        Ok(())
    }

    pub async fn presence(&self) -> DiscordPresence {
        self.presence.lock().await.clone()
    }

    pub async fn health(&self) -> DiscordHealth {
        self.health.read().await.clone()
    }

    pub async fn create_thread(
        &self,
        channel_id: &str,
        guild_id: Option<&str>,
        name: &str,
    ) -> Result<String> {
        self.rate_limiter.wait_route("POST:/threads").await;
        let id = format!(
            "thread-{channel_id}-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        );
        self.threads.write().await.insert(
            id.clone(),
            DiscordThread {
                id: id.clone(),
                channel_id: channel_id.to_string(),
                guild_id: guild_id.map(ToString::to_string),
                name: name.to_string(),
            },
        );
        Ok(id)
    }

    pub async fn threads(&self) -> Vec<DiscordThread> {
        self.threads.read().await.values().cloned().collect()
    }

    pub async fn list_channels(&self, guild_id: &str) -> Result<Vec<String>> {
        let messages = self.history.read().await;
        let mut channels = messages
            .keys()
            .filter(|key| key.starts_with(&format!("guild:{guild_id}:")))
            .map(|key| key.trim_start_matches(&format!("guild:{guild_id}:")).to_string())
            .collect::<Vec<_>>();
        channels.sort();
        channels.dedup();
        Ok(channels)
    }

    pub async fn send_typing_indicator(&self, channel_id: &str) -> Result<()> {
        self.processed_events.lock().await.push(DiscordProcessedEvent {
            kind: "typing".to_string(),
            channel_id: channel_id.to_string(),
            guild_id: None,
            thread_id: None,
            session_scope: format!("discord:channel:{channel_id}"),
        });
        Ok(())
    }

    pub async fn send_message(
        &self,
        channel_id: &str,
        guild_id: Option<&str>,
        author_id: &str,
        message: OutboundMessage,
        embeds: Vec<DiscordEmbed>,
        attachments: Vec<DiscordAttachment>,
        thread_id: Option<&str>,
    ) -> Result<MessageId> {
        self.send_typing_indicator(channel_id).await?;
        let chunks = Self::split_message(&message.text);
        let mut last = None;
        for chunk in chunks {
            let id = self.next_message_id();
            let stored = DiscordMessage {
                id: id.clone(),
                channel_id: channel_id.to_string(),
                guild_id: guild_id.map(ToString::to_string),
                author_id: author_id.to_string(),
                author_name: None,
                content: chunk,
                thread_id: thread_id.map(ToString::to_string),
                reply_to: message.reply_to.clone(),
                attachments: attachments.clone(),
                embeds: embeds.clone(),
                parse_mode: message.parse_mode,
            };
            self.messages.write().await.insert(id.clone(), stored);
            self.history
                .write()
                .await
                .entry(history_key(guild_id, channel_id))
                .or_default()
                .push(id.clone());
            last = Some(id);
        }
        Ok(last.unwrap_or_else(|| self.next_message_id()))
    }

    pub async fn fetch_message_history(
        &self,
        channel_id: &str,
        guild_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DiscordMessage>> {
        let history = self.history.read().await;
        let ids = history
            .get(&history_key(guild_id, channel_id))
            .cloned()
            .unwrap_or_default();
        let messages = self.messages.read().await;
        Ok(ids
            .into_iter()
            .rev()
            .take(limit)
            .filter_map(|id| messages.get(&id).cloned())
            .collect())
    }

    pub async fn edit_message(&self, message_id: &str, content: &str) -> Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(message) = messages.get_mut(message_id) {
            message.content = content.to_string();
            Ok(())
        } else {
            Err(crate::framework::ChannelError::PlatformRequest(
                DiscordApiError::invalid_input("message not found").to_string(),
            ))
        }
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        self.messages.write().await.remove(message_id);
        Ok(())
    }

    pub async fn add_reaction(&self, message_id: &str, emoji: &str) -> Result<()> {
        self.reactions
            .write()
            .await
            .entry(message_id.to_string())
            .or_default()
            .push(emoji.to_string());
        Ok(())
    }

    pub async fn remove_reaction(&self, message_id: &str, emoji: &str) -> Result<()> {
        if let Some(reactions) = self.reactions.write().await.get_mut(message_id) {
            reactions.retain(|existing| existing != emoji);
        }
        Ok(())
    }

    pub async fn reactions(&self, message_id: &str) -> Vec<String> {
        self.reactions
            .read()
            .await
            .get(message_id)
            .cloned()
            .unwrap_or_default()
    }

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

fn history_key(guild_id: Option<&str>, channel_id: &str) -> String {
    guild_id
        .map(|guild_id| format!("guild:{guild_id}:{channel_id}"))
        .unwrap_or_else(|| format!("dm:{channel_id}"))
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
        self.connected.store(true, Ordering::Relaxed);
        self.health.write().await.state = DiscordGatewayState::Connecting;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.connected.store(false, Ordering::Relaxed);
        self.health.write().await.state = DiscordGatewayState::Disconnected;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        self.send_message(target, None, "bot", message, Vec::new(), Vec::new(), None)
            .await
    }

    async fn edit(&self, _target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        self.edit_message(message_id, &message.text).await
    }

    async fn delete(&self, _target: &str, message_id: &str) -> Result<()> {
        self.delete_message(message_id).await
    }

    async fn react(&self, _target: &str, message_id: &str, emoji: &str) -> Result<()> {
        self.add_reaction(message_id, emoji).await
    }
}

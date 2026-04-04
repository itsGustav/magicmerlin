use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, RwLock};

use crate::framework::{
    Channel, ChannelError, ChatType, DmPolicy, DmPolicyEnforcer, InboundMessage, MentionGate,
    MessageId, OutboundMessage, Platform, Result, Sender,
};

use super::audit::AuditLogStore;
use super::channel_mgmt::ChannelManager;
use super::components::{ActionRow, ComponentInteraction, ComponentManager, Modal};
use super::guild::GuildManager;
use super::scheduled_events::ScheduledEventManager;
use super::types::{
    session_scope, DiscordApiError, DiscordAttachment, DiscordConfig, DiscordEmbed,
    DiscordGatewayState, DiscordGuildChannel, DiscordGuildMember, DiscordHealth, DiscordHello,
    DiscordInteraction, DiscordInteractionResponse, DiscordMessage, DiscordPresence,
    DiscordProcessedEvent, DiscordResponseKind, DiscordSession, DiscordThread, DurationHolder,
    DISCORD_MAX_MESSAGE_LEN,
};
use super::voice::VoiceStateTracker;
use super::webhook::WebhookManager;

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
    dm_policy: Arc<Mutex<DmPolicyEnforcer>>,
    mention_gate: MentionGate,
    pinned: Arc<RwLock<HashMap<String, Vec<String>>>>,
    component_manager: ComponentManager,
    guild_manager: GuildManager,
    webhook_manager: WebhookManager,
    voice_tracker: VoiceStateTracker,
    audit_log: AuditLogStore,
    channel_manager: ChannelManager,
    event_manager: ScheduledEventManager,
    guild_channels: RwLock<HashMap<String, Vec<DiscordGuildChannel>>>,
    guild_members: RwLock<HashMap<String, Vec<DiscordGuildMember>>>,
    dm_channels: RwLock<HashMap<String, String>>,
    next_id: AtomicU64,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig) -> Self {
        let dm_enabled = config.dm_enabled;
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
            dm_policy: Arc::new(Mutex::new(if dm_enabled {
                DmPolicyEnforcer::new(DmPolicy::Open)
            } else {
                DmPolicyEnforcer::new(DmPolicy::Allowlist)
            })),
            mention_gate: MentionGate::new("magicmerlin", true),
            pinned: Arc::new(RwLock::new(HashMap::new())),
            component_manager: ComponentManager::new(),
            guild_manager: GuildManager::new(),
            webhook_manager: WebhookManager::new(),
            voice_tracker: VoiceStateTracker::new(),
            audit_log: AuditLogStore::new(),
            channel_manager: ChannelManager::new(),
            event_manager: ScheduledEventManager::new(),
            guild_channels: RwLock::new(HashMap::new()),
            guild_members: RwLock::new(HashMap::new()),
            dm_channels: RwLock::new(HashMap::new()),
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

    pub async fn allow_dm_user(&self, user_id: &str) {
        self.dm_policy.lock().await.allow_user(user_id.to_string());
    }

    pub async fn approve_paired_user(&self, user_id: &str) {
        self.dm_policy
            .lock()
            .await
            .approve_pairing(user_id.to_string());
    }

    pub async fn allows_inbound(
        &self,
        chat_type: ChatType,
        guild_id: Option<&str>,
        channel_id: &str,
        user_id: &str,
        text: Option<&str>,
    ) -> Result<()> {
        if let Some(guild_id) = guild_id {
            if !self.config.guild_allowlist.is_empty()
                && !self
                    .config
                    .guild_allowlist
                    .iter()
                    .any(|allowed| allowed == guild_id)
            {
                return Err(ChannelError::PlatformRequest(format!(
                    "discord forbidden: guild {guild_id} not allowlisted"
                )));
            }
        }

        if !self.config.channel_allowlist.is_empty()
            && !self
                .config
                .channel_allowlist
                .iter()
                .any(|allowed| allowed == channel_id)
        {
            return Err(ChannelError::PlatformRequest(format!(
                "discord forbidden: channel {channel_id} not allowlisted"
            )));
        }

        let inbound = InboundMessage {
            id: "discord-inbound".to_string(),
            platform: Platform::Discord,
            chat_id: channel_id.to_string(),
            chat_type,
            sender: Sender {
                id: user_id.to_string(),
                name: user_id.to_string(),
                username: None,
            },
            text: text.map(ToString::to_string),
            reply_to: None,
            media: Vec::new(),
            timestamp: chrono::Utc::now(),
            raw: serde_json::json!({}),
        };

        if !self.dm_policy.lock().await.allows(&inbound) {
            return Err(ChannelError::PlatformRequest(
                "discord forbidden: dm policy blocked sender".to_string(),
            ));
        }

        if !self.mention_gate.should_process(&inbound) {
            return Err(ChannelError::PlatformRequest(
                "discord forbidden: mention gate blocked group message".to_string(),
            ));
        }

        Ok(())
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
        self.interaction_responses
            .lock()
            .await
            .push(DiscordInteractionResponse {
                interaction_id: interaction_id.to_string(),
                kind: DiscordResponseKind::Deferred,
                content: String::new(),
            });
        Ok(())
    }

    pub async fn respond_to_interaction(&self, interaction_id: &str, content: &str) -> Result<()> {
        self.interaction_responses
            .lock()
            .await
            .push(DiscordInteractionResponse {
                interaction_id: interaction_id.to_string(),
                kind: DiscordResponseKind::Immediate,
                content: content.to_string(),
            });
        Ok(())
    }

    pub async fn followup_interaction(&self, interaction_id: &str, content: &str) -> Result<()> {
        self.interaction_responses
            .lock()
            .await
            .push(DiscordInteractionResponse {
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
            .map(|key| {
                key.trim_start_matches(&format!("guild:{guild_id}:"))
                    .to_string()
            })
            .collect::<Vec<_>>();
        channels.sort();
        channels.dedup();
        Ok(channels)
    }

    pub async fn send_typing_indicator(&self, channel_id: &str) -> Result<()> {
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
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

    // -- Bulk operations ---

    /// Delete multiple messages at once (Discord allows up to 100).
    pub async fn bulk_delete_messages(&self, message_ids: &[&str]) -> Result<u64> {
        let mut messages = self.messages.write().await;
        let mut deleted = 0u64;
        for id in message_ids {
            if messages.remove(*id).is_some() {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    // -- Pin / Unpin ---

    /// Pin a message in a channel.
    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let messages = self.messages.read().await;
        if !messages.contains_key(message_id) {
            return Err(ChannelError::PlatformRequest(
                DiscordApiError::invalid_input("message not found").to_string(),
            ));
        }
        drop(messages);
        let mut pinned = self.pinned.write().await;
        let channel_pins = pinned.entry(channel_id.to_string()).or_default();
        if !channel_pins.contains(&message_id.to_string()) {
            channel_pins.push(message_id.to_string());
        }
        Ok(())
    }

    /// Unpin a message from a channel.
    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let mut pinned = self.pinned.write().await;
        if let Some(channel_pins) = pinned.get_mut(channel_id) {
            channel_pins.retain(|id| id != message_id);
        }
        Ok(())
    }

    /// Get pinned messages in a channel.
    pub async fn pinned_messages(&self, channel_id: &str) -> Vec<DiscordMessage> {
        let pinned = self.pinned.read().await;
        let pin_ids = match pinned.get(channel_id) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        drop(pinned);
        let messages = self.messages.read().await;
        pin_ids
            .iter()
            .filter_map(|id| messages.get(id).cloned())
            .collect()
    }

    // -- Component sends ---

    /// Send a message with action rows (buttons / select menus).
    pub async fn send_message_with_components(
        &self,
        channel_id: &str,
        guild_id: Option<&str>,
        author_id: &str,
        message: OutboundMessage,
        embeds: Vec<DiscordEmbed>,
        attachments: Vec<DiscordAttachment>,
        components: Vec<ActionRow>,
        thread_id: Option<&str>,
    ) -> Result<MessageId> {
        let msg_id = self
            .send_message(
                channel_id,
                guild_id,
                author_id,
                message,
                embeds,
                attachments,
                thread_id,
            )
            .await?;
        self.component_manager
            .attach_components(&msg_id, components)
            .await;
        Ok(msg_id)
    }

    /// Show a modal dialog to a user (in response to a component interaction).
    pub async fn show_modal(&self, user_id: &str, modal: Modal) {
        self.component_manager.show_modal(user_id, modal).await;
    }

    /// Push a component interaction (button click, select, modal submit).
    pub async fn push_component_interaction(&self, interaction: ComponentInteraction) {
        self.component_manager.push_interaction(interaction).await;
    }

    /// Pop the next component interaction.
    pub async fn pop_component_interaction(&self) -> Option<ComponentInteraction> {
        self.component_manager.pop_interaction().await
    }

    // -- Sub-manager accessors ---

    /// Access the component manager.
    pub fn components(&self) -> &ComponentManager {
        &self.component_manager
    }

    /// Access the guild manager.
    pub fn guilds(&self) -> &GuildManager {
        &self.guild_manager
    }

    /// Access the webhook manager.
    pub fn webhooks(&self) -> &WebhookManager {
        &self.webhook_manager
    }

    /// Access the voice state tracker.
    pub fn voice(&self) -> &VoiceStateTracker {
        &self.voice_tracker
    }

    /// Access the audit log store.
    pub fn audit(&self) -> &AuditLogStore {
        &self.audit_log
    }

    /// Access the channel manager.
    pub fn channel_mgmt(&self) -> &ChannelManager {
        &self.channel_manager
    }

    /// Access the scheduled event manager.
    pub fn events(&self) -> &ScheduledEventManager {
        &self.event_manager
    }

    // -- Guild channel / member caches --------------------------------------

    /// Caches the list of channels for a guild (for `#channel-name` → ID lookups).
    pub async fn cache_guild_channels(
        &self,
        guild_id: &str,
        channels: Vec<DiscordGuildChannel>,
    ) {
        self.guild_channels
            .write()
            .await
            .insert(guild_id.to_string(), channels);
    }

    /// Caches the list of members for a guild (for `@username` → ID lookups).
    pub async fn cache_guild_members(
        &self,
        guild_id: &str,
        members: Vec<DiscordGuildMember>,
    ) {
        self.guild_members
            .write()
            .await
            .insert(guild_id.to_string(), members);
    }

    /// Resolves a `#channel-name` to a channel ID within a guild.
    pub async fn resolve_channel_by_name(
        &self,
        guild_id: &str,
        name: &str,
    ) -> Option<String> {
        let name = name.strip_prefix('#').unwrap_or(name);
        let channels = self.guild_channels.read().await;
        channels.get(guild_id).and_then(|list| {
            list.iter()
                .find(|c| c.name.eq_ignore_ascii_case(name))
                .map(|c| c.id.clone())
        })
    }

    /// Resolves an `@username` to a user ID within a guild.
    pub async fn resolve_user_by_name(
        &self,
        guild_id: &str,
        username: &str,
    ) -> Option<String> {
        let username = username.strip_prefix('@').unwrap_or(username);
        let members = self.guild_members.read().await;
        members.get(guild_id).and_then(|list| {
            list.iter()
                .find(|m| {
                    m.username.eq_ignore_ascii_case(username)
                        || m.nickname
                            .as_deref()
                            .is_some_and(|n| n.eq_ignore_ascii_case(username))
                })
                .map(|m| m.user_id.clone())
        })
    }

    /// Opens (or retrieves cached) DM channel with a user.
    pub async fn open_dm(&self, user_id: &str) -> Result<String> {
        // Return cached DM channel if we already have one.
        if let Some(channel_id) = self.dm_channels.read().await.get(user_id) {
            return Ok(channel_id.clone());
        }
        let channel_id = format!(
            "dm-{user_id}-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        );
        self.dm_channels
            .write()
            .await
            .insert(user_id.to_string(), channel_id.clone());
        Ok(channel_id)
    }

    /// Sends a DM to a user (opens DM channel if needed, then sends).
    pub async fn send_dm(
        &self,
        user_id: &str,
        message: OutboundMessage,
    ) -> Result<MessageId> {
        let dm_channel = self.open_dm(user_id).await?;
        self.send_message(&dm_channel, None, "bot", message, Vec::new(), Vec::new(), None)
            .await
    }

    // -- Default slash commands registration --------------------------------

    /// Registers the default set of application slash commands.
    pub async fn register_default_slash_commands(&self) -> Result<()> {
        let commands = [
            ("status", "Show bot connection status and health"),
            ("help", "Show available commands and usage"),
            ("model", "Display or switch the current AI model"),
            ("compact", "Compact the conversation context"),
            ("sessions", "List active sessions across channels"),
        ];
        for (name, description) in commands {
            self.register_slash_command(
                name,
                serde_json::json!({
                    "name": name,
                    "description": description,
                    "type": 1
                }),
            )
            .await?;
        }
        Ok(())
    }

    // -- Gateway event handlers ---------------------------------------------

    /// Handles a `MESSAGE_UPDATE` gateway event (message edit).
    pub async fn on_message_update(
        &self,
        message_id: &str,
        new_content: &str,
    ) -> Result<()> {
        self.edit_message(message_id, new_content).await?;
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
                kind: "message_update".to_string(),
                channel_id: String::new(),
                guild_id: None,
                thread_id: None,
                session_scope: format!("discord:message:{message_id}"),
            });
        Ok(())
    }

    /// Handles a `MESSAGE_DELETE` gateway event.
    pub async fn on_message_delete(
        &self,
        message_id: &str,
        channel_id: &str,
    ) -> Result<()> {
        self.delete_message(message_id).await?;
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
                kind: "message_delete".to_string(),
                channel_id: channel_id.to_string(),
                guild_id: None,
                thread_id: None,
                session_scope: format!("discord:message:{message_id}"),
            });
        Ok(())
    }

    /// Handles a `GUILD_MEMBER_ADD` gateway event.
    pub async fn on_guild_member_add(
        &self,
        guild_id: &str,
        member: DiscordGuildMember,
    ) {
        let mut members = self.guild_members.write().await;
        members
            .entry(guild_id.to_string())
            .or_default()
            .push(member.clone());
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
                kind: "guild_member_add".to_string(),
                channel_id: String::new(),
                guild_id: Some(guild_id.to_string()),
                thread_id: None,
                session_scope: format!("discord:guild:{guild_id}:member:{}", member.user_id),
            });
    }

    /// Handles a `GUILD_MEMBER_REMOVE` gateway event.
    pub async fn on_guild_member_remove(&self, guild_id: &str, user_id: &str) {
        let mut members = self.guild_members.write().await;
        if let Some(list) = members.get_mut(guild_id) {
            list.retain(|m| m.user_id != user_id);
        }
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
                kind: "guild_member_remove".to_string(),
                channel_id: String::new(),
                guild_id: Some(guild_id.to_string()),
                thread_id: None,
                session_scope: format!("discord:guild:{guild_id}:member:{user_id}"),
            });
    }

    /// Handles a `MESSAGE_REACTION_ADD` gateway event.
    pub async fn on_reaction_add_event(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
        channel_id: &str,
    ) {
        self.add_reaction(message_id, emoji).await.ok();
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
                kind: "reaction_add".to_string(),
                channel_id: channel_id.to_string(),
                guild_id: None,
                thread_id: None,
                session_scope: format!("discord:reaction:{message_id}:{user_id}:{emoji}"),
            });
    }

    /// Handles a `MESSAGE_REACTION_REMOVE` gateway event.
    pub async fn on_reaction_remove_event(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
        channel_id: &str,
    ) {
        self.remove_reaction(message_id, emoji).await.ok();
        self.processed_events
            .lock()
            .await
            .push(DiscordProcessedEvent {
                kind: "reaction_remove".to_string(),
                channel_id: channel_id.to_string(),
                guild_id: None,
                thread_id: None,
                session_scope: format!("discord:reaction:{message_id}:{user_id}:{emoji}"),
            });
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

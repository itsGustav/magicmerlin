//! Discord webhook operations: create, execute, edit, delete.
//!
//! Supports both incoming webhooks and bot-created webhooks with avatar/name overrides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::DiscordEmbed;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A Discord webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub token: Option<String>,
    pub kind: WebhookKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookKind {
    Incoming,
    ChannelFollower,
    Application,
}

/// A message sent via webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookMessage {
    pub id: String,
    pub webhook_id: String,
    pub content: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub embeds: Vec<DiscordEmbed>,
    pub thread_id: Option<String>,
}

/// Parameters for executing a webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookExecuteParams {
    pub content: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub embeds: Vec<DiscordEmbed>,
    pub thread_id: Option<String>,
    pub tts: bool,
}

impl WebhookExecuteParams {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            username: None,
            avatar_url: None,
            embeds: Vec::new(),
            thread_id: None,
            tts: false,
        }
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn with_avatar(mut self, url: impl Into<String>) -> Self {
        self.avatar_url = Some(url.into());
        self
    }

    pub fn with_embeds(mut self, embeds: Vec<DiscordEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    pub fn in_thread(mut self, thread_id: impl Into<String>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Webhook manager
// ---------------------------------------------------------------------------

/// Manages webhook state and message tracking.
#[derive(Debug)]
pub struct WebhookManager {
    webhooks: Arc<RwLock<HashMap<String, Webhook>>>,
    messages: Arc<RwLock<Vec<WebhookMessage>>>,
    next_id: AtomicU64,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(Vec::new())),
            next_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> String {
        format!("wh-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Create a new webhook in a channel.
    pub async fn create(
        &self,
        channel_id: &str,
        guild_id: Option<&str>,
        name: &str,
    ) -> Webhook {
        let webhook = Webhook {
            id: self.next_id(),
            channel_id: channel_id.to_string(),
            guild_id: guild_id.map(ToString::to_string),
            name: Some(name.to_string()),
            avatar: None,
            token: Some(format!("whtoken-{}", self.next_id.load(Ordering::Relaxed))),
            kind: WebhookKind::Incoming,
        };
        self.webhooks
            .write()
            .await
            .insert(webhook.id.clone(), webhook.clone());
        webhook
    }

    /// Get a webhook by ID.
    pub async fn get(&self, webhook_id: &str) -> Option<Webhook> {
        self.webhooks.read().await.get(webhook_id).cloned()
    }

    /// List all webhooks for a channel.
    pub async fn list_for_channel(&self, channel_id: &str) -> Vec<Webhook> {
        self.webhooks
            .read()
            .await
            .values()
            .filter(|wh| wh.channel_id == channel_id)
            .cloned()
            .collect()
    }

    /// List all webhooks for a guild.
    pub async fn list_for_guild(&self, guild_id: &str) -> Vec<Webhook> {
        self.webhooks
            .read()
            .await
            .values()
            .filter(|wh| wh.guild_id.as_deref() == Some(guild_id))
            .cloned()
            .collect()
    }

    /// Modify a webhook's name or avatar.
    pub async fn modify(
        &self,
        webhook_id: &str,
        name: Option<String>,
        avatar: Option<String>,
    ) -> Option<Webhook> {
        let mut webhooks = self.webhooks.write().await;
        let wh = webhooks.get_mut(webhook_id)?;
        if let Some(name) = name {
            wh.name = Some(name);
        }
        if let Some(avatar) = avatar {
            wh.avatar = Some(avatar);
        }
        Some(wh.clone())
    }

    /// Delete a webhook.
    pub async fn delete(&self, webhook_id: &str) -> bool {
        self.webhooks.write().await.remove(webhook_id).is_some()
    }

    /// Execute a webhook (send a message).
    pub async fn execute(
        &self,
        webhook_id: &str,
        params: WebhookExecuteParams,
    ) -> Option<WebhookMessage> {
        let webhook = self.webhooks.read().await.get(webhook_id)?.clone();
        let msg = WebhookMessage {
            id: format!("whmsg-{}", self.next_id.fetch_add(1, Ordering::Relaxed)),
            webhook_id: webhook.id,
            content: params.content.unwrap_or_default(),
            username: params.username.or(webhook.name),
            avatar_url: params.avatar_url.or(webhook.avatar),
            embeds: params.embeds,
            thread_id: params.thread_id,
        };
        self.messages.write().await.push(msg.clone());
        Some(msg)
    }

    /// Edit a previously sent webhook message.
    pub async fn edit_message(&self, message_id: &str, content: &str) -> bool {
        let mut messages = self.messages.write().await;
        if let Some(msg) = messages.iter_mut().find(|m| m.id == message_id) {
            msg.content = content.to_string();
            true
        } else {
            false
        }
    }

    /// Delete a previously sent webhook message.
    pub async fn delete_message(&self, message_id: &str) -> bool {
        let mut messages = self.messages.write().await;
        let len_before = messages.len();
        messages.retain(|m| m.id != message_id);
        messages.len() < len_before
    }

    /// Get all messages sent by webhooks.
    pub async fn messages(&self) -> Vec<WebhookMessage> {
        self.messages.read().await.clone()
    }

    /// Get messages sent by a specific webhook.
    pub async fn messages_for(&self, webhook_id: &str) -> Vec<WebhookMessage> {
        self.messages
            .read()
            .await
            .iter()
            .filter(|m| m.webhook_id == webhook_id)
            .cloned()
            .collect()
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}

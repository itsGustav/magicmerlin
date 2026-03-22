//! Discord channel management: create, edit, delete channels, categories, and permission overwrites.
//!
//! Mirrors the Discord channel CRUD API with support for text, voice, category,
//! announcement, stage, and forum channel types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::guild::PermissionOverwrite;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Discord channel type matching API v10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelType {
    GuildText = 0,
    Dm = 1,
    GuildVoice = 2,
    GroupDm = 3,
    GuildCategory = 4,
    GuildAnnouncement = 5,
    AnnouncementThread = 10,
    PublicThread = 11,
    PrivateThread = 12,
    GuildStageVoice = 13,
    GuildDirectory = 14,
    GuildForum = 15,
    GuildMedia = 16,
}

/// A managed Discord channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedChannel {
    pub id: String,
    pub guild_id: Option<String>,
    pub name: String,
    pub kind: ChannelType,
    pub position: i32,
    pub topic: Option<String>,
    pub nsfw: bool,
    pub parent_id: Option<String>,
    pub rate_limit_per_user: Option<u32>,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub bitrate: Option<u32>,
    pub user_limit: Option<u32>,
}

/// Parameters for creating a channel.
#[derive(Debug, Clone)]
pub struct CreateChannelParams {
    pub name: String,
    pub kind: ChannelType,
    pub topic: Option<String>,
    pub parent_id: Option<String>,
    pub nsfw: bool,
    pub position: Option<i32>,
    pub rate_limit_per_user: Option<u32>,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub bitrate: Option<u32>,
    pub user_limit: Option<u32>,
}

impl CreateChannelParams {
    pub fn text(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelType::GuildText,
            topic: None,
            parent_id: None,
            nsfw: false,
            position: None,
            rate_limit_per_user: None,
            permission_overwrites: Vec::new(),
            bitrate: None,
            user_limit: None,
        }
    }

    pub fn voice(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelType::GuildVoice,
            topic: None,
            parent_id: None,
            nsfw: false,
            position: None,
            rate_limit_per_user: None,
            permission_overwrites: Vec::new(),
            bitrate: Some(64000),
            user_limit: None,
        }
    }

    pub fn category(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelType::GuildCategory,
            topic: None,
            parent_id: None,
            nsfw: false,
            position: None,
            rate_limit_per_user: None,
            permission_overwrites: Vec::new(),
            bitrate: None,
            user_limit: None,
        }
    }

    pub fn announcement(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelType::GuildAnnouncement,
            topic: None,
            parent_id: None,
            nsfw: false,
            position: None,
            rate_limit_per_user: None,
            permission_overwrites: Vec::new(),
            bitrate: None,
            user_limit: None,
        }
    }

    pub fn stage(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelType::GuildStageVoice,
            topic: None,
            parent_id: None,
            nsfw: false,
            position: None,
            rate_limit_per_user: None,
            permission_overwrites: Vec::new(),
            bitrate: Some(64000),
            user_limit: None,
        }
    }

    pub fn forum(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelType::GuildForum,
            topic: None,
            parent_id: None,
            nsfw: false,
            position: None,
            rate_limit_per_user: None,
            permission_overwrites: Vec::new(),
            bitrate: None,
            user_limit: None,
        }
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn in_category(mut self, category_id: impl Into<String>) -> Self {
        self.parent_id = Some(category_id.into());
        self
    }

    pub fn with_position(mut self, position: i32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_slowmode(mut self, seconds: u32) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }

    pub fn with_overwrites(mut self, overwrites: Vec<PermissionOverwrite>) -> Self {
        self.permission_overwrites = overwrites;
        self
    }

    pub fn with_user_limit(mut self, limit: u32) -> Self {
        self.user_limit = Some(limit);
        self
    }
}

/// Parameters for modifying a channel.
#[derive(Debug, Clone, Default)]
pub struct ModifyChannelParams {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub position: Option<i32>,
    pub nsfw: Option<bool>,
    pub parent_id: Option<Option<String>>,
    pub rate_limit_per_user: Option<u32>,
    pub bitrate: Option<u32>,
    pub user_limit: Option<u32>,
}

impl ModifyChannelParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn position(mut self, position: i32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = Some(nsfw);
        self
    }

    pub fn move_to_category(mut self, category_id: impl Into<String>) -> Self {
        self.parent_id = Some(Some(category_id.into()));
        self
    }

    pub fn remove_from_category(mut self) -> Self {
        self.parent_id = Some(None);
        self
    }

    pub fn slowmode(mut self, seconds: u32) -> Self {
        self.rate_limit_per_user = Some(seconds);
        self
    }
}

// ---------------------------------------------------------------------------
// Channel manager
// ---------------------------------------------------------------------------

/// Manages Discord channels within guilds.
#[derive(Debug)]
pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<String, ManagedChannel>>>,
    next_id: AtomicU64,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> String {
        format!("ch-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Create a channel in a guild.
    pub async fn create(&self, guild_id: &str, params: CreateChannelParams) -> ManagedChannel {
        let channel = ManagedChannel {
            id: self.next_id(),
            guild_id: Some(guild_id.to_string()),
            name: params.name,
            kind: params.kind,
            position: params.position.unwrap_or(0),
            topic: params.topic,
            nsfw: params.nsfw,
            parent_id: params.parent_id,
            rate_limit_per_user: params.rate_limit_per_user,
            permission_overwrites: params.permission_overwrites,
            bitrate: params.bitrate,
            user_limit: params.user_limit,
        };
        self.channels
            .write()
            .await
            .insert(channel.id.clone(), channel.clone());
        channel
    }

    /// Get a channel by ID.
    pub async fn get(&self, channel_id: &str) -> Option<ManagedChannel> {
        self.channels.read().await.get(channel_id).cloned()
    }

    /// List all channels in a guild.
    pub async fn list_guild_channels(&self, guild_id: &str) -> Vec<ManagedChannel> {
        let mut channels: Vec<_> = self
            .channels
            .read()
            .await
            .values()
            .filter(|ch| ch.guild_id.as_deref() == Some(guild_id))
            .cloned()
            .collect();
        channels.sort_by_key(|ch| ch.position);
        channels
    }

    /// List channels in a specific category.
    pub async fn list_category_children(&self, category_id: &str) -> Vec<ManagedChannel> {
        let mut channels: Vec<_> = self
            .channels
            .read()
            .await
            .values()
            .filter(|ch| ch.parent_id.as_deref() == Some(category_id))
            .cloned()
            .collect();
        channels.sort_by_key(|ch| ch.position);
        channels
    }

    /// List channels of a specific type in a guild.
    pub async fn list_by_type(
        &self,
        guild_id: &str,
        kind: ChannelType,
    ) -> Vec<ManagedChannel> {
        self.channels
            .read()
            .await
            .values()
            .filter(|ch| ch.guild_id.as_deref() == Some(guild_id) && ch.kind == kind)
            .cloned()
            .collect()
    }

    /// Modify a channel.
    pub async fn modify(
        &self,
        channel_id: &str,
        params: ModifyChannelParams,
    ) -> Option<ManagedChannel> {
        let mut channels = self.channels.write().await;
        let ch = channels.get_mut(channel_id)?;

        if let Some(name) = params.name {
            ch.name = name;
        }
        if let Some(topic) = params.topic {
            ch.topic = Some(topic);
        }
        if let Some(position) = params.position {
            ch.position = position;
        }
        if let Some(nsfw) = params.nsfw {
            ch.nsfw = nsfw;
        }
        if let Some(parent_id) = params.parent_id {
            ch.parent_id = parent_id;
        }
        if let Some(rate) = params.rate_limit_per_user {
            ch.rate_limit_per_user = Some(rate);
        }
        if let Some(bitrate) = params.bitrate {
            ch.bitrate = Some(bitrate);
        }
        if let Some(limit) = params.user_limit {
            ch.user_limit = Some(limit);
        }

        Some(ch.clone())
    }

    /// Delete a channel.
    pub async fn delete(&self, channel_id: &str) -> bool {
        self.channels.write().await.remove(channel_id).is_some()
    }

    /// Set permission overwrites for a channel.
    pub async fn set_overwrites(
        &self,
        channel_id: &str,
        overwrites: Vec<PermissionOverwrite>,
    ) -> bool {
        let mut channels = self.channels.write().await;
        if let Some(ch) = channels.get_mut(channel_id) {
            ch.permission_overwrites = overwrites;
            true
        } else {
            false
        }
    }

    /// Delete all channels in a guild (e.g. on GUILD_DELETE).
    pub async fn clear_guild(&self, guild_id: &str) {
        self.channels
            .write()
            .await
            .retain(|_, ch| ch.guild_id.as_deref() != Some(guild_id));
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

//! Discord voice state tracking.
//!
//! Tracks which users are in which voice channels, with mute/deaf/self-mute/self-deaf
//! and streaming/video state. Does NOT implement actual voice (opus/RTP) — just the
//! state machine that a bot uses to know who's where.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Voice state for a single user in a guild.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceState {
    pub user_id: String,
    pub guild_id: String,
    pub channel_id: Option<String>,
    pub session_id: String,
    pub deaf: bool,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    pub self_stream: bool,
    pub self_video: bool,
    pub suppress: bool,
}

impl VoiceState {
    /// Whether the user is currently connected to a voice channel.
    pub fn is_connected(&self) -> bool {
        self.channel_id.is_some()
    }

    /// Whether the user can be heard (not muted or deafened by anyone).
    pub fn is_audible(&self) -> bool {
        self.channel_id.is_some() && !self.mute && !self.self_mute && !self.suppress
    }
}

/// Summary of a voice channel's occupants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceChannelInfo {
    pub channel_id: String,
    pub guild_id: String,
    pub user_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Voice state tracker
// ---------------------------------------------------------------------------

/// Tracks voice state across all guilds.
#[derive(Debug)]
pub struct VoiceStateTracker {
    /// guild_id -> user_id -> VoiceState
    states: Arc<RwLock<HashMap<String, HashMap<String, VoiceState>>>>,
}

impl VoiceStateTracker {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update voice state for a user (called on VOICE_STATE_UPDATE dispatch).
    pub async fn update(&self, state: VoiceState) {
        let mut all = self.states.write().await;
        let guild = all.entry(state.guild_id.clone()).or_default();

        if state.channel_id.is_none() {
            // User disconnected from voice.
            guild.remove(&state.user_id);
        } else {
            guild.insert(state.user_id.clone(), state);
        }
    }

    /// Get voice state for a specific user in a guild.
    pub async fn get(&self, guild_id: &str, user_id: &str) -> Option<VoiceState> {
        self.states
            .read()
            .await
            .get(guild_id)?
            .get(user_id)
            .cloned()
    }

    /// List all users in a specific voice channel.
    pub async fn users_in_channel(&self, guild_id: &str, channel_id: &str) -> Vec<VoiceState> {
        self.states
            .read()
            .await
            .get(guild_id)
            .map(|guild| {
                guild
                    .values()
                    .filter(|s| s.channel_id.as_deref() == Some(channel_id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a summary of all active voice channels in a guild.
    pub async fn active_channels(&self, guild_id: &str) -> Vec<VoiceChannelInfo> {
        let states = self.states.read().await;
        let guild = match states.get(guild_id) {
            Some(g) => g,
            None => return Vec::new(),
        };

        let mut channels: HashMap<String, Vec<String>> = HashMap::new();
        for state in guild.values() {
            if let Some(channel_id) = &state.channel_id {
                channels
                    .entry(channel_id.clone())
                    .or_default()
                    .push(state.user_id.clone());
            }
        }

        channels
            .into_iter()
            .map(|(channel_id, user_ids)| VoiceChannelInfo {
                channel_id,
                guild_id: guild_id.to_string(),
                user_ids,
            })
            .collect()
    }

    /// Get all voice states for a guild.
    pub async fn guild_states(&self, guild_id: &str) -> Vec<VoiceState> {
        self.states
            .read()
            .await
            .get(guild_id)
            .map(|guild| guild.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove all voice state for a guild (e.g. on GUILD_DELETE).
    pub async fn clear_guild(&self, guild_id: &str) {
        self.states.write().await.remove(guild_id);
    }

    /// Count total users in voice across all guilds.
    pub async fn total_voice_users(&self) -> usize {
        self.states
            .read()
            .await
            .values()
            .map(|guild| guild.len())
            .sum()
    }
}

impl Default for VoiceStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

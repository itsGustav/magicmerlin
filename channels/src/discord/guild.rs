//! Discord guild management: guild info, members, roles, bans, and permission bitfields.
//!
//! Mirrors Discord's permission system with the standard bitfield layout (v10 API).
//! Provides in-memory guild state tracking for bot-side permission computation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Permission bitfield constants (Discord API v10)
// ---------------------------------------------------------------------------

/// Discord permission bitflags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions(pub u64);

impl Permissions {
    pub const CREATE_INSTANT_INVITE: u64 = 1 << 0;
    pub const KICK_MEMBERS: u64 = 1 << 1;
    pub const BAN_MEMBERS: u64 = 1 << 2;
    pub const ADMINISTRATOR: u64 = 1 << 3;
    pub const MANAGE_CHANNELS: u64 = 1 << 4;
    pub const MANAGE_GUILD: u64 = 1 << 5;
    pub const ADD_REACTIONS: u64 = 1 << 6;
    pub const VIEW_AUDIT_LOG: u64 = 1 << 7;
    pub const PRIORITY_SPEAKER: u64 = 1 << 8;
    pub const STREAM: u64 = 1 << 9;
    pub const VIEW_CHANNEL: u64 = 1 << 10;
    pub const SEND_MESSAGES: u64 = 1 << 11;
    pub const SEND_TTS_MESSAGES: u64 = 1 << 12;
    pub const MANAGE_MESSAGES: u64 = 1 << 13;
    pub const EMBED_LINKS: u64 = 1 << 14;
    pub const ATTACH_FILES: u64 = 1 << 15;
    pub const READ_MESSAGE_HISTORY: u64 = 1 << 16;
    pub const MENTION_EVERYONE: u64 = 1 << 17;
    pub const USE_EXTERNAL_EMOJIS: u64 = 1 << 18;
    pub const VIEW_GUILD_INSIGHTS: u64 = 1 << 19;
    pub const CONNECT: u64 = 1 << 20;
    pub const SPEAK: u64 = 1 << 21;
    pub const MUTE_MEMBERS: u64 = 1 << 22;
    pub const DEAFEN_MEMBERS: u64 = 1 << 23;
    pub const MOVE_MEMBERS: u64 = 1 << 24;
    pub const USE_VAD: u64 = 1 << 25;
    pub const CHANGE_NICKNAME: u64 = 1 << 26;
    pub const MANAGE_NICKNAMES: u64 = 1 << 27;
    pub const MANAGE_ROLES: u64 = 1 << 28;
    pub const MANAGE_WEBHOOKS: u64 = 1 << 29;
    pub const MANAGE_GUILD_EXPRESSIONS: u64 = 1 << 30;
    pub const USE_APPLICATION_COMMANDS: u64 = 1 << 31;
    pub const REQUEST_TO_SPEAK: u64 = 1 << 32;
    pub const MANAGE_EVENTS: u64 = 1 << 33;
    pub const MANAGE_THREADS: u64 = 1 << 34;
    pub const CREATE_PUBLIC_THREADS: u64 = 1 << 35;
    pub const CREATE_PRIVATE_THREADS: u64 = 1 << 36;
    pub const USE_EXTERNAL_STICKERS: u64 = 1 << 37;
    pub const SEND_MESSAGES_IN_THREADS: u64 = 1 << 38;
    pub const USE_EMBEDDED_ACTIVITIES: u64 = 1 << 39;
    pub const MODERATE_MEMBERS: u64 = 1 << 40;

    pub const NONE: Permissions = Permissions(0);
    pub const ALL: Permissions = Permissions(u64::MAX);

    pub fn has(self, flag: u64) -> bool {
        self.0 & flag == flag
    }

    pub fn is_admin(self) -> bool {
        self.has(Self::ADMINISTRATOR)
    }

    pub fn add(self, flag: u64) -> Self {
        Self(self.0 | flag)
    }

    pub fn remove(self, flag: u64) -> Self {
        Self(self.0 & !flag)
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::NONE
    }
}

// ---------------------------------------------------------------------------
// Permission overwrite (channel-level)
// ---------------------------------------------------------------------------

/// Channel-level permission overwrite targeting a role or member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionOverwrite {
    pub target_id: String,
    pub target_kind: OverwriteKind,
    pub allow: Permissions,
    pub deny: Permissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverwriteKind {
    Role,
    Member,
}

// ---------------------------------------------------------------------------
// Core guild types
// ---------------------------------------------------------------------------

/// A Discord role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub position: i32,
    pub permissions: Permissions,
    pub mentionable: bool,
    pub hoist: bool,
    pub managed: bool,
}

/// A guild member (user + guild-specific data).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildMember {
    pub user_id: String,
    pub nickname: Option<String>,
    pub role_ids: Vec<String>,
    pub joined_at: String,
    pub deaf: bool,
    pub mute: bool,
}

/// Guild-level ban entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ban {
    pub user_id: String,
    pub reason: Option<String>,
}

/// Summary info about a guild.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub member_count: u64,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub features: Vec<String>,
}

// ---------------------------------------------------------------------------
// Guild manager
// ---------------------------------------------------------------------------

/// Manages guild state: info, members, roles, bans, and computed permissions.
#[derive(Debug)]
pub struct GuildManager {
    guilds: Arc<RwLock<HashMap<String, GuildInfo>>>,
    members: Arc<RwLock<HashMap<String, Vec<GuildMember>>>>,
    roles: Arc<RwLock<HashMap<String, Vec<Role>>>>,
    bans: Arc<RwLock<HashMap<String, Vec<Ban>>>>,
    channel_overwrites: Arc<RwLock<HashMap<String, Vec<PermissionOverwrite>>>>,
}

impl GuildManager {
    pub fn new() -> Self {
        Self {
            guilds: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            bans: Arc::new(RwLock::new(HashMap::new())),
            channel_overwrites: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // -- Guild info ---

    pub async fn upsert_guild(&self, info: GuildInfo) {
        self.guilds.write().await.insert(info.id.clone(), info);
    }

    pub async fn guild(&self, guild_id: &str) -> Option<GuildInfo> {
        self.guilds.read().await.get(guild_id).cloned()
    }

    pub async fn all_guilds(&self) -> Vec<GuildInfo> {
        self.guilds.read().await.values().cloned().collect()
    }

    pub async fn remove_guild(&self, guild_id: &str) {
        self.guilds.write().await.remove(guild_id);
        self.members.write().await.remove(guild_id);
        self.roles.write().await.remove(guild_id);
        self.bans.write().await.remove(guild_id);
    }

    // -- Members ---

    pub async fn add_member(&self, guild_id: &str, member: GuildMember) {
        self.members
            .write()
            .await
            .entry(guild_id.to_string())
            .or_default()
            .push(member);
    }

    pub async fn remove_member(&self, guild_id: &str, user_id: &str) {
        if let Some(members) = self.members.write().await.get_mut(guild_id) {
            members.retain(|m| m.user_id != user_id);
        }
    }

    pub async fn member(&self, guild_id: &str, user_id: &str) -> Option<GuildMember> {
        self.members
            .read()
            .await
            .get(guild_id)?
            .iter()
            .find(|m| m.user_id == user_id)
            .cloned()
    }

    pub async fn members(&self, guild_id: &str) -> Vec<GuildMember> {
        self.members
            .read()
            .await
            .get(guild_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn update_nickname(&self, guild_id: &str, user_id: &str, nickname: Option<String>) {
        if let Some(members) = self.members.write().await.get_mut(guild_id) {
            if let Some(m) = members.iter_mut().find(|m| m.user_id == user_id) {
                m.nickname = nickname;
            }
        }
    }

    pub async fn add_role_to_member(&self, guild_id: &str, user_id: &str, role_id: &str) {
        if let Some(members) = self.members.write().await.get_mut(guild_id) {
            if let Some(m) = members.iter_mut().find(|m| m.user_id == user_id) {
                if !m.role_ids.contains(&role_id.to_string()) {
                    m.role_ids.push(role_id.to_string());
                }
            }
        }
    }

    pub async fn remove_role_from_member(&self, guild_id: &str, user_id: &str, role_id: &str) {
        if let Some(members) = self.members.write().await.get_mut(guild_id) {
            if let Some(m) = members.iter_mut().find(|m| m.user_id == user_id) {
                m.role_ids.retain(|r| r != role_id);
            }
        }
    }

    // -- Roles ---

    pub async fn add_role(&self, guild_id: &str, role: Role) {
        self.roles
            .write()
            .await
            .entry(guild_id.to_string())
            .or_default()
            .push(role);
    }

    pub async fn remove_role(&self, guild_id: &str, role_id: &str) {
        if let Some(roles) = self.roles.write().await.get_mut(guild_id) {
            roles.retain(|r| r.id != role_id);
        }
    }

    pub async fn role(&self, guild_id: &str, role_id: &str) -> Option<Role> {
        self.roles
            .read()
            .await
            .get(guild_id)?
            .iter()
            .find(|r| r.id == role_id)
            .cloned()
    }

    pub async fn roles(&self, guild_id: &str) -> Vec<Role> {
        self.roles
            .read()
            .await
            .get(guild_id)
            .cloned()
            .unwrap_or_default()
    }

    // -- Bans ---

    pub async fn ban(&self, guild_id: &str, user_id: &str, reason: Option<String>) {
        self.remove_member(guild_id, user_id).await;
        self.bans
            .write()
            .await
            .entry(guild_id.to_string())
            .or_default()
            .push(Ban {
                user_id: user_id.to_string(),
                reason,
            });
    }

    pub async fn unban(&self, guild_id: &str, user_id: &str) {
        if let Some(bans) = self.bans.write().await.get_mut(guild_id) {
            bans.retain(|b| b.user_id != user_id);
        }
    }

    pub async fn bans(&self, guild_id: &str) -> Vec<Ban> {
        self.bans
            .read()
            .await
            .get(guild_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn is_banned(&self, guild_id: &str, user_id: &str) -> bool {
        self.bans
            .read()
            .await
            .get(guild_id)
            .map(|bans| bans.iter().any(|b| b.user_id == user_id))
            .unwrap_or(false)
    }

    pub async fn kick(&self, guild_id: &str, user_id: &str) {
        self.remove_member(guild_id, user_id).await;
    }

    // -- Channel permission overwrites ---

    pub async fn set_channel_overwrites(
        &self,
        channel_id: &str,
        overwrites: Vec<PermissionOverwrite>,
    ) {
        self.channel_overwrites
            .write()
            .await
            .insert(channel_id.to_string(), overwrites);
    }

    pub async fn channel_overwrites(&self, channel_id: &str) -> Vec<PermissionOverwrite> {
        self.channel_overwrites
            .read()
            .await
            .get(channel_id)
            .cloned()
            .unwrap_or_default()
    }

    // -- Permission computation ---

    /// Compute effective permissions for a member in a guild (base, without channel overwrites).
    pub async fn compute_base_permissions(&self, guild_id: &str, user_id: &str) -> Permissions {
        let guild = match self.guild(guild_id).await {
            Some(g) => g,
            None => return Permissions::NONE,
        };

        // Owner has all permissions.
        if guild.owner_id == user_id {
            return Permissions::ALL;
        }

        let member = match self.member(guild_id, user_id).await {
            Some(m) => m,
            None => return Permissions::NONE,
        };

        let roles = self.roles(guild_id).await;

        // Start with @everyone role permissions (role id == guild id).
        let mut perms = roles
            .iter()
            .find(|r| r.id == guild_id)
            .map(|r| r.permissions)
            .unwrap_or(Permissions::NONE);

        // Union all member role permissions.
        for role_id in &member.role_ids {
            if let Some(role) = roles.iter().find(|r| &r.id == role_id) {
                perms = perms.union(role.permissions);
            }
        }

        // Administrator bypasses everything.
        if perms.is_admin() {
            return Permissions::ALL;
        }

        perms
    }

    /// Compute effective permissions for a member in a specific channel (with overwrites).
    pub async fn compute_channel_permissions(
        &self,
        guild_id: &str,
        channel_id: &str,
        user_id: &str,
    ) -> Permissions {
        let mut perms = self.compute_base_permissions(guild_id, user_id).await;

        if perms.is_admin() {
            return Permissions::ALL;
        }

        let overwrites = self.channel_overwrites(channel_id).await;
        let member = match self.member(guild_id, user_id).await {
            Some(m) => m,
            None => return perms,
        };

        // Apply @everyone role overwrite first.
        for ow in &overwrites {
            if ow.target_kind == OverwriteKind::Role && ow.target_id == guild_id {
                perms = perms.remove(ow.deny.0);
                perms = perms.add(ow.allow.0);
            }
        }

        // Apply role overwrites (union all allow, union all deny).
        let mut role_allow = Permissions::NONE;
        let mut role_deny = Permissions::NONE;
        for ow in &overwrites {
            if ow.target_kind == OverwriteKind::Role && member.role_ids.contains(&ow.target_id) {
                role_allow = role_allow.union(ow.allow);
                role_deny = role_deny.union(ow.deny);
            }
        }
        perms = perms.remove(role_deny.0);
        perms = perms.add(role_allow.0);

        // Apply member-specific overwrite last (highest priority).
        for ow in &overwrites {
            if ow.target_kind == OverwriteKind::Member && ow.target_id == user_id {
                perms = perms.remove(ow.deny.0);
                perms = perms.add(ow.allow.0);
            }
        }

        perms
    }
}

impl Default for GuildManager {
    fn default() -> Self {
        Self::new()
    }
}

//! Discord audit log queries.
//!
//! Tracks moderation and administrative actions within a guild. Mirrors the
//! Discord audit log entry structure with action types, targets, and changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Discord audit log action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditLogAction {
    GuildUpdate = 1,
    ChannelCreate = 10,
    ChannelUpdate = 11,
    ChannelDelete = 12,
    ChannelOverwriteCreate = 13,
    ChannelOverwriteUpdate = 14,
    ChannelOverwriteDelete = 15,
    MemberKick = 20,
    MemberPrune = 21,
    MemberBanAdd = 22,
    MemberBanRemove = 23,
    MemberUpdate = 24,
    MemberRoleUpdate = 25,
    MemberMove = 26,
    MemberDisconnect = 27,
    BotAdd = 28,
    RoleCreate = 30,
    RoleUpdate = 31,
    RoleDelete = 32,
    InviteCreate = 40,
    InviteUpdate = 41,
    InviteDelete = 42,
    WebhookCreate = 50,
    WebhookUpdate = 51,
    WebhookDelete = 52,
    EmojiCreate = 60,
    EmojiUpdate = 61,
    EmojiDelete = 62,
    MessageDelete = 72,
    MessageBulkDelete = 73,
    MessagePin = 74,
    MessageUnpin = 75,
    IntegrationCreate = 80,
    IntegrationUpdate = 81,
    IntegrationDelete = 82,
    ThreadCreate = 110,
    ThreadUpdate = 111,
    ThreadDelete = 112,
    AutoModerationRuleCreate = 140,
    AutoModerationRuleUpdate = 141,
    AutoModerationRuleDelete = 142,
    AutoModerationBlockMessage = 143,
}

/// A single change within an audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogChange {
    pub key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

/// An audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub guild_id: String,
    pub action: AuditLogAction,
    pub user_id: Option<String>,
    pub target_id: Option<String>,
    pub reason: Option<String>,
    pub changes: Vec<AuditLogChange>,
    pub timestamp: String,
}

/// Query filter for audit log retrieval.
#[derive(Debug, Clone, Default)]
pub struct AuditLogQuery {
    pub action_type: Option<AuditLogAction>,
    pub user_id: Option<String>,
    pub before: Option<String>,
    pub limit: Option<usize>,
}

impl AuditLogQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action(mut self, action: AuditLogAction) -> Self {
        self.action_type = Some(action);
        self
    }

    pub fn by_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn before_entry(mut self, entry_id: impl Into<String>) -> Self {
        self.before = Some(entry_id.into());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

// ---------------------------------------------------------------------------
// Audit log store
// ---------------------------------------------------------------------------

/// In-memory audit log store for guild events.
#[derive(Debug)]
pub struct AuditLogStore {
    entries: Arc<RwLock<HashMap<String, Vec<AuditLogEntry>>>>,
    next_id: AtomicU64,
}

impl AuditLogStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1),
        }
    }

    /// Record an audit log entry.
    pub async fn record(
        &self,
        guild_id: &str,
        action: AuditLogAction,
        user_id: Option<&str>,
        target_id: Option<&str>,
        reason: Option<&str>,
        changes: Vec<AuditLogChange>,
    ) -> AuditLogEntry {
        let entry = AuditLogEntry {
            id: format!("audit-{}", self.next_id.fetch_add(1, Ordering::Relaxed)),
            guild_id: guild_id.to_string(),
            action,
            user_id: user_id.map(ToString::to_string),
            target_id: target_id.map(ToString::to_string),
            reason: reason.map(ToString::to_string),
            changes,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.entries
            .write()
            .await
            .entry(guild_id.to_string())
            .or_default()
            .push(entry.clone());
        entry
    }

    /// Query audit log entries for a guild with optional filters.
    pub async fn query(&self, guild_id: &str, query: AuditLogQuery) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let guild_entries = match entries.get(guild_id) {
            Some(e) => e,
            None => return Vec::new(),
        };

        let mut results: Vec<_> = guild_entries
            .iter()
            .filter(|e| {
                if let Some(action) = query.action_type {
                    if e.action != action {
                        return false;
                    }
                }
                if let Some(ref user_id) = query.user_id {
                    if e.user_id.as_deref() != Some(user_id.as_str()) {
                        return false;
                    }
                }
                if let Some(ref before) = query.before {
                    if e.id >= *before {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Return newest first.
        results.reverse();

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get all entries for a guild (newest first).
    pub async fn all(&self, guild_id: &str) -> Vec<AuditLogEntry> {
        let mut entries = self
            .entries
            .read()
            .await
            .get(guild_id)
            .cloned()
            .unwrap_or_default();
        entries.reverse();
        entries
    }

    /// Count entries by action type for a guild.
    pub async fn count_by_action(&self, guild_id: &str) -> HashMap<AuditLogAction, usize> {
        let entries = self.entries.read().await;
        let guild_entries = match entries.get(guild_id) {
            Some(e) => e,
            None => return HashMap::new(),
        };

        let mut counts = HashMap::new();
        for entry in guild_entries {
            *counts.entry(entry.action).or_insert(0) += 1;
        }
        counts
    }

    /// Clear audit log for a guild.
    pub async fn clear(&self, guild_id: &str) {
        self.entries.write().await.remove(guild_id);
    }
}

impl Default for AuditLogStore {
    fn default() -> Self {
        Self::new()
    }
}

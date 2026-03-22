//! Discord guild scheduled events.
//!
//! Create, modify, cancel, and track scheduled events (voice, stage, external).
//! Maps to Discord's Guild Scheduled Events API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Entity type for a scheduled event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledEventEntityType {
    StageInstance = 1,
    Voice = 2,
    External = 3,
}

/// Status of a scheduled event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledEventStatus {
    Scheduled = 1,
    Active = 2,
    Completed = 3,
    Cancelled = 4,
}

/// A guild scheduled event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: String,
    pub guild_id: String,
    pub channel_id: Option<String>,
    pub creator_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub scheduled_start_time: String,
    pub scheduled_end_time: Option<String>,
    pub entity_type: ScheduledEventEntityType,
    pub status: ScheduledEventStatus,
    pub entity_metadata: Option<EventEntityMetadata>,
    pub user_count: u64,
    pub image: Option<String>,
}

/// Extra metadata for external events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEntityMetadata {
    pub location: Option<String>,
}

/// Parameters for creating a scheduled event.
#[derive(Debug, Clone)]
pub struct CreateEventParams {
    pub name: String,
    pub entity_type: ScheduledEventEntityType,
    pub scheduled_start_time: String,
    pub scheduled_end_time: Option<String>,
    pub description: Option<String>,
    pub channel_id: Option<String>,
    pub location: Option<String>,
    pub image: Option<String>,
}

impl CreateEventParams {
    pub fn voice(
        name: impl Into<String>,
        channel_id: impl Into<String>,
        start: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            entity_type: ScheduledEventEntityType::Voice,
            scheduled_start_time: start.into(),
            scheduled_end_time: None,
            description: None,
            channel_id: Some(channel_id.into()),
            location: None,
            image: None,
        }
    }

    pub fn stage(
        name: impl Into<String>,
        channel_id: impl Into<String>,
        start: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            entity_type: ScheduledEventEntityType::StageInstance,
            scheduled_start_time: start.into(),
            scheduled_end_time: None,
            description: None,
            channel_id: Some(channel_id.into()),
            location: None,
            image: None,
        }
    }

    pub fn external(
        name: impl Into<String>,
        location: impl Into<String>,
        start: impl Into<String>,
        end: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            entity_type: ScheduledEventEntityType::External,
            scheduled_start_time: start.into(),
            scheduled_end_time: Some(end.into()),
            description: None,
            channel_id: None,
            location: Some(location.into()),
            image: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_end_time(mut self, end: impl Into<String>) -> Self {
        self.scheduled_end_time = Some(end.into());
        self
    }

    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }
}

/// An RSVP / interested user for an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventUser {
    pub user_id: String,
    pub event_id: String,
}

// ---------------------------------------------------------------------------
// Event manager
// ---------------------------------------------------------------------------

/// Manages guild scheduled events.
#[derive(Debug)]
pub struct ScheduledEventManager {
    events: Arc<RwLock<HashMap<String, ScheduledEvent>>>,
    interested: Arc<RwLock<HashMap<String, Vec<String>>>>,
    next_id: AtomicU64,
}

impl ScheduledEventManager {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            interested: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> String {
        format!("evt-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Create a scheduled event.
    pub async fn create(
        &self,
        guild_id: &str,
        creator_id: Option<&str>,
        params: CreateEventParams,
    ) -> ScheduledEvent {
        let event = ScheduledEvent {
            id: self.next_id(),
            guild_id: guild_id.to_string(),
            channel_id: params.channel_id,
            creator_id: creator_id.map(ToString::to_string),
            name: params.name,
            description: params.description,
            scheduled_start_time: params.scheduled_start_time,
            scheduled_end_time: params.scheduled_end_time,
            entity_type: params.entity_type,
            status: ScheduledEventStatus::Scheduled,
            entity_metadata: params.location.map(|loc| EventEntityMetadata {
                location: Some(loc),
            }),
            user_count: 0,
            image: params.image,
        };
        self.events
            .write()
            .await
            .insert(event.id.clone(), event.clone());
        event
    }

    /// Get an event by ID.
    pub async fn get(&self, event_id: &str) -> Option<ScheduledEvent> {
        self.events.read().await.get(event_id).cloned()
    }

    /// List all events for a guild.
    pub async fn list_guild_events(&self, guild_id: &str) -> Vec<ScheduledEvent> {
        self.events
            .read()
            .await
            .values()
            .filter(|e| e.guild_id == guild_id)
            .cloned()
            .collect()
    }

    /// Start an event (move from Scheduled to Active).
    pub async fn start(&self, event_id: &str) -> Option<ScheduledEvent> {
        let mut events = self.events.write().await;
        let event = events.get_mut(event_id)?;
        if event.status == ScheduledEventStatus::Scheduled {
            event.status = ScheduledEventStatus::Active;
        }
        Some(event.clone())
    }

    /// Complete an event.
    pub async fn complete(&self, event_id: &str) -> Option<ScheduledEvent> {
        let mut events = self.events.write().await;
        let event = events.get_mut(event_id)?;
        if event.status == ScheduledEventStatus::Active {
            event.status = ScheduledEventStatus::Completed;
        }
        Some(event.clone())
    }

    /// Cancel an event.
    pub async fn cancel(&self, event_id: &str) -> Option<ScheduledEvent> {
        let mut events = self.events.write().await;
        let event = events.get_mut(event_id)?;
        if event.status == ScheduledEventStatus::Scheduled {
            event.status = ScheduledEventStatus::Cancelled;
        }
        Some(event.clone())
    }

    /// Modify an event's name or description.
    pub async fn modify(
        &self,
        event_id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Option<ScheduledEvent> {
        let mut events = self.events.write().await;
        let event = events.get_mut(event_id)?;
        if let Some(name) = name {
            event.name = name;
        }
        if let Some(desc) = description {
            event.description = Some(desc);
        }
        Some(event.clone())
    }

    /// Delete an event.
    pub async fn delete(&self, event_id: &str) -> bool {
        let removed = self.events.write().await.remove(event_id).is_some();
        if removed {
            self.interested.write().await.remove(event_id);
        }
        removed
    }

    /// Mark a user as interested in an event.
    pub async fn add_interested(&self, event_id: &str, user_id: &str) {
        let mut interested = self.interested.write().await;
        let users = interested.entry(event_id.to_string()).or_default();
        if !users.contains(&user_id.to_string()) {
            users.push(user_id.to_string());
            // Update user count.
            drop(interested);
            if let Some(event) = self.events.write().await.get_mut(event_id) {
                event.user_count += 1;
            }
        }
    }

    /// Remove a user's interest in an event.
    pub async fn remove_interested(&self, event_id: &str, user_id: &str) {
        let mut interested = self.interested.write().await;
        if let Some(users) = interested.get_mut(event_id) {
            let len_before = users.len();
            users.retain(|u| u != user_id);
            if users.len() < len_before {
                drop(interested);
                if let Some(event) = self.events.write().await.get_mut(event_id) {
                    event.user_count = event.user_count.saturating_sub(1);
                }
            }
        }
    }

    /// Get interested users for an event.
    pub async fn interested_users(&self, event_id: &str) -> Vec<String> {
        self.interested
            .read()
            .await
            .get(event_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for ScheduledEventManager {
    fn default() -> Self {
        Self::new()
    }
}

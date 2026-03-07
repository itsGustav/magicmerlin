use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use super::Platform;

/// Current connection state for a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Channel is healthy and connected.
    Connected,
    /// Channel is currently disconnected.
    Disconnected,
    /// Channel is reconnecting.
    Reconnecting,
}

/// Health details for a single platform channel.
#[derive(Debug, Clone)]
pub struct ChannelHealth {
    /// Platform identifier.
    pub platform: Platform,
    /// Current status.
    pub state: ConnectionState,
    /// Last error string if any.
    pub last_error: Option<String>,
    /// Last transition timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Shared channel health monitor.
#[derive(Debug, Default)]
pub struct HealthMonitor {
    statuses: RwLock<HashMap<Platform, ChannelHealth>>,
}

impl HealthMonitor {
    /// Creates an empty health monitor.
    pub fn new() -> Self {
        Self {
            statuses: RwLock::new(HashMap::new()),
        }
    }

    /// Marks a channel as connected.
    pub async fn mark_connected(&self, platform: Platform) {
        self.set(platform, ConnectionState::Connected, None).await;
    }

    /// Marks a channel as disconnected with an optional error.
    pub async fn mark_disconnected(&self, platform: Platform, error: Option<String>) {
        self.set(platform, ConnectionState::Disconnected, error).await;
    }

    /// Marks a channel as reconnecting.
    pub async fn mark_reconnecting(&self, platform: Platform) {
        self.set(platform, ConnectionState::Reconnecting, None).await;
    }

    /// Returns health details for a platform.
    pub async fn get(&self, platform: Platform) -> Option<ChannelHealth> {
        self.statuses.read().await.get(&platform).cloned()
    }

    /// Returns health snapshot for all channels.
    pub async fn snapshot(&self) -> Vec<ChannelHealth> {
        self.statuses.read().await.values().cloned().collect()
    }

    async fn set(&self, platform: Platform, state: ConnectionState, last_error: Option<String>) {
        let mut lock = self.statuses.write().await;
        lock.insert(
            platform,
            ChannelHealth {
                platform,
                state,
                last_error,
                updated_at: Utc::now(),
            },
        );
    }
}

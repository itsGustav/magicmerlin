use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{
    format_for_platform, split_for_platform, AutoReplyBridge, HealthMonitor, MessageId,
    OutboundMessage, Platform, Result,
};
use crate::framework::types::ChannelError;

/// Unified channel interface implemented by each platform adapter.
#[async_trait]
pub trait Channel: Send + Sync {
    /// Human-readable channel name.
    fn name(&self) -> &str;
    /// Channel platform.
    fn platform(&self) -> Platform;
    /// Starts listening for inbound events.
    async fn start(&mut self) -> Result<()>;
    /// Stops listening for inbound events.
    async fn stop(&mut self) -> Result<()>;
    /// Sends a new message to a target chat.
    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId>;
    /// Edits an existing message.
    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()>;
    /// Deletes an existing message.
    async fn delete(&self, target: &str, message_id: &str) -> Result<()>;
    /// Adds a reaction to an existing message.
    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()>;
}

/// Runtime channel registry for platform routing.
#[derive(Default)]
pub struct ChannelRegistry {
    channels: HashMap<Platform, Arc<RwLock<Box<dyn Channel>>>>,
    /// Shared health monitor.
    pub health: Arc<HealthMonitor>,
    /// Optional bridge to auto-reply pipeline.
    pub auto_reply: Option<AutoReplyBridge>,
}

impl ChannelRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            health: Arc::new(HealthMonitor::new()),
            auto_reply: None,
        }
    }

    /// Attaches auto-reply bridge.
    pub fn with_auto_reply(mut self, bridge: AutoReplyBridge) -> Self {
        self.auto_reply = Some(bridge);
        self
    }

    /// Registers a channel implementation.
    pub fn register(&mut self, channel: Box<dyn Channel>) {
        self.channels
            .insert(channel.platform(), Arc::new(RwLock::new(channel)));
    }

    /// Starts all registered channels.
    pub async fn start_all(&self) -> Result<()> {
        for (platform, channel) in &self.channels {
            self.health.mark_reconnecting(*platform).await;
            let mut lock = channel.write().await;
            match lock.start().await {
                Ok(()) => self.health.mark_connected(*platform).await,
                Err(error) => {
                    self.health
                        .mark_disconnected(*platform, Some(error.to_string()))
                        .await;
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Attempts reconnect for disconnected channels.
    pub async fn reconnect_disconnected(&self) -> Result<()> {
        for (platform, channel) in &self.channels {
            let is_disconnected = self
                .health
                .get(*platform)
                .await
                .map(|health| health.state == super::ConnectionState::Disconnected)
                .unwrap_or(true);

            if !is_disconnected {
                continue;
            }

            self.health.mark_reconnecting(*platform).await;
            let mut lock = channel.write().await;
            match lock.start().await {
                Ok(()) => self.health.mark_connected(*platform).await,
                Err(error) => {
                    self.health
                        .mark_disconnected(*platform, Some(error.to_string()))
                        .await;
                }
            }
        }
        Ok(())
    }

    /// Stops all registered channels.
    pub async fn stop_all(&self) -> Result<()> {
        for (platform, channel) in &self.channels {
            let mut lock = channel.write().await;
            lock.stop().await?;
            self.health.mark_disconnected(*platform, None).await;
        }
        Ok(())
    }

    /// Sends outbound message to a platform using automatic split + formatting.
    pub async fn send(
        &self,
        platform: Platform,
        target: &str,
        message: OutboundMessage,
    ) -> Result<Vec<MessageId>> {
        let channel = self
            .channels
            .get(&platform)
            .ok_or(ChannelError::ChannelNotRegistered(platform))?
            .clone();

        let chunks = split_for_platform(platform, &message);
        let mut ids = Vec::with_capacity(chunks.len().max(1));

        if chunks.is_empty() {
            return Ok(ids);
        }

        let read = channel.read().await;
        for chunk in chunks {
            let mut part = message.clone();
            part.text = chunk;
            part.parse_mode = None;
            part.text = format_for_platform(platform, &part);
            ids.push(read.send(target, part).await?);
        }

        Ok(ids)
    }

    /// Returns true if a channel exists for a platform.
    pub fn has_platform(&self, platform: Platform) -> bool {
        self.channels.contains_key(&platform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::{
        ChatType, DmPolicy, DmPolicyEnforcer, InboundMessage, MentionGate, ParseMode, Sender,
    };
    use chrono::Utc;
    use serde_json::json;

    #[derive(Debug)]
    struct MockChannel;

    #[async_trait]
    impl Channel for MockChannel {
        fn name(&self) -> &str {
            "mock"
        }

        fn platform(&self) -> Platform {
            Platform::Telegram
        }

        async fn start(&mut self) -> Result<()> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<()> {
            Ok(())
        }

        async fn send(&self, _target: &str, message: OutboundMessage) -> Result<MessageId> {
            Ok(format!("id:{}", message.text.len()))
        }

        async fn edit(
            &self,
            _target: &str,
            _message_id: &str,
            _message: OutboundMessage,
        ) -> Result<()> {
            Ok(())
        }

        async fn delete(&self, _target: &str, _message_id: &str) -> Result<()> {
            Ok(())
        }

        async fn react(&self, _target: &str, _message_id: &str, _emoji: &str) -> Result<()> {
            Ok(())
        }
    }

    fn sample_message(text: Option<&str>, chat_type: ChatType) -> InboundMessage {
        InboundMessage {
            id: "m1".to_string(),
            platform: Platform::Telegram,
            chat_id: "c1".to_string(),
            chat_type,
            sender: Sender {
                id: "u1".to_string(),
                name: "User".to_string(),
                username: Some("user".to_string()),
            },
            text: text.map(ToOwned::to_owned),
            reply_to: None,
            media: Vec::new(),
            timestamp: Utc::now(),
            raw: json!({"text": text}),
        }
    }

    #[test]
    fn normalizes_inbound_text() {
        let mut inbound = sample_message(Some("  hello world  "), ChatType::Direct);
        inbound.normalize();
        assert_eq!(inbound.text.as_deref(), Some("hello world"));

        let mut empty = sample_message(Some("   \n\t"), ChatType::Direct);
        empty.normalize();
        assert_eq!(empty.text, None);
    }

    #[test]
    fn enforces_dm_policy_modes() {
        let direct = sample_message(Some("hi"), ChatType::Direct);

        let open = DmPolicyEnforcer::new(DmPolicy::Open);
        assert!(open.allows(&direct));

        let pairing = DmPolicyEnforcer::new(DmPolicy::Pairing);
        assert!(!pairing.allows(&direct));

        let mut pairing_ok = DmPolicyEnforcer::new(DmPolicy::Pairing);
        pairing_ok.approve_pairing("u1");
        assert!(pairing_ok.allows(&direct));

        let mut allowlist = DmPolicyEnforcer::new(DmPolicy::Allowlist);
        assert!(!allowlist.allows(&direct));
        allowlist.allow_user("u1");
        assert!(allowlist.allows(&direct));
    }

    #[test]
    fn mention_gate_blocks_unmentioned_group_messages() {
        let gate = MentionGate::new("merlin", true);
        let group = sample_message(Some("hello everyone"), ChatType::Group);
        assert!(!gate.should_process(&group));

        let mentioned = sample_message(Some("@merlin hello"), ChatType::Group);
        assert!(gate.should_process(&mentioned));
    }

    #[test]
    fn applies_platform_formatting() {
        let message = OutboundMessage {
            text: "*Bold* [link]".to_string(),
            reply_to: None,
            media: Vec::new(),
            buttons: None,
            silent: false,
            parse_mode: Some(ParseMode::Markdown),
        };

        let telegram = format_for_platform(Platform::Telegram, &message);
        assert!(telegram.contains("\\*Bold\\*"));

        let discord = format_for_platform(Platform::Discord, &message);
        assert_eq!(discord, "*Bold* [link]");
    }

    #[test]
    fn splits_long_messages_by_platform_limit() {
        let long = "word ".repeat(1200);
        let message = OutboundMessage {
            text: long,
            reply_to: None,
            media: Vec::new(),
            buttons: None,
            silent: false,
            parse_mode: None,
        };

        let chunks = split_for_platform(Platform::Discord, &message);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 2000));
    }

    #[tokio::test]
    async fn registry_routes_messages() {
        let mut registry = ChannelRegistry::new();
        registry.register(Box::new(MockChannel));

        let outbound = OutboundMessage {
            text: "hello".to_string(),
            reply_to: None,
            media: Vec::new(),
            buttons: None,
            silent: false,
            parse_mode: None,
        };

        let ids = registry
            .send(Platform::Telegram, "chat", outbound)
            .await
            .expect("send should succeed");
        assert_eq!(ids.len(), 1);
    }
}

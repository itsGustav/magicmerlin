//! Auto-reply pipeline with policy gating, slash commands, debounce collection, and formatting.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use magicmerlin_sessions::ResolutionContext;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for auto-reply operations.
pub type Result<T> = std::result::Result<T, AutoReplyError>;

/// Error type for auto-reply operations.
#[derive(Debug, Error)]
pub enum AutoReplyError {
    /// Returned for invalid model command payload.
    #[error("invalid /model command")]
    InvalidModelCommand,
}

/// Supported inbound platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Telegram markdown formatting with 4096-char limit.
    Telegram,
    /// Discord markdown formatting with 2000-char limit.
    Discord,
    /// WhatsApp plain text formatting.
    WhatsApp,
}

/// DM policy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmPolicy {
    /// Anyone can DM and receive replies.
    Open,
    /// DM requires pairing approval.
    Pairing,
    /// DM restricted to explicit allowlist.
    Allowlist,
}

/// Auto-reply runtime configuration.
#[derive(Debug, Clone)]
pub struct AutoReplyConfig {
    /// Message collect/debounce window.
    pub debounce_window: Duration,
    /// DM policy behavior.
    pub dm_policy: DmPolicy,
    /// In group chats, only respond when mentioned.
    pub mention_required_in_groups: bool,
    /// User allowlist used for `DmPolicy::Allowlist`.
    pub allowlist_users: HashSet<String>,
}

impl Default for AutoReplyConfig {
    fn default() -> Self {
        Self {
            debounce_window: Duration::from_secs(2),
            dm_policy: DmPolicy::Open,
            mention_required_in_groups: true,
            allowlist_users: HashSet::new(),
        }
    }
}

/// Normalized inbound message for the auto-reply pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundMessage {
    /// Platform/channel name.
    pub channel: String,
    /// Optional chat id for group contexts.
    pub chat_id: Option<String>,
    /// User id of sender.
    pub user_id: String,
    /// Plain text message body.
    pub text: String,
    /// Whether the message came from a direct message context.
    pub is_dm: bool,
    /// Whether the agent was explicitly mentioned.
    pub mentioned: bool,
    /// Message priority score; higher means more urgent.
    pub priority: u8,
}

/// Parsed slash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    /// `/status`
    Status,
    /// `/compact`
    Compact,
    /// `/reasoning on|off`
    Reasoning(Option<bool>),
    /// `/model` show or `/model <name>` set.
    Model(Option<String>),
    /// `/reset`
    Reset,
    /// `/help`
    Help,
}

/// Result of inbound policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineDecision {
    /// Message should be processed and queued for agent turn.
    Queue {
        /// Resolved session key.
        session_key: String,
    },
    /// Message should be ignored.
    Ignore,
    /// Slash command detected and should be handled locally.
    Command(SlashCommand),
}

/// Delivery context bound to a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryContext {
    /// Owning session key.
    pub session_key: String,
    /// Source channel name.
    pub channel: String,
    /// Source chat id.
    pub chat_id: Option<String>,
    /// Optional announce-mode target channel.
    pub announce_channel: Option<String>,
}

/// Debounced batch ready to run through the agent.
#[derive(Debug, Clone)]
pub struct CollectedBatch {
    /// Session key for the batch.
    pub session_key: String,
    /// Collected messages.
    pub messages: Vec<InboundMessage>,
}

#[derive(Debug, Clone)]
struct CollectState {
    messages: Vec<InboundMessage>,
    deadline: Instant,
    max_priority: u8,
}

/// Result of pushing a message into the collect buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectPushResult {
    /// Whether a pending turn should be canceled due to higher-priority message.
    pub cancel_pending_turn: bool,
}

/// Stateful debounce collector keyed by session key.
#[derive(Debug)]
pub struct DebounceCollector {
    window: Duration,
    states: HashMap<String, CollectState>,
}

impl DebounceCollector {
    /// Creates a new debounce collector with fixed time window.
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            states: HashMap::new(),
        }
    }

    /// Adds a message to a session collect buffer and returns push behavior.
    pub fn push(
        &mut self,
        now: Instant,
        session_key: &str,
        message: InboundMessage,
    ) -> CollectPushResult {
        let state = self
            .states
            .entry(session_key.to_string())
            .or_insert_with(|| CollectState {
                max_priority: message.priority,
                messages: Vec::new(),
                deadline: now + self.window,
            });

        let cancel_pending_turn = message.priority > state.max_priority;
        if cancel_pending_turn {
            state.max_priority = message.priority;
        }
        state.messages.push(message);
        state.deadline = now + self.window;
        CollectPushResult {
            cancel_pending_turn,
        }
    }

    /// Drains all session batches whose debounce deadline has elapsed.
    pub fn due_batches(&mut self, now: Instant) -> Vec<CollectedBatch> {
        let mut ready = Vec::new();
        let keys = self
            .states
            .iter()
            .filter_map(|(k, v)| {
                if v.deadline <= now {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for key in keys {
            if let Some(state) = self.states.remove(&key) {
                ready.push(CollectedBatch {
                    session_key: key,
                    messages: state.messages,
                });
            }
        }
        ready
    }
}

/// Stateful auto-reply engine.
#[derive(Debug)]
pub struct AutoReplyEngine {
    config: AutoReplyConfig,
    delivery: HashMap<String, DeliveryContext>,
}

impl AutoReplyEngine {
    /// Creates a new auto-reply engine from config.
    pub fn new(config: AutoReplyConfig) -> Self {
        Self {
            config,
            delivery: HashMap::new(),
        }
    }

    /// Evaluates inbound message and returns queue/ignore/command decision.
    pub fn evaluate_inbound(&mut self, inbound: &InboundMessage) -> PipelineDecision {
        if let Some(command) = parse_slash_command(&inbound.text) {
            return PipelineDecision::Command(command);
        }

        if inbound.is_dm && !self.dm_allowed(inbound) {
            return PipelineDecision::Ignore;
        }

        if !inbound.is_dm && self.config.mention_required_in_groups && !inbound.mentioned {
            return PipelineDecision::Ignore;
        }

        let session_key = magicmerlin_sessions::resolve_session_key(&ResolutionContext {
            channel: inbound.channel.clone(),
            agent_name: Some("merlin".to_string()),
            chat_id: inbound.chat_id.clone(),
            user_id: Some(inbound.user_id.clone()),
            slash_command: false,
            custom_pattern: None,
        });

        self.delivery.insert(
            session_key.clone(),
            DeliveryContext {
                session_key: session_key.clone(),
                channel: inbound.channel.clone(),
                chat_id: inbound.chat_id.clone(),
                announce_channel: None,
            },
        );

        PipelineDecision::Queue { session_key }
    }

    /// Returns delivery context for a known session.
    pub fn delivery_context(&self, session_key: &str) -> Option<&DeliveryContext> {
        self.delivery.get(session_key)
    }

    /// Enables announce mode for a session.
    pub fn set_announce_channel(&mut self, session_key: &str, channel: Option<String>) {
        if let Some(ctx) = self.delivery.get_mut(session_key) {
            ctx.announce_channel = channel;
        }
    }

    fn dm_allowed(&self, inbound: &InboundMessage) -> bool {
        match self.config.dm_policy {
            DmPolicy::Open => true,
            DmPolicy::Pairing => inbound.mentioned,
            DmPolicy::Allowlist => self.config.allowlist_users.contains(&inbound.user_id),
        }
    }
}

/// Parses supported slash commands.
pub fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    let text = input.trim();
    if !text.starts_with('/') {
        return None;
    }

    let mut parts = text.split_whitespace();
    let cmd = parts.next()?;
    match cmd {
        "/status" => Some(SlashCommand::Status),
        "/compact" => Some(SlashCommand::Compact),
        "/reasoning" => {
            let arg = parts.next();
            let value = match arg {
                Some("on") => Some(true),
                Some("off") => Some(false),
                Some(_) => None,
                None => None,
            };
            Some(SlashCommand::Reasoning(value))
        }
        "/model" => {
            let rest = parts.collect::<Vec<_>>().join(" ");
            if rest.is_empty() {
                Some(SlashCommand::Model(None))
            } else {
                Some(SlashCommand::Model(Some(rest)))
            }
        }
        "/reset" => Some(SlashCommand::Reset),
        "/help" => Some(SlashCommand::Help),
        _ => None,
    }
}

/// Formats and splits an outbound reply for a target platform.
pub fn format_reply(platform: Platform, input: &str) -> Vec<String> {
    let text = input.trim();
    if text.is_empty() || text == "NO_REPLY" || text == "HEARTBEAT_OK" {
        return Vec::new();
    }

    let limit = match platform {
        Platform::Telegram => 4096,
        Platform::Discord => 2000,
        Platform::WhatsApp => 4096,
    };

    split_by_limit(text, limit)
}

fn split_by_limit(text: &str, limit: usize) -> Vec<String> {
    if text.len() <= limit {
        return vec![text.to_string()];
    }

    let mut out = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        if current.len() + 1 + word.len() > limit {
            out.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_slash_commands() {
        assert_eq!(parse_slash_command("/status"), Some(SlashCommand::Status));
        assert_eq!(parse_slash_command("/compact"), Some(SlashCommand::Compact));
        assert_eq!(
            parse_slash_command("/reasoning on"),
            Some(SlashCommand::Reasoning(Some(true)))
        );
        assert_eq!(
            parse_slash_command("/model gpt-5"),
            Some(SlashCommand::Model(Some("gpt-5".to_string())))
        );
        assert_eq!(parse_slash_command("/reset"), Some(SlashCommand::Reset));
        assert_eq!(parse_slash_command("/help"), Some(SlashCommand::Help));
    }

    #[test]
    fn collect_debounce_cancels_on_higher_priority() {
        let mut collector = DebounceCollector::new(Duration::from_secs(2));
        let now = Instant::now();

        let first = InboundMessage {
            channel: "telegram".to_string(),
            chat_id: Some("c1".to_string()),
            user_id: "u1".to_string(),
            text: "first".to_string(),
            is_dm: false,
            mentioned: true,
            priority: 1,
        };
        let second = InboundMessage {
            text: "urgent".to_string(),
            priority: 9,
            ..first.clone()
        };

        let first_result = collector.push(now, "telegram:c1", first);
        assert!(!first_result.cancel_pending_turn);
        let second_result = collector.push(now + Duration::from_millis(500), "telegram:c1", second);
        assert!(second_result.cancel_pending_turn);

        let due = collector.due_batches(now + Duration::from_secs(3));
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].messages.len(), 2);
    }

    #[test]
    fn format_suppresses_and_splits() {
        assert!(format_reply(Platform::Telegram, "NO_REPLY").is_empty());
        let long = "word ".repeat(900);
        let chunks = format_reply(Platform::Discord, &long);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 2000));
    }
}

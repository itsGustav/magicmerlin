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

/// Chat type for DM gate enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatType {
    /// Direct message.
    Direct,
    /// Group/channel chat.
    Group,
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

// ---------------------------------------------------------------------------
// Slash commands
// ---------------------------------------------------------------------------

/// Parsed slash command covering 30+ commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    // Status & info
    /// `/status`
    Status,
    /// `/version`
    Version,
    /// `/help [topic]`
    Help { topic: Option<String> },

    // Model control
    /// `/model [name]` — show or set model.
    Model { name: Option<String> },
    /// `/reasoning [on|off]`
    Reasoning { on: Option<bool> },
    /// `/thinking [level]`
    Thinking { level: Option<String> },
    /// `/verbose [on|off]`
    Verbose { on: Option<bool> },

    // Session management
    /// `/compact`
    Compact,
    /// `/session [key]`
    Session { key: Option<String> },
    /// `/sessions`
    Sessions,
    /// `/memory [query]`
    Memory { query: Option<String> },

    // Cron management
    /// `/cron [action] [args...]`
    Cron {
        action: Option<String>,
        args: Vec<String>,
    },

    // Admin
    /// `/approve <code> [allow-once|allow-always|deny]`
    Approve { code: String, mode: Option<String> },
    /// `/logs [tail]`
    Logs { tail: Option<u32> },
    /// `/debug`
    Debug,
    /// `/reset`
    Reset,
    /// `/config [key] [value]`
    Config {
        key: Option<String>,
        value: Option<String>,
    },

    // Channel-specific
    /// `/pause`
    Pause,
    /// `/resume`
    Resume,
    /// `/announce [channel]`
    Announce { channel: Option<String> },

    // Agent management
    /// `/agents`
    Agents,
    /// `/spawn <prompt>`
    Spawn { prompt: String },
    /// `/kill <session>`
    Kill { session: String },

    // Context & history
    /// `/context`
    Context,
    /// `/history [count]`
    History { count: Option<u32> },
    /// `/cost`
    Cost,
    /// `/whoami`
    Whoami,
    /// `/clear`
    Clear,
    /// `/subscribe [event]`
    Subscribe { event: Option<String> },
    /// `/unsubscribe [event]`
    Unsubscribe { event: Option<String> },
    /// `/feedback [text]`
    Feedback { text: Option<String> },

    // Misc
    /// `/ping`
    Ping,
    /// Internal: reply was NO_REPLY sentinel.
    NoReply,
    /// Internal: reply was HEARTBEAT_OK sentinel.
    HeartbeatOk,

    // Unknown
    /// Unknown slash command.
    Unknown { name: String, args: Vec<String> },
}

// ---------------------------------------------------------------------------
// Pipeline types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Debounce collector (existing)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Collect window (simple single-stream variant)
// ---------------------------------------------------------------------------

/// Simple collect/debounce window for batching inbound messages.
#[derive(Debug)]
pub struct CollectWindow {
    pending: Vec<InboundMessage>,
    deadline: Option<Instant>,
    window_ms: u64,
}

impl CollectWindow {
    /// Creates a new collect window with default 500ms window.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            deadline: None,
            window_ms: 500,
        }
    }

    /// Creates a new collect window with custom window duration in milliseconds.
    pub fn with_window_ms(window_ms: u64) -> Self {
        Self {
            pending: Vec::new(),
            deadline: None,
            window_ms,
        }
    }

    /// Add a message. Returns `Some(batch)` if the previous window expired, `None` if still collecting.
    pub fn add(&mut self, msg: InboundMessage) -> Option<Vec<InboundMessage>> {
        let now = Instant::now();

        // Check if existing window expired before adding the new message.
        if let Some(deadline) = self.deadline {
            if now >= deadline && !self.pending.is_empty() {
                let batch = std::mem::take(&mut self.pending);
                self.deadline = None;
                // Start new window with current message.
                self.pending.push(msg);
                self.deadline = Some(now + Duration::from_millis(self.window_ms));
                return Some(batch);
            }
        }

        self.pending.push(msg);
        self.deadline = Some(now + Duration::from_millis(self.window_ms));
        None
    }

    /// Check if the window expired. Returns `Some(batch)` if yes.
    pub fn check_expiry(&mut self) -> Option<Vec<InboundMessage>> {
        if let Some(deadline) = self.deadline {
            if Instant::now() >= deadline && !self.pending.is_empty() {
                self.deadline = None;
                return Some(std::mem::take(&mut self.pending));
            }
        }
        None
    }
}

impl Default for CollectWindow {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// DM gate
// ---------------------------------------------------------------------------

/// Authorized sender enforcement gate.
#[derive(Debug)]
pub struct DmGate {
    /// DM policy mode.
    pub policy: DmPolicy,
    /// User IDs or chat IDs in the allowlist.
    pub allowlist: HashSet<String>,
    /// User IDs paired via approval flow.
    pub paired: HashSet<String>,
}

impl DmGate {
    /// Creates a new DM gate with the given policy.
    pub fn new(policy: DmPolicy) -> Self {
        Self {
            policy,
            allowlist: HashSet::new(),
            paired: HashSet::new(),
        }
    }

    /// Returns true if the sender is allowed to trigger an agent turn.
    pub fn is_allowed(&self, sender_id: &str, chat_type: ChatType) -> bool {
        match chat_type {
            ChatType::Group => true,
            ChatType::Direct => match self.policy {
                DmPolicy::Open => true,
                DmPolicy::Allowlist => self.allowlist.contains(sender_id),
                DmPolicy::Pairing => {
                    self.paired.contains(sender_id) || self.allowlist.contains(sender_id)
                }
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Reply-to construction
// ---------------------------------------------------------------------------

/// Reference to a message for reply-to threading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyRef {
    /// Target message ID (`"current"` for the triggering message).
    pub message_id: String,
    /// Optional quoted text snippet.
    pub quote_text: Option<String>,
}

/// Extract `[[reply_to_current]]` or `[[reply_to:<id>]]` tag from reply text.
/// Returns `(cleaned_text, Option<ReplyRef>)`.
pub fn extract_reply_tag(text: &str) -> (String, Option<ReplyRef>) {
    let trimmed = text.trim_start();

    if let Some(rest) = trimmed.strip_prefix("[[reply_to_current]]") {
        return (
            rest.trim_start().to_string(),
            Some(ReplyRef {
                message_id: "current".to_string(),
                quote_text: None,
            }),
        );
    }

    if let Some(after_prefix) = trimmed.strip_prefix("[[reply_to:") {
        if let Some(id_end) = after_prefix.find("]]") {
            let id = after_prefix[..id_end].to_string();
            let rest = after_prefix[id_end + 2..].trim_start().to_string();
            return (
                rest,
                Some(ReplyRef {
                    message_id: id,
                    quote_text: None,
                }),
            );
        }
    }

    (text.to_string(), None)
}

// ---------------------------------------------------------------------------
// Auto-reply engine
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Slash command parser
// ---------------------------------------------------------------------------

/// Parses 30+ supported slash commands from raw message text.
pub fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    let text = input.trim();
    if !text.starts_with('/') {
        return None;
    }

    let mut parts = text.split_whitespace();
    let cmd = parts.next()?;
    let rest: Vec<&str> = parts.collect();

    match cmd {
        // Status & info
        "/status" => Some(SlashCommand::Status),
        "/version" => Some(SlashCommand::Version),
        "/help" => {
            let topic = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Help { topic })
        }

        // Model control
        "/model" => {
            let name = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Model { name })
        }
        "/reasoning" => {
            let on = match rest.first().copied() {
                Some("on") => Some(true),
                Some("off") => Some(false),
                _ => None,
            };
            Some(SlashCommand::Reasoning { on })
        }
        "/thinking" => {
            let level = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Thinking { level })
        }
        "/verbose" => {
            let on = match rest.first().copied() {
                Some("on") => Some(true),
                Some("off") => Some(false),
                _ => None,
            };
            Some(SlashCommand::Verbose { on })
        }

        // Session management
        "/compact" => Some(SlashCommand::Compact),
        "/session" => {
            let key = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Session { key })
        }
        "/sessions" => Some(SlashCommand::Sessions),
        "/memory" => {
            let query = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Memory { query })
        }

        // Cron management
        "/cron" => {
            let action = rest.first().map(|s| s.to_string());
            let args = rest.iter().skip(1).map(|s| s.to_string()).collect();
            Some(SlashCommand::Cron { action, args })
        }

        // Admin
        "/approve" => {
            let code = rest.first()?.to_string();
            let mode = rest.get(1).map(|s| s.to_string());
            Some(SlashCommand::Approve { code, mode })
        }
        "/logs" => {
            let tail = rest.first().and_then(|s| s.parse().ok());
            Some(SlashCommand::Logs { tail })
        }
        "/debug" => Some(SlashCommand::Debug),
        "/reset" => Some(SlashCommand::Reset),
        "/config" => {
            let key = rest.first().map(|s| s.to_string());
            let value = if rest.len() > 1 {
                Some(rest[1..].join(" "))
            } else {
                None
            };
            Some(SlashCommand::Config { key, value })
        }

        // Channel-specific
        "/pause" => Some(SlashCommand::Pause),
        "/resume" => Some(SlashCommand::Resume),
        "/announce" => {
            let channel = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Announce { channel })
        }

        // Agent management
        "/agents" => Some(SlashCommand::Agents),
        "/spawn" => {
            if rest.is_empty() {
                return None;
            }
            Some(SlashCommand::Spawn {
                prompt: rest.join(" "),
            })
        }
        "/kill" => {
            if rest.is_empty() {
                return None;
            }
            Some(SlashCommand::Kill {
                session: rest.join(" "),
            })
        }

        // Context & history
        "/context" => Some(SlashCommand::Context),
        "/history" => {
            let count = rest.first().and_then(|s| s.parse().ok());
            Some(SlashCommand::History { count })
        }
        "/cost" => Some(SlashCommand::Cost),
        "/whoami" => Some(SlashCommand::Whoami),
        "/clear" => Some(SlashCommand::Clear),
        "/subscribe" => {
            let event = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Subscribe { event })
        }
        "/unsubscribe" => {
            let event = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Unsubscribe { event })
        }
        "/feedback" => {
            let text = if rest.is_empty() {
                None
            } else {
                Some(rest.join(" "))
            };
            Some(SlashCommand::Feedback { text })
        }

        // Misc
        "/ping" => Some(SlashCommand::Ping),

        // Unknown — any /word pattern not matched above
        _ => {
            let name = cmd.trim_start_matches('/').to_string();
            if name.is_empty() {
                return None;
            }
            let args = rest.iter().map(|s| s.to_string()).collect();
            Some(SlashCommand::Unknown { name, args })
        }
    }
}

// ---------------------------------------------------------------------------
// Silent reply detection
// ---------------------------------------------------------------------------

/// Returns true if the text is a silent reply sentinel (`NO_REPLY` or `HEARTBEAT_OK`).
pub fn is_silent_reply(text: &str) -> bool {
    let t = text.trim();
    t == "NO_REPLY" || t == "HEARTBEAT_OK"
}

/// Returns true if the text is a `HEARTBEAT_OK` sentinel.
pub fn is_heartbeat_ok(text: &str) -> bool {
    text.trim() == "HEARTBEAT_OK"
}

// ---------------------------------------------------------------------------
// Platform formatting
// ---------------------------------------------------------------------------

/// Characters that must be escaped in Telegram MarkdownV2 plain text.
const TELEGRAM_SPECIAL: &[char] = &[
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
];

/// Escape all special chars for Telegram MarkdownV2.
///
/// Preserves valid formatting: `*bold*`, `_italic_`, `` `code` ``, ```` ```block``` ````,
/// `[text](url)`, `~strike~`. All other special characters in plain text segments are escaped.
pub fn escape_telegram_v2(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len + len / 4);
    let mut i = 0;

    while i < len {
        // Code block: ```...```
        if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            out.push_str("```");
            i += 3;
            while i < len {
                if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
                    out.push_str("```");
                    i += 3;
                    break;
                }
                out.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Inline code: `...`
        if chars[i] == '`' {
            out.push('`');
            i += 1;
            while i < len && chars[i] != '`' {
                out.push(chars[i]);
                i += 1;
            }
            if i < len {
                out.push('`');
                i += 1;
            }
            continue;
        }

        // Bold: *...*
        if chars[i] == '*' {
            if let Some(end) = find_closing_marker(&chars, i + 1, '*') {
                out.push('*');
                for ch in chars.iter().take(end).skip(i + 1) {
                    push_escaped(&mut out, *ch);
                }
                out.push('*');
                i = end + 1;
                continue;
            }
        }

        // Italic: _..._
        if chars[i] == '_' {
            if let Some(end) = find_closing_marker(&chars, i + 1, '_') {
                out.push('_');
                for ch in chars.iter().take(end).skip(i + 1) {
                    push_escaped(&mut out, *ch);
                }
                out.push('_');
                i = end + 1;
                continue;
            }
        }

        // Strikethrough: ~...~
        if chars[i] == '~' {
            if let Some(end) = find_closing_marker(&chars, i + 1, '~') {
                out.push('~');
                for ch in chars.iter().take(end).skip(i + 1) {
                    push_escaped(&mut out, *ch);
                }
                out.push('~');
                i = end + 1;
                continue;
            }
        }

        // Link: [text](url)
        if chars[i] == '[' {
            if let Some((text_end, url_end)) = find_link_span(&chars, i) {
                out.push('[');
                for ch in chars.iter().take(text_end).skip(i + 1) {
                    push_escaped(&mut out, *ch);
                }
                out.push_str("](");
                // URLs are not escaped
                for ch in chars.iter().take(url_end).skip(text_end + 2) {
                    out.push(*ch);
                }
                out.push(')');
                i = url_end + 1;
                continue;
            }
        }

        // Plain text: escape special chars
        push_escaped(&mut out, chars[i]);
        i += 1;
    }

    out
}

/// Convert standard Markdown to Telegram MarkdownV2 format.
///
/// Converts `**bold**` to `*bold*`, `*italic*` to `_italic_`, `~~strike~~` to `~strike~`,
/// and `# Header` to `*Header*`. Code blocks and inline code are passed through.
/// Special characters in plain text are escaped.
pub fn format_for_telegram(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len + len / 4);
    let mut i = 0;
    let mut line_start = true;

    while i < len {
        // Code block: ```...``` — pass through without conversion
        if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            out.push_str("```");
            i += 3;
            while i < len {
                if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
                    out.push_str("```");
                    i += 3;
                    break;
                }
                out.push(chars[i]);
                i += 1;
            }
            line_start = false;
            continue;
        }

        // Inline code — pass through
        if chars[i] == '`' {
            out.push('`');
            i += 1;
            while i < len && chars[i] != '`' {
                out.push(chars[i]);
                i += 1;
            }
            if i < len {
                out.push('`');
                i += 1;
            }
            line_start = false;
            continue;
        }

        // Headers at line start: # text → *text* (bold)
        if line_start && chars[i] == '#' {
            while i < len && chars[i] == '#' {
                i += 1;
            }
            if i < len && chars[i] == ' ' {
                i += 1;
            }
            out.push('*');
            while i < len && chars[i] != '\n' {
                push_escaped(&mut out, chars[i]);
                i += 1;
            }
            out.push('*');
            if i < len {
                out.push('\n');
                i += 1;
                line_start = true;
            }
            continue;
        }

        // Bold: **text** → *text*
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_marker(&chars, i + 2, '*') {
                out.push('*');
                for ch in chars.iter().take(end).skip(i + 2) {
                    push_escaped(&mut out, *ch);
                }
                out.push('*');
                i = end + 2;
                line_start = false;
                continue;
            }
        }

        // Italic: *text* → _text_ (single asterisks, not double)
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if let Some(end) = find_closing_marker(&chars, i + 1, '*') {
                out.push('_');
                for ch in chars.iter().take(end).skip(i + 1) {
                    push_escaped(&mut out, *ch);
                }
                out.push('_');
                i = end + 1;
                line_start = false;
                continue;
            }
        }

        // Strikethrough: ~~text~~ → ~text~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_double_marker(&chars, i + 2, '~') {
                out.push('~');
                for ch in chars.iter().take(end).skip(i + 2) {
                    push_escaped(&mut out, *ch);
                }
                out.push('~');
                i = end + 2;
                line_start = false;
                continue;
            }
        }

        // Link: [text](url) — preserve for Telegram
        if chars[i] == '[' {
            if let Some((text_end, url_end)) = find_link_span(&chars, i) {
                out.push('[');
                for ch in chars.iter().take(text_end).skip(i + 1) {
                    push_escaped(&mut out, *ch);
                }
                out.push_str("](");
                for ch in chars.iter().take(url_end).skip(text_end + 2) {
                    out.push(*ch);
                }
                out.push(')');
                i = url_end + 1;
                line_start = false;
                continue;
            }
        }

        // Newline tracking
        if chars[i] == '\n' {
            out.push('\n');
            i += 1;
            line_start = true;
            continue;
        }

        // Plain text: escape special chars
        push_escaped(&mut out, chars[i]);
        i += 1;
        line_start = false;
    }

    out
}

/// Convert standard Markdown to Discord format (mostly passthrough).
///
/// Discord uses standard Markdown, so this is largely a no-op.
pub fn format_for_discord(text: &str) -> String {
    text.to_string()
}

/// Convert standard Markdown to WhatsApp format.
///
/// WhatsApp supports: `*bold*`, `_italic_`, `~strike~`, `` `code` ``, no headers.
pub fn format_for_whatsapp(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;
    let mut line_start = true;

    while i < len {
        // Code block: ```...``` — keep as-is
        if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            out.push_str("```");
            i += 3;
            while i < len {
                if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
                    out.push_str("```");
                    i += 3;
                    break;
                }
                out.push(chars[i]);
                i += 1;
            }
            line_start = false;
            continue;
        }

        // Inline code — keep as-is
        if chars[i] == '`' {
            out.push('`');
            i += 1;
            while i < len && chars[i] != '`' {
                out.push(chars[i]);
                i += 1;
            }
            if i < len {
                out.push('`');
                i += 1;
            }
            line_start = false;
            continue;
        }

        // Headers → *header text* (bold)
        if line_start && chars[i] == '#' {
            while i < len && chars[i] == '#' {
                i += 1;
            }
            if i < len && chars[i] == ' ' {
                i += 1;
            }
            out.push('*');
            while i < len && chars[i] != '\n' {
                out.push(chars[i]);
                i += 1;
            }
            out.push('*');
            if i < len {
                out.push('\n');
                i += 1;
                line_start = true;
            }
            continue;
        }

        // Bold: **text** → *text*
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_marker(&chars, i + 2, '*') {
                out.push('*');
                for ch in chars.iter().take(end).skip(i + 2) {
                    out.push(*ch);
                }
                out.push('*');
                i = end + 2;
                line_start = false;
                continue;
            }
        }

        // Italic: *text* → _text_
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if let Some(end) = find_closing_marker(&chars, i + 1, '*') {
                out.push('_');
                for ch in chars.iter().take(end).skip(i + 1) {
                    out.push(*ch);
                }
                out.push('_');
                i = end + 1;
                line_start = false;
                continue;
            }
        }

        // Strikethrough: ~~text~~ → ~text~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_double_marker(&chars, i + 2, '~') {
                out.push('~');
                for ch in chars.iter().take(end).skip(i + 2) {
                    out.push(*ch);
                }
                out.push('~');
                i = end + 2;
                line_start = false;
                continue;
            }
        }

        // Link: [text](url) → text (url) — WhatsApp doesn't support markdown links
        if chars[i] == '[' {
            if let Some((text_end, url_end)) = find_link_span(&chars, i) {
                for ch in chars.iter().take(text_end).skip(i + 1) {
                    out.push(*ch);
                }
                out.push_str(" (");
                for ch in chars.iter().take(url_end).skip(text_end + 2) {
                    out.push(*ch);
                }
                out.push(')');
                i = url_end + 1;
                line_start = false;
                continue;
            }
        }

        if chars[i] == '\n' {
            out.push('\n');
            i += 1;
            line_start = true;
            continue;
        }

        out.push(chars[i]);
        i += 1;
        line_start = false;
    }

    out
}

/// Format text for a specific platform, converting from standard Markdown.
pub fn format_for_platform(text: &str, platform: Platform) -> String {
    match platform {
        Platform::Telegram => format_for_telegram(text),
        Platform::Discord => format_for_discord(text),
        Platform::WhatsApp => format_for_whatsapp(text),
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn push_escaped(out: &mut String, c: char) {
    if TELEGRAM_SPECIAL.contains(&c) {
        out.push('\\');
    }
    out.push(c);
}

/// Find closing single marker on the same line, not preceded by backslash.
fn find_closing_marker(chars: &[char], start: usize, marker: char) -> Option<usize> {
    if start >= chars.len() {
        return None;
    }
    for i in start..chars.len() {
        if chars[i] == marker && (i == 0 || chars[i - 1] != '\\') && i > start {
            return Some(i);
        }
        if chars[i] == '\n' {
            return None;
        }
    }
    None
}

/// Find closing double marker (e.g. `**`) on the same line.
fn find_double_marker(chars: &[char], start: usize, marker: char) -> Option<usize> {
    if start >= chars.len() {
        return None;
    }
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == marker
            && chars[i + 1] == marker
            && (i == 0 || chars[i - 1] != '\\')
            && i > start
        {
            return Some(i);
        }
        if chars[i] == '\n' {
            return None;
        }
        i += 1;
    }
    None
}

/// Find `[text](url)` span starting at `[`. Returns `(index of ], index of ))`
fn find_link_span(chars: &[char], start: usize) -> Option<(usize, usize)> {
    let mut i = start + 1;
    while i < chars.len() && chars[i] != ']' && chars[i] != '\n' {
        i += 1;
    }
    if i >= chars.len() || chars[i] != ']' {
        return None;
    }
    let text_end = i;
    i += 1;
    if i >= chars.len() || chars[i] != '(' {
        return None;
    }
    i += 1;
    let url_start = i;
    while i < chars.len() && chars[i] != ')' && chars[i] != '\n' {
        i += 1;
    }
    if i >= chars.len() || chars[i] != ')' {
        return None;
    }
    if i > url_start {
        Some((text_end, i))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Reply formatting (existing)
// ---------------------------------------------------------------------------

/// Formats and splits an outbound reply for a target platform.
///
/// Returns an empty vec for silent replies (`NO_REPLY`, `HEARTBEAT_OK`).
pub fn format_reply(platform: Platform, input: &str) -> Vec<String> {
    let text = input.trim();
    if text.is_empty() || is_silent_reply(text) {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_slash_commands() {
        assert_eq!(parse_slash_command("/status"), Some(SlashCommand::Status));
        assert_eq!(parse_slash_command("/compact"), Some(SlashCommand::Compact));
        assert_eq!(
            parse_slash_command("/reasoning on"),
            Some(SlashCommand::Reasoning { on: Some(true) })
        );
        assert_eq!(
            parse_slash_command("/reasoning off"),
            Some(SlashCommand::Reasoning { on: Some(false) })
        );
        assert_eq!(
            parse_slash_command("/model gpt-5"),
            Some(SlashCommand::Model {
                name: Some("gpt-5".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/model"),
            Some(SlashCommand::Model { name: None })
        );
        assert_eq!(parse_slash_command("/reset"), Some(SlashCommand::Reset));
        assert_eq!(
            parse_slash_command("/help"),
            Some(SlashCommand::Help { topic: None })
        );
    }

    #[test]
    fn parses_extended_slash_commands() {
        assert_eq!(parse_slash_command("/version"), Some(SlashCommand::Version));
        assert_eq!(
            parse_slash_command("/help topics"),
            Some(SlashCommand::Help {
                topic: Some("topics".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/thinking high"),
            Some(SlashCommand::Thinking {
                level: Some("high".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/verbose on"),
            Some(SlashCommand::Verbose { on: Some(true) })
        );
        assert_eq!(
            parse_slash_command("/session my-session"),
            Some(SlashCommand::Session {
                key: Some("my-session".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/sessions"),
            Some(SlashCommand::Sessions)
        );
        assert_eq!(
            parse_slash_command("/memory search term"),
            Some(SlashCommand::Memory {
                query: Some("search term".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/cron list"),
            Some(SlashCommand::Cron {
                action: Some("list".to_string()),
                args: vec![]
            })
        );
        assert_eq!(
            parse_slash_command("/cron add daily backup"),
            Some(SlashCommand::Cron {
                action: Some("add".to_string()),
                args: vec!["daily".to_string(), "backup".to_string()],
            })
        );
        assert_eq!(
            parse_slash_command("/approve abc123 allow-always"),
            Some(SlashCommand::Approve {
                code: "abc123".to_string(),
                mode: Some("allow-always".to_string()),
            })
        );
        assert_eq!(
            parse_slash_command("/logs 50"),
            Some(SlashCommand::Logs { tail: Some(50) })
        );
        assert_eq!(parse_slash_command("/debug"), Some(SlashCommand::Debug));
        assert_eq!(parse_slash_command("/pause"), Some(SlashCommand::Pause));
        assert_eq!(parse_slash_command("/resume"), Some(SlashCommand::Resume));
        assert_eq!(parse_slash_command("/agents"), Some(SlashCommand::Agents));
        assert_eq!(
            parse_slash_command("/spawn run backup now"),
            Some(SlashCommand::Spawn {
                prompt: "run backup now".to_string()
            })
        );
        assert_eq!(
            parse_slash_command("/kill session-42"),
            Some(SlashCommand::Kill {
                session: "session-42".to_string()
            })
        );
        assert_eq!(parse_slash_command("/ping"), Some(SlashCommand::Ping));
        assert_eq!(parse_slash_command("/context"), Some(SlashCommand::Context));
        assert_eq!(
            parse_slash_command("/history 10"),
            Some(SlashCommand::History { count: Some(10) })
        );
        assert_eq!(parse_slash_command("/cost"), Some(SlashCommand::Cost));
        assert_eq!(parse_slash_command("/whoami"), Some(SlashCommand::Whoami));
        assert_eq!(parse_slash_command("/clear"), Some(SlashCommand::Clear));
        assert_eq!(
            parse_slash_command("/config theme dark"),
            Some(SlashCommand::Config {
                key: Some("theme".to_string()),
                value: Some("dark".to_string()),
            })
        );
        assert_eq!(
            parse_slash_command("/announce #general"),
            Some(SlashCommand::Announce {
                channel: Some("#general".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/subscribe deploy"),
            Some(SlashCommand::Subscribe {
                event: Some("deploy".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/unsubscribe deploy"),
            Some(SlashCommand::Unsubscribe {
                event: Some("deploy".to_string())
            })
        );
        assert_eq!(
            parse_slash_command("/feedback great work"),
            Some(SlashCommand::Feedback {
                text: Some("great work".to_string())
            })
        );
    }

    #[test]
    fn parses_unknown_slash_commands() {
        assert_eq!(
            parse_slash_command("/foo bar baz"),
            Some(SlashCommand::Unknown {
                name: "foo".to_string(),
                args: vec!["bar".to_string(), "baz".to_string()],
            })
        );
        assert_eq!(parse_slash_command("not a command"), None);
        assert_eq!(parse_slash_command(""), None);
    }

    #[test]
    fn approve_requires_code() {
        assert_eq!(parse_slash_command("/approve"), None);
    }

    #[test]
    fn spawn_requires_prompt() {
        assert_eq!(parse_slash_command("/spawn"), None);
    }

    #[test]
    fn kill_requires_session() {
        assert_eq!(parse_slash_command("/kill"), None);
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
        assert!(format_reply(Platform::Telegram, "HEARTBEAT_OK").is_empty());
        let long = "word ".repeat(900);
        let chunks = format_reply(Platform::Discord, &long);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 2000));
    }

    #[test]
    fn silent_reply_detection() {
        assert!(is_silent_reply("NO_REPLY"));
        assert!(is_silent_reply("  NO_REPLY  "));
        assert!(is_silent_reply("HEARTBEAT_OK"));
        assert!(is_silent_reply("  HEARTBEAT_OK\n"));
        assert!(!is_silent_reply("Hello world"));
        assert!(!is_silent_reply("NO_REPLY but more text"));
        assert!(is_heartbeat_ok("HEARTBEAT_OK"));
        assert!(is_heartbeat_ok(" HEARTBEAT_OK "));
        assert!(!is_heartbeat_ok("NO_REPLY"));
    }

    #[test]
    fn escape_telegram_preserves_formatting() {
        // Plain text special chars get escaped
        assert_eq!(escape_telegram_v2("hello.world"), "hello\\.world");
        assert_eq!(escape_telegram_v2("1+1=2"), "1\\+1\\=2");
        // Bold preserved
        assert_eq!(escape_telegram_v2("*bold*"), "*bold*");
        // Italic preserved
        assert_eq!(escape_telegram_v2("_italic_"), "_italic_");
        // Code preserved
        assert_eq!(escape_telegram_v2("`code`"), "`code`");
        // Code block preserved (no escaping inside)
        assert_eq!(
            escape_telegram_v2("```rust\nfn main() {}\n```"),
            "```rust\nfn main() {}\n```"
        );
        // Link preserved
        assert_eq!(
            escape_telegram_v2("[click](https://example.com)"),
            "[click](https://example.com)"
        );
    }

    #[test]
    fn format_for_telegram_converts_markdown() {
        // Headers become bold
        assert_eq!(format_for_telegram("# Hello"), "*Hello*");
        // Double asterisks to single
        assert_eq!(format_for_telegram("**bold**"), "*bold*");
        // Single asterisks to underscores (italic)
        assert_eq!(format_for_telegram("*italic*"), "_italic_");
        // Strikethrough: ~~ to ~
        assert_eq!(format_for_telegram("~~strike~~"), "~strike~");
        // Code preserved
        assert_eq!(format_for_telegram("`code`"), "`code`");
        // Plain dots escaped
        assert!(format_for_telegram("end.").contains("\\."));
    }

    #[test]
    fn format_for_discord_passthrough() {
        let md = "**bold** *italic* `code`";
        assert_eq!(format_for_discord(md), md);
    }

    #[test]
    fn format_for_whatsapp_converts() {
        assert_eq!(format_for_whatsapp("**bold**"), "*bold*");
        assert_eq!(format_for_whatsapp("*italic*"), "_italic_");
        assert_eq!(format_for_whatsapp("~~strike~~"), "~strike~");
        assert_eq!(format_for_whatsapp("# Header"), "*Header*");
    }

    #[test]
    fn format_for_platform_dispatches() {
        let text = "**hello**";
        assert_eq!(format_for_platform(text, Platform::Discord), "**hello**");
        assert_eq!(format_for_platform(text, Platform::Telegram), "*hello*");
        assert_eq!(format_for_platform(text, Platform::WhatsApp), "*hello*");
    }

    #[test]
    fn collect_window_batches() {
        let mut w = CollectWindow::with_window_ms(100);
        let msg = InboundMessage {
            channel: "test".to_string(),
            chat_id: None,
            user_id: "u1".to_string(),
            text: "hi".to_string(),
            is_dm: true,
            mentioned: false,
            priority: 1,
        };
        // First add should not produce a batch
        assert!(w.add(msg.clone()).is_none());
        // Immediate check should not produce (window not expired)
        assert!(w.check_expiry().is_none());
    }

    #[test]
    fn collect_window_default() {
        let w = CollectWindow::default();
        assert_eq!(w.window_ms, 500);
    }

    #[test]
    fn dm_gate_enforcement() {
        let mut gate = DmGate::new(DmPolicy::Allowlist);
        gate.allowlist.insert("user1".to_string());
        assert!(gate.is_allowed("user1", ChatType::Direct));
        assert!(!gate.is_allowed("user2", ChatType::Direct));
        // Group messages always allowed
        assert!(gate.is_allowed("user2", ChatType::Group));

        let open_gate = DmGate::new(DmPolicy::Open);
        assert!(open_gate.is_allowed("anyone", ChatType::Direct));

        let mut pairing_gate = DmGate::new(DmPolicy::Pairing);
        pairing_gate.paired.insert("paired_user".to_string());
        pairing_gate.allowlist.insert("allowed_user".to_string());
        assert!(pairing_gate.is_allowed("paired_user", ChatType::Direct));
        assert!(pairing_gate.is_allowed("allowed_user", ChatType::Direct));
        assert!(!pairing_gate.is_allowed("unknown", ChatType::Direct));
    }

    #[test]
    fn extract_reply_tags() {
        let (text, reply) = extract_reply_tag("[[reply_to_current]]Hello!");
        assert_eq!(text, "Hello!");
        assert_eq!(
            reply,
            Some(ReplyRef {
                message_id: "current".to_string(),
                quote_text: None,
            })
        );

        let (text, reply) = extract_reply_tag("[[reply_to:12345]]Thanks");
        assert_eq!(text, "Thanks");
        assert_eq!(
            reply,
            Some(ReplyRef {
                message_id: "12345".to_string(),
                quote_text: None,
            })
        );

        let (text, reply) = extract_reply_tag("No reply tag here");
        assert_eq!(text, "No reply tag here");
        assert!(reply.is_none());
    }

    #[test]
    fn extract_reply_tag_with_whitespace() {
        let (text, reply) = extract_reply_tag("  [[reply_to_current]]  spaced");
        assert_eq!(text, "spaced");
        assert!(reply.is_some());

        let (text, reply) = extract_reply_tag("  [[reply_to:msg-99]]  answer");
        assert_eq!(text, "answer");
        assert_eq!(reply.unwrap().message_id, "msg-99");
    }
}

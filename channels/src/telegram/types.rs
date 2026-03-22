use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::framework::{ChatType, MessageId, ParseMode};

/// Telegram text length limit.
pub const TELEGRAM_MAX_MESSAGE_LEN: usize = 4096;

/// Telegram-supported inline button styling hint for app-level theming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramInlineButtonStyle {
    Default,
    Primary,
    Success,
    Danger,
}

/// Telegram inline keyboard button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramInlineButton {
    pub text: String,
    pub callback_data: Option<String>,
    pub url: Option<String>,
    pub switch_inline_query: Option<String>,
    pub style: TelegramInlineButtonStyle,
}

impl TelegramInlineButton {
    pub fn callback(text: impl Into<String>, callback_data: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            callback_data: Some(callback_data.into()),
            url: None,
            switch_inline_query: None,
            style: TelegramInlineButtonStyle::Default,
        }
    }

    pub fn url(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            callback_data: None,
            url: Some(url.into()),
            switch_inline_query: None,
            style: TelegramInlineButtonStyle::Default,
        }
    }
}

/// Telegram inline keyboard markup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramInlineKeyboardMarkup {
    pub rows: Vec<Vec<TelegramInlineButton>>,
}

/// Telegram formatting entity type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramEntityKind {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Code,
    Pre,
    Link,
}

/// Telegram formatting entity tracked with char-based offsets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramMessageEntity {
    pub kind: TelegramEntityKind,
    pub offset: usize,
    pub length: usize,
    pub url: Option<String>,
}

/// Parsed Telegram text with entity metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramFormattedText {
    pub text: String,
    pub entities: Vec<TelegramMessageEntity>,
    pub parse_mode: ParseMode,
}

/// Telegram media payload retained in local runtime state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelegramMedia {
    pub file_id: String,
    pub kind: TelegramMediaKind,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub bytes: Vec<u8>,
    pub duration_seconds: Option<u32>,
    pub sticker_emoji: Option<String>,
    pub is_animated: bool,
    pub is_video_note: bool,
}

impl TelegramMedia {
    pub fn kind_name(&self) -> &'static str {
        self.kind.endpoint_name()
    }
}

/// Telegram media endpoint kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramMediaKind {
    Photo,
    Voice,
    Document,
    VideoNote,
    Video,
    Sticker,
    Animation,
}

impl TelegramMediaKind {
    pub fn endpoint_name(self) -> &'static str {
        match self {
            Self::Photo => "sendPhoto",
            Self::Voice => "sendVoice",
            Self::Document => "sendDocument",
            Self::VideoNote => "sendVideoNote",
            Self::Video => "sendVideo",
            Self::Sticker => "sendSticker",
            Self::Animation => "sendAnimation",
        }
    }
}

/// Telegram live/static location payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelegramLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub live_period_seconds: Option<u32>,
}

/// Telegram poll kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramPollKind {
    Regular,
    Quiz,
}

/// Telegram poll request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramPollRequest {
    pub question: String,
    pub options: Vec<String>,
    pub kind: TelegramPollKind,
    pub is_anonymous: bool,
    pub correct_option_id: Option<usize>,
}

/// Sticker metadata preserved from Telegram sticker messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramStickerMetadata {
    pub emoji: Option<String>,
    pub set_name: Option<String>,
    pub is_animated: bool,
    pub is_video: bool,
}

/// Telegram reaction payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelegramReaction {
    Emoji(String),
    CustomEmoji(String),
}

impl TelegramReaction {
    pub fn as_value(&self) -> &str {
        match self {
            Self::Emoji(value) | Self::CustomEmoji(value) => value,
        }
    }
}

/// Aggregated reaction count for a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramReactionCount {
    pub reaction: TelegramReaction,
    pub count: u64,
}

/// Update describing a message reaction change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramReactionUpdate {
    pub chat_id: String,
    pub message_id: i64,
    pub actor_user_id: String,
    pub old_reactions: Vec<TelegramReaction>,
    pub new_reactions: Vec<TelegramReaction>,
    pub counts: Vec<TelegramReactionCount>,
}

/// Bot chat action sent before content delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatAction {
    Typing,
    UploadPhoto,
    UploadDocument,
    RecordVoice,
    UploadVideo,
    UploadVideoNote,
}

impl TelegramChatAction {
    pub fn api_name(self) -> &'static str {
        match self {
            Self::Typing => "typing",
            Self::UploadPhoto => "upload_photo",
            Self::UploadDocument => "upload_document",
            Self::RecordVoice => "record_voice",
            Self::UploadVideo => "upload_video",
            Self::UploadVideoNote => "upload_video_note",
        }
    }
}

/// Forum topic metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramForumTopic {
    pub topic_id: i64,
    pub chat_id: String,
    pub title: String,
    pub icon_color: Option<String>,
}

/// Telegram member status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramMemberStatus {
    Owner,
    Administrator,
    Member,
    Restricted,
    Left,
    Banned,
}

/// Telegram chat member state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramChatMember {
    pub user_id: String,
    pub username: Option<String>,
    pub status: TelegramMemberStatus,
    pub can_send_messages: bool,
    pub can_manage_topics: bool,
    pub can_delete_messages: bool,
    pub is_bot: bool,
}

/// Group management update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramChatMemberUpdate {
    pub chat_id: String,
    pub old_member: TelegramChatMember,
    pub new_member: TelegramChatMember,
}

/// Bot permissions tracked for a chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramBotPermissions {
    pub can_send_messages: bool,
    pub can_manage_topics: bool,
    pub can_restrict_members: bool,
    pub can_delete_messages: bool,
}

/// Quote-forward request for message forwarding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramQuoteForward {
    pub source_chat_id: String,
    pub source_message_id: i64,
    pub quote: Option<String>,
}

/// Parsed Telegram message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramMessage {
    pub message_id: i64,
    pub chat_id: String,
    pub chat_type: ChatType,
    pub from_user_id: Option<String>,
    pub from_username: Option<String>,
    pub bot_username: Option<String>,
    pub text: Option<String>,
    pub message_thread_id: Option<i64>,
    pub reply_to_message_id: Option<i64>,
    pub entities: Vec<TelegramMessageEntity>,
    pub media: Vec<TelegramMedia>,
    pub inline_keyboard: Option<TelegramInlineKeyboardMarkup>,
    pub reactions: Vec<TelegramReactionCount>,
    pub location: Option<TelegramLocation>,
    pub poll: Option<TelegramPollRequest>,
    pub sticker: Option<TelegramStickerMetadata>,
    pub quote: Option<TelegramQuoteForward>,
}

/// Telegram callback query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramCallbackQuery {
    pub id: String,
    pub from_user_id: String,
    pub from_username: Option<String>,
    pub bot_username: Option<String>,
    pub data: Option<String>,
    pub chat_id: Option<String>,
    pub message_id: Option<i64>,
}

/// Callback answer result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramCallbackAnswer {
    pub text: Option<String>,
    pub show_alert: bool,
    pub url: Option<String>,
}

/// Telegram webhook status for an account.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramWebhookState {
    pub active: bool,
    pub url: Option<String>,
    pub secret_token: Option<String>,
    pub last_delivery_at: Option<DateTime<Utc>>,
    pub consecutive_failures: usize,
}

/// Inbound Telegram update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub bot_username: Option<String>,
    pub message: Option<TelegramMessage>,
    pub edited_message: Option<TelegramMessage>,
    pub callback_query: Option<TelegramCallbackQuery>,
    pub reaction: Option<TelegramReactionUpdate>,
    pub chat_member: Option<TelegramChatMemberUpdate>,
}

/// High-level delivery operation recorded by the local mock runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramOperation {
    SendText,
    SendPhoto,
    SendVoice,
    SendDocument,
    SendVideoNote,
    SendVideo,
    SendSticker,
    SendAnimation,
    SendLocation,
    SendPoll,
    SendChatAction,
    AnswerCallbackQuery,
    SetMessageReaction,
    ForwardMessage,
    CreateForumTopic,
    BanMember,
    KickMember,
    SetWebhook,
    DeleteWebhook,
}

/// Stored outbound delivery record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelegramDelivery {
    pub id: MessageId,
    pub account_name: String,
    pub bot_username: String,
    pub operation: TelegramOperation,
    pub chat_id: String,
    pub thread_id: Option<i64>,
    pub text: Option<String>,
    pub parse_mode: Option<ParseMode>,
    pub entities: Vec<TelegramMessageEntity>,
    pub media: Vec<TelegramMedia>,
    pub keyboard: Option<TelegramInlineKeyboardMarkup>,
    pub reactions: Vec<TelegramReaction>,
    pub location: Option<TelegramLocation>,
    pub poll: Option<TelegramPollRequest>,
    pub quote_forward: Option<TelegramQuoteForward>,
    pub callback_answer: Option<TelegramCallbackAnswer>,
    pub chat_action: Option<TelegramChatAction>,
    pub silent: bool,
    pub created_at: DateTime<Utc>,
    pub continuation_index: Option<usize>,
    pub continuation_total: Option<usize>,
}

/// Update-processing result stored for tests and runtime inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramProcessedUpdate {
    pub account_name: String,
    pub update_id: i64,
    pub bot_username: String,
    pub kind: String,
    pub callback_data: Option<String>,
    pub chat_id: Option<String>,
    pub thread_id: Option<i64>,
}

/// Target selector for outbound Telegram operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramTarget {
    pub account_name: Option<String>,
    pub bot_username: Option<String>,
    pub chat_id: String,
    pub thread_id: Option<i64>,
}

impl TelegramTarget {
    pub fn chat(chat_id: impl Into<String>) -> Self {
        Self {
            account_name: None,
            bot_username: None,
            chat_id: chat_id.into(),
            thread_id: None,
        }
    }

    pub fn with_account(mut self, account_name: impl Into<String>) -> Self {
        self.account_name = Some(account_name.into());
        self
    }

    pub fn with_bot_username(mut self, bot_username: impl Into<String>) -> Self {
        self.bot_username = Some(bot_username.into());
        self
    }

    pub fn with_thread(mut self, thread_id: i64) -> Self {
        self.thread_id = Some(thread_id);
        self
    }

    pub fn parse(input: &str) -> Self {
        if let Some(stripped) = input.strip_prefix('@') {
            let mut parts = stripped.splitn(3, ':');
            let bot_username = parts.next().unwrap_or_default();
            let chat_id = parts.next().unwrap_or_default();
            let thread_id = parts.next().and_then(|value| value.parse::<i64>().ok());
            return Self::chat(chat_id)
                .with_bot_username(bot_username)
                .with_thread_opt(thread_id);
        }

        if input.contains("::") {
            let mut parts = input.splitn(3, "::");
            let account = parts.next().unwrap_or_default();
            let chat_id = parts.next().unwrap_or_default();
            let thread_id = parts.next().and_then(|value| value.parse::<i64>().ok());
            return Self::chat(chat_id)
                .with_account(account)
                .with_thread_opt(thread_id);
        }

        Self::chat(input)
    }

    fn with_thread_opt(mut self, thread_id: Option<i64>) -> Self {
        self.thread_id = thread_id;
        self
    }
}

impl fmt::Display for TelegramTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(account_name) = &self.account_name {
            if let Some(thread_id) = self.thread_id {
                return write!(f, "{account_name}::{}::{thread_id}", self.chat_id);
            }
            return write!(f, "{account_name}::{}", self.chat_id);
        }

        if let Some(bot_username) = &self.bot_username {
            if let Some(thread_id) = self.thread_id {
                return write!(f, "{bot_username}:{}:{thread_id}", self.chat_id);
            }
            return write!(f, "{bot_username}:{}", self.chat_id);
        }

        if let Some(thread_id) = self.thread_id {
            return write!(f, "{}::{thread_id}", self.chat_id);
        }

        f.write_str(&self.chat_id)
    }
}

/// Account-specific connection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryMode {
    Polling,
    Webhook,
    FallbackPolling,
}

/// Account health state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramAccountHealthState {
    Connected,
    Disconnected,
    Reconnecting,
    RateLimited,
    AuthError,
    WebhookOnly,
}

/// Detailed Telegram account health snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramAccountHealth {
    pub account_name: String,
    pub bot_username: String,
    pub state: TelegramAccountHealthState,
    pub delivery_mode: TelegramDeliveryMode,
    pub last_error: Option<String>,
    pub consecutive_failures: usize,
    pub last_update_offset: i64,
    pub last_update_at: Option<DateTime<Utc>>,
}

/// Telegram API error kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramApiErrorKind {
    RateLimited,
    FloodWait,
    Server,
    Unauthorized,
    Blocked,
    NetworkTimeout,
    PermissionDenied,
    NotFound,
    Config,
}

/// Telegram API error used by the local runtime simulator.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("{kind:?}: {message}")]
pub struct TelegramApiError {
    pub kind: TelegramApiErrorKind,
    pub message: String,
    pub status_code: Option<u16>,
    pub retry_after_seconds: Option<u64>,
}

impl TelegramApiError {
    pub fn new(kind: TelegramApiErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status_code: None,
            retry_after_seconds: None,
        }
    }

    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn with_retry_after(mut self, retry_after_seconds: u64) -> Self {
        self.retry_after_seconds = Some(retry_after_seconds);
        self
    }

    pub fn rate_limited(retry_after_seconds: u64) -> Self {
        Self::new(TelegramApiErrorKind::RateLimited, "telegram rate limited")
            .with_status_code(429)
            .with_retry_after(retry_after_seconds)
    }

    pub fn flood_wait(wait_seconds: u64) -> Self {
        Self::new(TelegramApiErrorKind::FloodWait, "telegram flood wait")
            .with_status_code(429)
            .with_retry_after(wait_seconds)
    }

    pub fn server(message: impl Into<String>) -> Self {
        Self::new(TelegramApiErrorKind::Server, message).with_status_code(500)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(TelegramApiErrorKind::Unauthorized, message).with_status_code(401)
    }

    pub fn blocked(message: impl Into<String>) -> Self {
        Self::new(TelegramApiErrorKind::Blocked, message).with_status_code(403)
    }

    pub fn network_timeout(message: impl Into<String>) -> Self {
        Self::new(TelegramApiErrorKind::NetworkTimeout, message)
    }

    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after_seconds.map(Duration::from_secs)
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            TelegramApiErrorKind::RateLimited
                | TelegramApiErrorKind::FloodWait
                | TelegramApiErrorKind::Server
                | TelegramApiErrorKind::NetworkTimeout
        )
    }
}

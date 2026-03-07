use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Result type for channel operations.
pub type Result<T> = std::result::Result<T, ChannelError>;

/// Message identifier returned by platform APIs.
pub type MessageId = String;

/// Error type for channel operations.
#[derive(Debug, Error)]
pub enum ChannelError {
    /// Returned when a channel implementation has not been enabled.
    #[error("platform is not enabled: {0}")]
    PlatformDisabled(&'static str),
    /// Returned when channel is not registered in the runtime.
    #[error("channel not registered for platform: {0:?}")]
    ChannelNotRegistered(Platform),
    /// Returned when a platform API rejects a request.
    #[error("platform request failed: {0}")]
    PlatformRequest(String),
}

/// Supported chat platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Telegram.
    Telegram,
    /// Discord.
    Discord,
    /// WhatsApp.
    WhatsApp,
    /// Signal.
    Signal,
    /// Slack.
    Slack,
    /// iMessage.
    IMessage,
    /// LINE.
    Line,
    /// WebSocket web chat.
    Web,
}

/// Chat context type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatType {
    /// Direct one-to-one chat.
    Direct,
    /// Group chat.
    Group,
}

/// Normalized sender metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sender {
    /// Sender id from platform.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional username/handle.
    pub username: Option<String>,
}

/// Supported media attachment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    /// Image content.
    Image,
    /// Voice note/audio clip.
    Voice,
    /// Generic document or file.
    Document,
    /// Video content.
    Video,
    /// Sticker media.
    Sticker,
    /// Location payload.
    Location,
}

/// Unified media attachment metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaAttachment {
    /// Attachment type.
    pub kind: MediaType,
    /// Optional remote URL.
    pub url: Option<String>,
    /// Optional local file path for downloaded assets.
    pub file_path: Option<String>,
    /// Optional mime type.
    pub mime_type: Option<String>,
    /// Optional platform-specific id.
    pub platform_id: Option<String>,
}

/// Supported parse modes for outbound text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseMode {
    /// Markdown format.
    Markdown,
    /// HTML format.
    Html,
    /// Plain text.
    Plain,
}

/// Inline button payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineButton {
    /// Label visible to users.
    pub text: String,
    /// Callback payload.
    pub callback_data: String,
}

/// Unified inbound message for all channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboundMessage {
    /// Message id.
    pub id: String,
    /// Source platform.
    pub platform: Platform,
    /// Chat id.
    pub chat_id: String,
    /// Chat context type.
    pub chat_type: ChatType,
    /// Sender metadata.
    pub sender: Sender,
    /// Optional text content.
    pub text: Option<String>,
    /// Optional referenced message id.
    pub reply_to: Option<String>,
    /// Media attachments.
    pub media: Vec<MediaAttachment>,
    /// Message timestamp.
    pub timestamp: DateTime<Utc>,
    /// Platform-specific raw payload.
    pub raw: Value,
}

impl InboundMessage {
    /// Normalizes common text fields in-place.
    pub fn normalize(&mut self) {
        self.text = self
            .text
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
    }
}

/// Unified outbound message payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundMessage {
    /// Text body.
    pub text: String,
    /// Optional referenced message id for reply behavior.
    pub reply_to: Option<String>,
    /// Optional media payloads.
    pub media: Vec<MediaAttachment>,
    /// Optional inline buttons (rows x columns).
    pub buttons: Option<Vec<Vec<InlineButton>>>,
    /// Silent delivery hint.
    pub silent: bool,
    /// Optional parse mode for text.
    pub parse_mode: Option<ParseMode>,
}

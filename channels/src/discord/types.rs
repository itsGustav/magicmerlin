use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::framework::{ChatType, MessageId, ParseMode};

pub const DISCORD_MAX_MESSAGE_LEN: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub application_id: String,
    pub guild_allowlist: Vec<String>,
    pub channel_allowlist: Vec<String>,
    pub dm_enabled: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            application_id: String::new(),
            guild_allowlist: Vec::new(),
            channel_allowlist: Vec::new(),
            dm_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordHello {
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordSession {
    pub session_id: String,
    pub sequence: Option<u64>,
    pub resume_gateway_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<u32>,
    pub fields: Vec<DiscordEmbedField>,
    pub footer: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordEmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

impl DiscordEmbed {
    pub fn builder() -> DiscordEmbedBuilder {
        DiscordEmbedBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct DiscordEmbedBuilder {
    title: Option<String>,
    description: Option<String>,
    color: Option<u32>,
    fields: Vec<DiscordEmbedField>,
    footer: Option<String>,
    thumbnail_url: Option<String>,
}

impl DiscordEmbedBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn field(mut self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        self.fields.push(DiscordEmbedField {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail_url = Some(url.into());
        self
    }

    pub fn build(self) -> DiscordEmbed {
        DiscordEmbed {
            title: self.title,
            description: self.description,
            color: self.color,
            fields: self.fields,
            footer: self.footer,
            thumbnail_url: self.thumbnail_url,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordAttachment {
    pub id: String,
    pub filename: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub id: MessageId,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub author_id: String,
    pub author_name: Option<String>,
    pub content: String,
    pub thread_id: Option<String>,
    pub reply_to: Option<MessageId>,
    pub attachments: Vec<DiscordAttachment>,
    pub embeds: Vec<DiscordEmbed>,
    pub parse_mode: Option<ParseMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordThread {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordCommandOption {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordInteraction {
    pub id: String,
    pub command_name: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub user_id: String,
    pub thread_id: Option<String>,
    pub options: Vec<DiscordCommandOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordProcessedEvent {
    pub kind: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub thread_id: Option<String>,
    pub session_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordPresence {
    pub status: String,
    pub activity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscordGatewayState {
    Disconnected,
    Connecting,
    Identified,
    Resuming,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordHealth {
    pub state: DiscordGatewayState,
    pub last_sequence: Option<u64>,
    pub heartbeat_interval: Option<DurationHolder>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationHolder {
    pub millis: u64,
}

impl From<Duration> for DurationHolder {
    fn from(value: Duration) -> Self {
        Self {
            millis: value.as_millis() as u64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscordResponseKind {
    Immediate,
    Deferred,
    Followup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordInteractionResponse {
    pub interaction_id: String,
    pub kind: DiscordResponseKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscordApiErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
    Transport,
    InvalidInput,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[error("discord {kind:?}: {message}")]
pub struct DiscordApiError {
    pub kind: DiscordApiErrorKind,
    pub message: String,
}

impl DiscordApiError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            kind: DiscordApiErrorKind::Unauthorized,
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            kind: DiscordApiErrorKind::Forbidden,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            kind: DiscordApiErrorKind::InvalidInput,
            message: message.into(),
        }
    }
}

impl fmt::Display for DiscordGatewayState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Identified => write!(f, "identified"),
            Self::Resuming => write!(f, "resuming"),
            Self::Ready => write!(f, "ready"),
        }
    }
}

pub fn session_scope(chat_type: ChatType, guild_id: Option<&str>, channel_id: &str, thread_id: Option<&str>, user_id: &str) -> String {
    match chat_type {
        ChatType::Direct => format!("discord:dm:{user_id}"),
        ChatType::Group => {
            if let Some(thread_id) = thread_id {
                format!("discord:thread:{thread_id}")
            } else if let Some(guild_id) = guild_id {
                format!("discord:guild:{guild_id}:channel:{channel_id}")
            } else {
                format!("discord:channel:{channel_id}")
            }
        }
    }
}

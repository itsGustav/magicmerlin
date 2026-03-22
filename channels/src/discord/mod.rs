//! Discord channel implementation with gateway + REST semantics.

pub mod audit;
pub mod channel_mgmt;
pub mod components;
pub mod guild;
mod runtime;
pub mod scheduled_events;
mod types;
pub mod voice;
pub mod webhook;

pub use runtime::{DiscordChannel, DiscordResult};
pub use types::{
    session_scope, DiscordApiError, DiscordApiErrorKind, DiscordAttachment, DiscordCommandOption,
    DiscordConfig, DiscordEmbed, DiscordEmbedAuthor, DiscordEmbedBuilder, DiscordEmbedField,
    DiscordGatewayState, DiscordHealth, DiscordHello, DiscordInteraction,
    DiscordInteractionResponse, DiscordMessage, DiscordPresence, DiscordProcessedEvent,
    DiscordResponseKind, DiscordSession, DiscordThread, DISCORD_MAX_MESSAGE_LEN,
};

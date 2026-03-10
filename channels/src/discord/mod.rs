//! Discord channel implementation with gateway + REST semantics.

mod runtime;
mod types;

pub use runtime::{DiscordChannel, DiscordResult};
pub use types::{
    session_scope, DiscordApiError, DiscordApiErrorKind, DiscordAttachment, DiscordConfig,
    DiscordCommandOption, DiscordEmbed, DiscordEmbedBuilder, DiscordEmbedField,
    DiscordGatewayState, DiscordHealth, DiscordHello, DiscordInteraction,
    DiscordInteractionResponse, DiscordMessage, DiscordPresence, DiscordProcessedEvent,
    DiscordResponseKind, DiscordSession, DiscordThread, DISCORD_MAX_MESSAGE_LEN,
};

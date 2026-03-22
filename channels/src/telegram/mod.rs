//! Telegram channel implementation using Bot API semantics.

mod config;
mod formatting;
mod runtime;
mod types;
mod webhook;

pub use config::{
    TelegramAccount, TelegramConfig, TELEGRAM_DEFAULT_CHAT_RATE_LIMIT,
    TELEGRAM_DEFAULT_CHAT_RATE_WINDOW_SECONDS, TELEGRAM_DEFAULT_GLOBAL_RATE_LIMIT_PER_SECOND,
    TELEGRAM_DEFAULT_POLL_INTERVAL_MS, TELEGRAM_DEFAULT_POLL_TIMEOUT_SECONDS,
    TELEGRAM_DEFAULT_RETRY_LIMIT,
};
pub use formatting::{
    escape_markdown_v2, format_text, parse_html, parse_markdown_v2, split_formatted_text,
    split_message,
};
pub use runtime::{TelegramChannel, TelegramResult};
pub use types::{
    TelegramAccountHealth, TelegramAccountHealthState, TelegramApiError, TelegramApiErrorKind,
    TelegramBotPermissions, TelegramCallbackAnswer, TelegramCallbackQuery, TelegramChatAction,
    TelegramChatMember, TelegramChatMemberUpdate, TelegramDelivery, TelegramDeliveryMode,
    TelegramEntityKind, TelegramFormattedText, TelegramForumTopic, TelegramInlineButton,
    TelegramInlineButtonStyle, TelegramInlineKeyboardMarkup, TelegramLocation, TelegramMedia,
    TelegramMediaKind, TelegramMemberStatus, TelegramMessage, TelegramMessageEntity,
    TelegramOperation, TelegramPollKind, TelegramPollRequest, TelegramProcessedUpdate,
    TelegramQuoteForward, TelegramReaction, TelegramReactionCount, TelegramReactionUpdate,
    TelegramStickerMetadata, TelegramTarget, TelegramUpdate, TelegramWebhookState,
    TELEGRAM_MAX_MESSAGE_LEN,
};
pub use webhook::router as webhook_router;

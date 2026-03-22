//! Unified chat channel framework and platform integrations.

pub mod framework;

#[cfg(feature = "discord")]
pub mod discord;
#[cfg(all(feature = "imessage", target_os = "macos"))]
pub mod imessage;
#[cfg(feature = "line")]
pub mod line;
#[cfg(feature = "signal")]
pub mod signal;
#[cfg(feature = "slack")]
pub mod slack;
#[cfg(feature = "telegram")]
pub mod telegram;
#[cfg(feature = "web")]
pub mod web;
#[cfg(feature = "whatsapp")]
pub mod whatsapp;

pub use framework::{
    AutoReplyBridge, Channel, ChannelHealth, ChannelRegistry, ChatType, DmPolicy, DmPolicyEnforcer,
    HealthMonitor, InboundMessage, InlineButton, MediaAttachment, MediaType, MentionGate,
    MessageId, OutboundMessage, ParseMode, Platform, Result, Sender, ChannelError,
};

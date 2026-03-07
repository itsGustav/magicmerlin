//! Core channel abstractions and cross-platform behavior.

mod bridge;
mod formatting;
mod health;
mod policy;
mod registry;
mod types;

pub use bridge::AutoReplyBridge;
pub use formatting::{format_for_platform, split_for_platform, split_text_by_limit};
pub use health::{ChannelHealth, ConnectionState, HealthMonitor};
pub use policy::{DmPolicy, DmPolicyEnforcer, MentionGate};
pub use registry::{Channel, ChannelRegistry};
pub use types::{
    ChatType, InboundMessage, InlineButton, MediaAttachment, MediaType, MessageId, OutboundMessage,
    ParseMode, Platform, Result, Sender,
};

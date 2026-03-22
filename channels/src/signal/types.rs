//! Signal-specific types and inbound message conversion.

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::framework::{ChatType, InboundMessage, MediaAttachment, MediaType, Platform, Sender};

/// Raw Signal envelope as produced by `signal-cli --output=json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalEnvelope {
    /// Source phone number (legacy format).
    pub source: Option<String>,
    /// Source phone number.
    pub source_number: Option<String>,
    /// Source contact name.
    pub source_name: Option<String>,
    /// Envelope timestamp in milliseconds.
    pub timestamp: Option<i64>,
    /// Data message payload (absent for receipts, typing indicators, etc.).
    pub data_message: Option<DataMessage>,
}

/// Signal data message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataMessage {
    /// Message timestamp in milliseconds.
    pub timestamp: Option<i64>,
    /// Text body.
    pub message: Option<String>,
    /// Group information if this is a group message.
    pub group_info: Option<GroupInfo>,
    /// File attachments.
    pub attachments: Option<Vec<SignalAttachment>>,
}

/// Signal group metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    /// Base64-encoded group identifier.
    pub group_id: String,
    /// Group event type (e.g. "DELIVER").
    #[serde(rename = "type")]
    pub group_type: Option<String>,
}

/// Signal file attachment metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalAttachment {
    /// MIME content type.
    pub content_type: Option<String>,
    /// Original filename.
    pub filename: Option<String>,
    /// Platform attachment identifier.
    pub id: Option<String>,
    /// File size in bytes.
    pub size: Option<u64>,
}

impl SignalEnvelope {
    /// Converts a raw Signal envelope into a normalized [`InboundMessage`].
    ///
    /// Returns `None` if the envelope has no data message (e.g. delivery receipts,
    /// typing indicators).
    pub fn into_inbound(self) -> Option<InboundMessage> {
        let raw = serde_json::to_value(&self).unwrap_or(Value::Null);

        let Self {
            source,
            source_number,
            source_name,
            timestamp: env_ts,
            data_message,
        } = self;

        let data = data_message?;

        let sender_phone = source_number.or(source).unwrap_or_default();
        let sender_name = source_name.unwrap_or_else(|| sender_phone.clone());

        let ts = data.timestamp.or(env_ts).unwrap_or(0);
        let timestamp = Utc
            .timestamp_millis_opt(ts)
            .single()
            .unwrap_or_else(Utc::now);

        let (chat_id, chat_type) = match &data.group_info {
            Some(group) => (group.group_id.clone(), ChatType::Group),
            None => (sender_phone.clone(), ChatType::Direct),
        };

        let media = data
            .attachments
            .unwrap_or_default()
            .into_iter()
            .map(|a| {
                let kind = match a.content_type.as_deref() {
                    Some(ct) if ct.starts_with("image/") => MediaType::Image,
                    Some(ct) if ct.starts_with("video/") => MediaType::Video,
                    Some(ct) if ct.starts_with("audio/") => MediaType::Voice,
                    _ => MediaType::Document,
                };
                MediaAttachment {
                    kind,
                    url: None,
                    file_path: a.filename,
                    mime_type: a.content_type,
                    platform_id: a.id,
                }
            })
            .collect();

        Some(InboundMessage {
            id: format!("signal-{ts}"),
            platform: Platform::Signal,
            chat_id,
            chat_type,
            sender: Sender {
                id: sender_phone,
                name: sender_name,
                username: None,
            },
            text: data.message,
            reply_to: None,
            media,
            timestamp,
            raw,
        })
    }
}

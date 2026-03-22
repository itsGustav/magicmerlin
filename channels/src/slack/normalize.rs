//! Slack event → InboundMessage normalization.

use chrono::Utc;
use serde_json::Value;

use crate::framework::{ChatType, InboundMessage, MediaAttachment, MediaType, Platform, Sender};

/// Normalize a Slack event payload into an `InboundMessage`.
///
/// Handles: `message`, `app_mention`, `message_changed`, `message_deleted`.
/// Returns `None` for bot messages, subtypes we don't handle, or invalid payloads.
pub fn normalize_slack_event(
    event: &Value,
    bot_user_id: Option<&str>,
) -> Option<InboundMessage> {
    let event_type = event["type"].as_str()?;

    // Skip bot messages
    if event.get("bot_id").is_some() {
        return None;
    }
    if event.get("subtype").and_then(Value::as_str) == Some("bot_message") {
        return None;
    }

    match event_type {
        "message" | "app_mention" => normalize_message(event, bot_user_id),
        _ => None,
    }
}

fn normalize_message(event: &Value, bot_user_id: Option<&str>) -> Option<InboundMessage> {
    // Handle message_changed subtype
    let subtype = event.get("subtype").and_then(Value::as_str);
    if subtype == Some("message_changed") {
        // The actual message is nested inside "message"
        if let Some(inner) = event.get("message") {
            return normalize_message(inner, bot_user_id);
        }
        return None;
    }
    if subtype == Some("message_deleted") {
        return None;
    }

    let user = event["user"].as_str()?;
    let channel = event["channel"].as_str()?;
    let ts = event["ts"].as_str()?;
    let text = event["text"].as_str().map(|t| {
        let mut t = t.to_string();
        // Strip bot user mention: <@UBOT> text → text
        if let Some(bot_id) = bot_user_id {
            let mention = format!("<@{bot_id}>");
            if let Some(rest) = t.strip_prefix(&mention) {
                t = rest.trim_start().to_string();
            }
        }
        t
    });

    let thread_ts = event["thread_ts"].as_str().map(ToString::to_string);

    // Determine chat type from channel ID prefix
    // D = DM, C = public channel, G = group/private channel
    let chat_type = if channel.starts_with('D') {
        ChatType::Direct
    } else {
        ChatType::Group
    };

    // Extract file attachments if present
    let media = extract_media(event);

    Some(InboundMessage {
        id: ts.to_string(),
        platform: Platform::Slack,
        chat_id: channel.to_string(),
        chat_type,
        sender: Sender {
            id: user.to_string(),
            name: user.to_string(), // Resolved via users.info if needed
            username: None,
        },
        text,
        reply_to: thread_ts,
        media,
        timestamp: Utc::now(),
        raw: event.clone(),
    })
}

fn extract_media(event: &Value) -> Vec<MediaAttachment> {
    let mut media = Vec::new();
    if let Some(files) = event.get("files").and_then(Value::as_array) {
        for file in files {
            let mimetype = file["mimetype"].as_str().unwrap_or("");
            let kind = if mimetype.starts_with("image/") {
                MediaType::Image
            } else if mimetype.starts_with("video/") {
                MediaType::Video
            } else if mimetype.starts_with("audio/") {
                MediaType::Voice
            } else {
                MediaType::Document
            };

            media.push(MediaAttachment {
                kind,
                url: file["url_private"]
                    .as_str()
                    .or_else(|| file["permalink"].as_str())
                    .map(ToString::to_string),
                file_path: None,
                mime_type: Some(mimetype.to_string()),
                platform_id: file["id"].as_str().map(ToString::to_string),
            });
        }
    }
    media
}

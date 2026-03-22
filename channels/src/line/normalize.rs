//! LINE webhook event → InboundMessage normalization.

use chrono::Utc;
use serde_json::Value;

use crate::framework::{ChatType, InboundMessage, MediaAttachment, MediaType, Platform, Sender};

/// Normalize a LINE webhook event into an `InboundMessage`.
///
/// Supports event types: message (text/image/audio/video/file/location/sticker),
/// follow, unfollow, join, leave, postback.
/// Returns `None` for events that don't produce an inbound message.
pub fn normalize_line_event(event: &Value) -> Option<InboundMessage> {
    let event_type = event["type"].as_str()?;

    match event_type {
        "message" => normalize_message_event(event),
        "postback" => normalize_postback_event(event),
        "follow" | "join" => {
            // User followed the bot or bot joined a group — generate a system message
            let source = event.get("source")?;
            let (user_id, chat_id, chat_type) = extract_source(source)?;
            Some(InboundMessage {
                id: event["timestamp"]
                    .as_u64()
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                platform: Platform::Line,
                chat_id,
                chat_type,
                sender: Sender {
                    id: user_id.clone(),
                    name: user_id,
                    username: None,
                },
                text: Some(format!("[{event_type}]")),
                reply_to: None,
                media: Vec::new(),
                timestamp: Utc::now(),
                raw: event.clone(),
            })
        }
        _ => None,
    }
}

fn normalize_message_event(event: &Value) -> Option<InboundMessage> {
    let source = event.get("source")?;
    let message = event.get("message")?;
    let (user_id, chat_id, chat_type) = extract_source(source)?;

    let msg_type = message["type"].as_str().unwrap_or("text");
    let msg_id = message["id"].as_str().unwrap_or_default().to_string();

    let (text, media) = match msg_type {
        "text" => (
            message["text"].as_str().map(ToString::to_string),
            Vec::new(),
        ),
        "image" => (
            None,
            vec![MediaAttachment {
                kind: MediaType::Image,
                url: None,
                file_path: None,
                mime_type: Some("image/jpeg".to_string()),
                platform_id: Some(msg_id.clone()),
            }],
        ),
        "video" => (
            None,
            vec![MediaAttachment {
                kind: MediaType::Video,
                url: None,
                file_path: None,
                mime_type: Some("video/mp4".to_string()),
                platform_id: Some(msg_id.clone()),
            }],
        ),
        "audio" => (
            None,
            vec![MediaAttachment {
                kind: MediaType::Voice,
                url: None,
                file_path: None,
                mime_type: Some("audio/m4a".to_string()),
                platform_id: Some(msg_id.clone()),
            }],
        ),
        "file" => (
            message["fileName"].as_str().map(|n| format!("[file: {n}]")),
            vec![MediaAttachment {
                kind: MediaType::Document,
                url: None,
                file_path: None,
                mime_type: None,
                platform_id: Some(msg_id.clone()),
            }],
        ),
        "location" => {
            let title = message["title"].as_str().unwrap_or("Location");
            let lat = message["latitude"].as_f64().unwrap_or(0.0);
            let lng = message["longitude"].as_f64().unwrap_or(0.0);
            (
                Some(format!("[location: {title} ({lat},{lng})]")),
                vec![MediaAttachment {
                    kind: MediaType::Location,
                    url: None,
                    file_path: None,
                    mime_type: None,
                    platform_id: None,
                }],
            )
        }
        "sticker" => {
            let pkg = message["packageId"].as_str().unwrap_or("?");
            let stk = message["stickerId"].as_str().unwrap_or("?");
            (
                Some(format!("[sticker:{pkg}/{stk}]")),
                vec![MediaAttachment {
                    kind: MediaType::Sticker,
                    url: None,
                    file_path: None,
                    mime_type: None,
                    platform_id: Some(format!("{pkg}/{stk}")),
                }],
            )
        }
        _ => (Some(format!("[{msg_type}]")), Vec::new()),
    };

    Some(InboundMessage {
        id: msg_id,
        platform: Platform::Line,
        chat_id,
        chat_type,
        sender: Sender {
            id: user_id.clone(),
            name: user_id,
            username: None,
        },
        text,
        reply_to: None,
        media,
        timestamp: Utc::now(),
        raw: event.clone(),
    })
}

fn normalize_postback_event(event: &Value) -> Option<InboundMessage> {
    let source = event.get("source")?;
    let (user_id, chat_id, chat_type) = extract_source(source)?;
    let data = event
        .get("postback")
        .and_then(|p| p["data"].as_str())
        .unwrap_or("")
        .to_string();

    Some(InboundMessage {
        id: event["timestamp"]
            .as_u64()
            .map(|t| t.to_string())
            .unwrap_or_default(),
        platform: Platform::Line,
        chat_id,
        chat_type,
        sender: Sender {
            id: user_id.clone(),
            name: user_id,
            username: None,
        },
        text: Some(format!("[postback:{data}]")),
        reply_to: None,
        media: Vec::new(),
        timestamp: Utc::now(),
        raw: event.clone(),
    })
}

/// Extract (user_id, chat_id, chat_type) from a LINE source object.
fn extract_source(source: &Value) -> Option<(String, String, ChatType)> {
    let source_type = source["type"].as_str()?;
    let user_id = source["userId"].as_str().unwrap_or("unknown").to_string();

    match source_type {
        "user" => Some((user_id.clone(), user_id, ChatType::Direct)),
        "group" => {
            let group_id = source["groupId"].as_str()?.to_string();
            Some((user_id, group_id, ChatType::Group))
        }
        "room" => {
            let room_id = source["roomId"].as_str()?.to_string();
            Some((user_id, room_id, ChatType::Group))
        }
        _ => None,
    }
}

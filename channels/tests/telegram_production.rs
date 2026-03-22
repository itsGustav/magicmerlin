#![cfg(feature = "telegram")]

use std::fs;
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use magicmerlin_channels::framework::{ChatType, ParseMode};
use magicmerlin_channels::telegram::{
    webhook_router, TelegramAccount, TelegramAccountHealthState, TelegramApiError,
    TelegramBotPermissions, TelegramCallbackAnswer, TelegramCallbackQuery, TelegramChannel,
    TelegramChatAction, TelegramChatMember, TelegramConfig, TelegramDelivery, TelegramInlineButton,
    TelegramInlineButtonStyle, TelegramInlineKeyboardMarkup, TelegramLocation, TelegramMedia,
    TelegramMediaKind, TelegramMemberStatus, TelegramMessage, TelegramOperation, TelegramPollKind,
    TelegramPollRequest, TelegramQuoteForward, TelegramReaction, TelegramReactionCount,
    TelegramReactionUpdate, TelegramTarget, TelegramUpdate,
};
use magicmerlin_channels::Channel;
use tower::ServiceExt;

fn account(name: &str, bot_username: &str, media_dir: &str) -> TelegramAccount {
    TelegramAccount {
        name: name.to_string(),
        token: format!("token-{name}"),
        bot_username: bot_username.to_string(),
        polling_enabled: true,
        media_dir: Some(media_dir.to_string()),
        webhook_secret: Some(format!("secret-{name}")),
    }
}

fn test_config() -> TelegramConfig {
    TelegramConfig {
        accounts: vec![
            account("alpha", "bot_alpha", "/tmp/magicmerlin-telegram-alpha"),
            account("beta", "bot_beta", "/tmp/magicmerlin-telegram-beta"),
        ],
        poll_interval_ms: 20,
        per_chat_rate_limit: 4,
        per_chat_rate_window_seconds: 1,
        ..TelegramConfig::default()
    }
}

fn text_update(
    update_id: i64,
    bot_username: &str,
    chat_id: &str,
    text: &str,
    thread_id: Option<i64>,
) -> TelegramUpdate {
    TelegramUpdate {
        update_id,
        bot_username: Some(format!("@{bot_username}")),
        message: Some(TelegramMessage {
            message_id: update_id,
            chat_id: chat_id.to_string(),
            chat_type: ChatType::Group,
            from_user_id: Some("user-1".to_string()),
            from_username: Some("user_one".to_string()),
            bot_username: Some(format!("@{bot_username}")),
            text: Some(text.to_string()),
            message_thread_id: thread_id,
            reply_to_message_id: None,
            entities: Vec::new(),
            media: Vec::new(),
            inline_keyboard: None,
            reactions: Vec::new(),
            location: None,
            poll: None,
            sticker: None,
            quote: None,
        }),
        edited_message: None,
        callback_query: None,
        reaction: None,
        chat_member: None,
    }
}

fn callback_update(
    update_id: i64,
    bot_username: &str,
    chat_id: &str,
    data: &str,
) -> TelegramUpdate {
    TelegramUpdate {
        update_id,
        bot_username: Some(format!("@{bot_username}")),
        message: None,
        edited_message: None,
        callback_query: Some(TelegramCallbackQuery {
            id: format!("callback-{update_id}"),
            from_user_id: "user-1".to_string(),
            from_username: Some("user_one".to_string()),
            bot_username: Some(format!("@{bot_username}")),
            data: Some(data.to_string()),
            chat_id: Some(chat_id.to_string()),
            message_id: Some(update_id),
        }),
        reaction: None,
        chat_member: None,
    }
}

fn reaction_update(update_id: i64, chat_id: &str, message_id: i64) -> TelegramUpdate {
    TelegramUpdate {
        update_id,
        bot_username: Some("@bot_alpha".to_string()),
        message: None,
        edited_message: None,
        callback_query: None,
        reaction: Some(TelegramReactionUpdate {
            chat_id: chat_id.to_string(),
            message_id,
            actor_user_id: "user-1".to_string(),
            old_reactions: vec![],
            new_reactions: vec![TelegramReaction::Emoji("🔥".to_string())],
            counts: vec![TelegramReactionCount {
                reaction: TelegramReaction::Emoji("🔥".to_string()),
                count: 7,
            }],
        }),
        chat_member: None,
    }
}

fn sample_media(kind: TelegramMediaKind, file_id: &str, bytes: &[u8]) -> TelegramMedia {
    TelegramMedia {
        file_id: file_id.to_string(),
        kind,
        file_name: Some(format!("{file_id}.bin")),
        mime_type: Some("application/octet-stream".to_string()),
        file_path: None,
        url: None,
        bytes: bytes.to_vec(),
        duration_seconds: None,
        sticker_emoji: Some("🙂".to_string()),
        is_animated: false,
        is_video_note: false,
    }
}

async fn send_text_deliveries(channel: &TelegramChannel) -> Vec<TelegramDelivery> {
    channel
        .deliveries()
        .await
        .into_iter()
        .filter(|delivery| delivery.operation == TelegramOperation::SendText)
        .collect()
}

#[tokio::test]
async fn multi_account_concurrent_polling_processes_updates() {
    let mut channel = TelegramChannel::new(test_config());
    channel.start().await.unwrap();

    channel
        .ingest_update(
            "alpha",
            text_update(1, "bot_alpha", "chat-a", "hello alpha", None),
        )
        .await;
    channel
        .ingest_update(
            "beta",
            text_update(2, "bot_beta", "chat-b", "hello beta", None),
        )
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
    channel.stop().await.unwrap();

    let processed = channel.processed_updates().await;
    assert_eq!(processed.len(), 2);
    assert!(processed
        .iter()
        .any(|update| update.account_name == "alpha"));
    assert!(processed.iter().any(|update| update.account_name == "beta"));
}

#[tokio::test]
async fn routes_updates_by_bot_username_to_correct_account() {
    let channel = TelegramChannel::new(test_config());
    channel
        .ingest_routed_update(text_update(9, "bot_beta", "chat-b", "routed", None))
        .await
        .unwrap();

    let alpha = channel.get_updates("alpha", 10).await.unwrap();
    let beta = channel.get_updates("beta", 10).await.unwrap();

    assert!(alpha.is_empty());
    assert_eq!(beta.len(), 1);
    assert_eq!(
        beta[0].message.as_ref().and_then(|m| m.text.as_deref()),
        Some("routed")
    );
}

#[tokio::test]
async fn poll_once_retries_server_error_then_processes_updates() {
    let channel = TelegramChannel::new(test_config());
    channel
        .queue_poll_error("alpha", TelegramApiError::server("temporary 500"))
        .await;
    channel
        .ingest_update(
            "alpha",
            text_update(11, "bot_alpha", "chat-a", "after retry", None),
        )
        .await;

    let processed = channel.poll_once("alpha").await.unwrap();
    let health = channel.account_health("alpha").await.unwrap();

    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].kind, "message");
    assert_eq!(health.state, TelegramAccountHealthState::Connected);
}

#[tokio::test]
async fn poll_once_marks_auth_error_on_unauthorized() {
    let channel = TelegramChannel::new(test_config());
    channel
        .queue_poll_error("alpha", TelegramApiError::unauthorized("bad token"))
        .await;

    let error = channel.poll_once("alpha").await.unwrap_err();
    let health = channel.account_health("alpha").await.unwrap();

    assert!(error.to_string().contains("bad token"));
    assert_eq!(health.state, TelegramAccountHealthState::AuthError);
}

#[tokio::test]
async fn media_upload_download_roundtrip_is_preserved() {
    let channel = TelegramChannel::new(test_config());
    let bytes = b"telegram-photo-roundtrip";
    let media = sample_media(TelegramMediaKind::Photo, "photo-roundtrip", bytes);

    channel
        .send_photo(
            TelegramTarget::chat("media-chat"),
            media.clone(),
            Some("caption"),
        )
        .await
        .unwrap();

    let path = channel.download_media("photo-roundtrip").await.unwrap();
    let saved = fs::read(path).unwrap();

    assert_eq!(saved, bytes);
}

#[tokio::test]
async fn voice_duration_is_detected_from_payload_size() {
    let channel = TelegramChannel::new(test_config());
    let media = sample_media(TelegramMediaKind::Voice, "voice-clip", &vec![0u8; 48_000]);

    channel
        .send_voice(TelegramTarget::chat("voice-chat"), media, Some("voice"))
        .await
        .unwrap();

    let deliveries = channel.deliveries_for_chat("voice-chat").await;
    let voice = deliveries
        .into_iter()
        .find(|delivery| delivery.operation == TelegramOperation::SendVoice)
        .unwrap();
    assert!(voice.media[0].duration_seconds.unwrap() >= 3);
}

#[tokio::test]
async fn supports_inline_keyboard_callback_cycle() {
    let channel = TelegramChannel::new(test_config());
    let keyboard = TelegramInlineKeyboardMarkup {
        rows: vec![
            vec![
                TelegramInlineButton {
                    text: "Primary".to_string(),
                    callback_data: Some("approve".to_string()),
                    url: None,
                    switch_inline_query: None,
                    style: TelegramInlineButtonStyle::Primary,
                },
                TelegramInlineButton::url("Docs", "https://example.com"),
            ],
            vec![TelegramInlineButton {
                text: "Switch".to_string(),
                callback_data: None,
                url: None,
                switch_inline_query: Some("help".to_string()),
                style: TelegramInlineButtonStyle::Success,
            }],
        ],
    };

    channel
        .send_text_message(
            TelegramTarget::chat("callback-chat"),
            "choose",
            Some(ParseMode::Plain),
            Some(keyboard.clone()),
            None,
            false,
        )
        .await
        .unwrap();
    channel
        .ingest_update(
            "alpha",
            callback_update(21, "bot_alpha", "callback-chat", "approve"),
        )
        .await;
    let processed = channel.poll_once("alpha").await.unwrap();
    channel
        .answer_callback_query_with_options(
            "callback-21",
            TelegramCallbackAnswer {
                text: Some("Approved".to_string()),
                show_alert: true,
                url: Some("https://example.com/approved".to_string()),
            },
        )
        .await
        .unwrap();

    let deliveries = channel.deliveries().await;
    assert_eq!(processed[0].callback_data.as_deref(), Some("approve"));
    assert!(deliveries
        .iter()
        .any(|delivery| delivery.keyboard.as_ref() == Some(&keyboard)));
    assert!(deliveries
        .iter()
        .any(|delivery| delivery.operation == TelegramOperation::AnswerCallbackQuery));
}

#[tokio::test]
async fn markdown_and_html_entities_are_preserved_on_delivery() {
    let channel = TelegramChannel::new(test_config());

    channel
        .send_text_message(
            TelegramTarget::chat("format-chat"),
            "*bold* [link](https://example.com)",
            Some(ParseMode::Markdown),
            None,
            None,
            false,
        )
        .await
        .unwrap();
    channel
        .send_text_message(
            TelegramTarget::chat("format-chat"),
            "<b>bold</b><a href=\"https://example.com\">link</a>",
            Some(ParseMode::Html),
            None,
            None,
            false,
        )
        .await
        .unwrap();

    let deliveries = send_text_deliveries(&channel).await;
    assert_eq!(deliveries.len(), 2);
    assert_eq!(deliveries[0].entities.len(), 2);
    assert_eq!(deliveries[1].entities.len(), 2);
    assert_eq!(deliveries[0].entities[0].offset, 0);
    assert_eq!(
        deliveries[1].entities[1].url.as_deref(),
        Some("https://example.com")
    );
}

#[tokio::test]
async fn long_messages_are_split_with_continuation_markers() {
    let channel = TelegramChannel::new(test_config());
    channel
        .send_text_message(
            TelegramTarget::chat("split-chat"),
            &"split ".repeat(900),
            Some(ParseMode::Plain),
            None,
            None,
            false,
        )
        .await
        .unwrap();

    let deliveries = send_text_deliveries(&channel).await;
    assert!(deliveries.len() > 1);
    assert!(deliveries[0].text.as_deref().unwrap().starts_with("[1/"));
    assert!(deliveries
        .iter()
        .all(|delivery| delivery.text.as_ref().unwrap().len() <= 4096));
}

#[tokio::test]
async fn reactions_are_parsed_and_sent() {
    let channel = TelegramChannel::new(test_config());
    channel
        .ingest_update("alpha", reaction_update(31, "react-chat", 44))
        .await;
    channel.poll_once("alpha").await.unwrap();
    channel
        .set_message_reaction(
            TelegramTarget::chat("react-chat"),
            44,
            vec![
                TelegramReaction::Emoji("🔥".to_string()),
                TelegramReaction::CustomEmoji("custom-1".to_string()),
            ],
        )
        .await
        .unwrap();

    let counts = channel.reaction_counts("react-chat", 44).await;
    let deliveries = channel.deliveries().await;

    assert_eq!(counts[0].count, 1);
    assert!(deliveries.iter().any(|delivery| {
        delivery.operation == TelegramOperation::SetMessageReaction && delivery.reactions.len() == 2
    }));
}

#[tokio::test]
async fn auto_chat_actions_are_sent_before_messages_and_media() {
    let channel = TelegramChannel::new(test_config());
    channel
        .send_text_message(
            TelegramTarget::chat("typing-chat"),
            "hello",
            Some(ParseMode::Plain),
            None,
            None,
            false,
        )
        .await
        .unwrap();
    channel
        .send_photo(
            TelegramTarget::chat("typing-chat"),
            sample_media(TelegramMediaKind::Photo, "typing-photo", b"123"),
            None,
        )
        .await
        .unwrap();

    let deliveries = channel.deliveries_for_chat("typing-chat").await;
    assert_eq!(deliveries[0].operation, TelegramOperation::SendChatAction);
    assert_eq!(deliveries[0].chat_action, Some(TelegramChatAction::Typing));
    assert!(deliveries.iter().any(|delivery| {
        delivery.operation == TelegramOperation::SendChatAction
            && delivery.chat_action == Some(TelegramChatAction::UploadPhoto)
    }));
}

#[tokio::test]
async fn location_and_poll_requests_are_supported() {
    let channel = TelegramChannel::new(test_config());
    channel
        .send_location(
            TelegramTarget::chat("map-chat"),
            TelegramLocation {
                latitude: 40.1,
                longitude: -70.2,
                live_period_seconds: Some(120),
            },
        )
        .await
        .unwrap();
    channel
        .send_poll_request(
            TelegramTarget::chat("map-chat"),
            TelegramPollRequest {
                question: "Best spell?".to_string(),
                options: vec!["Fire".to_string(), "Ice".to_string()],
                kind: TelegramPollKind::Quiz,
                is_anonymous: false,
                correct_option_id: Some(0),
            },
        )
        .await
        .unwrap();

    let deliveries = channel.deliveries_for_chat("map-chat").await;
    assert!(deliveries
        .iter()
        .any(|delivery| delivery.operation == TelegramOperation::SendLocation));
    assert!(deliveries.iter().any(|delivery| {
        delivery.operation == TelegramOperation::SendPoll
            && delivery.poll.as_ref().unwrap().kind == TelegramPollKind::Quiz
    }));
}

#[tokio::test]
async fn forum_topics_and_threaded_messages_are_supported() {
    let channel = TelegramChannel::new(test_config());
    let topic = channel
        .create_forum_topic(
            TelegramTarget::chat("forum-chat"),
            "Incidents",
            Some("#112233"),
        )
        .await
        .unwrap();
    channel
        .send_text_message(
            TelegramTarget::chat("forum-chat").with_thread(topic.topic_id),
            "inside thread",
            Some(ParseMode::Plain),
            None,
            None,
            false,
        )
        .await
        .unwrap();

    let topics = channel.forum_topics("forum-chat").await;
    let threaded = send_text_deliveries(&channel).await;
    assert_eq!(topics.len(), 1);
    assert_eq!(threaded[0].thread_id, Some(topic.topic_id));
}

#[tokio::test]
async fn group_member_management_and_permissions_are_supported() {
    let channel = TelegramChannel::new(test_config());
    channel
        .seed_chat_member(
            "group-chat",
            TelegramChatMember {
                user_id: "user-9".to_string(),
                username: Some("user_nine".to_string()),
                status: TelegramMemberStatus::Member,
                can_send_messages: true,
                can_manage_topics: false,
                can_delete_messages: false,
                is_bot: false,
            },
        )
        .await;
    channel
        .set_bot_permissions(
            "group-chat",
            TelegramBotPermissions {
                can_send_messages: true,
                can_manage_topics: true,
                can_restrict_members: true,
                can_delete_messages: false,
            },
        )
        .await;
    channel
        .ban_member(TelegramTarget::chat("group-chat"), "user-9")
        .await
        .unwrap();
    let banned = channel
        .get_chat_member("group-chat", "user-9")
        .await
        .unwrap();
    channel
        .kick_member(TelegramTarget::chat("group-chat"), "user-9")
        .await
        .unwrap();
    let kicked = channel
        .get_chat_member("group-chat", "user-9")
        .await
        .unwrap();
    let granted = channel
        .bot_has_permissions(
            "group-chat",
            &TelegramBotPermissions {
                can_send_messages: true,
                can_manage_topics: true,
                can_restrict_members: true,
                can_delete_messages: false,
            },
        )
        .await;

    assert_eq!(banned.status, TelegramMemberStatus::Banned);
    assert_eq!(kicked.status, TelegramMemberStatus::Left);
    assert!(granted);
}

#[tokio::test]
async fn quote_forwarding_is_recorded() {
    let channel = TelegramChannel::new(test_config());
    channel
        .forward_message_with_quote(
            TelegramTarget::chat("quote-chat"),
            TelegramQuoteForward {
                source_chat_id: "source".to_string(),
                source_message_id: 88,
                quote: Some("quoted".to_string()),
            },
        )
        .await
        .unwrap();

    let deliveries = channel.deliveries_for_chat("quote-chat").await;
    assert_eq!(deliveries[0].operation, TelegramOperation::ForwardMessage);
    assert_eq!(
        deliveries[0]
            .quote_forward
            .as_ref()
            .unwrap()
            .quote
            .as_deref(),
        Some("quoted")
    );
}

#[tokio::test]
async fn blocked_users_and_flood_waits_are_handled() {
    let channel = TelegramChannel::new(test_config());
    channel.block_chat("blocked-chat").await;
    let blocked_error = channel
        .send_text_message(
            TelegramTarget::chat("blocked-chat"),
            "blocked",
            Some(ParseMode::Plain),
            None,
            None,
            false,
        )
        .await
        .unwrap_err();
    assert!(blocked_error.to_string().contains("blocked"));

    let start = Instant::now();
    channel.apply_rate_limit("alpha", Some(1)).await;
    channel.wait_rate_window("alpha").await;
    assert!(start.elapsed().as_millis() >= 900);
}

#[tokio::test]
async fn webhook_router_accepts_updates_and_polling_fallback_recovers() {
    let mut config = test_config();
    config.polling_mode = false;
    config.webhook_fallback_to_polling = true;
    let mut channel = TelegramChannel::new(config);
    channel
        .set_webhook(
            "alpha",
            "https://example.com/telegram",
            Some("secret-alpha"),
        )
        .await
        .unwrap();

    let router = webhook_router(Arc::new(channel.clone()));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/telegram/alpha")
                .header("x-telegram-bot-api-secret-token", "secret-alpha")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&text_update(61, "bot_alpha", "hook-chat", "webhook", None))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    channel.start().await.unwrap();
    channel.simulate_webhook_failure("alpha").await.unwrap();
    channel
        .ingest_update(
            "alpha",
            text_update(62, "bot_alpha", "hook-chat", "fallback", None),
        )
        .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
    channel.stop().await.unwrap();

    let processed = channel.processed_updates().await;
    assert!(processed.iter().any(|update| update.update_id == 61));
    assert!(processed.iter().any(|update| update.update_id == 62));
}

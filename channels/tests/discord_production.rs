#![cfg(feature = "discord")]

use std::time::Instant;

use magicmerlin_channels::discord::{
    session_scope, DiscordAttachment, DiscordChannel, DiscordCommandOption, DiscordConfig,
    DiscordEmbed, DiscordGatewayState, DiscordHello, DiscordInteraction, DiscordResponseKind,
};
use magicmerlin_channels::framework::{ChatType, OutboundMessage, ParseMode};
use magicmerlin_channels::Channel;

fn config() -> DiscordConfig {
    DiscordConfig {
        token: "token".to_string(),
        application_id: "app".to_string(),
        guild_allowlist: vec!["guild-1".to_string()],
        channel_allowlist: vec!["channel-a".to_string(), "channel-b".to_string()],
        dm_enabled: true,
    }
}

fn message(text: &str) -> OutboundMessage {
    OutboundMessage {
        text: text.to_string(),
        reply_to: None,
        media: Vec::new(),
        buttons: None,
        silent: false,
        parse_mode: Some(ParseMode::Markdown),
    }
}

#[tokio::test]
async fn gateway_identify_resume_and_health_progress() {
    let mut channel = DiscordChannel::new(config());
    channel.start().await.unwrap();
    channel
        .on_gateway_hello(DiscordHello {
            heartbeat_interval_ms: 45000,
        })
        .await;
    let identify = channel.identify().await.unwrap();
    channel.on_gateway_dispatch(42, Some("sess-42".to_string())).await;
    let resume = channel.resume().await.unwrap();
    let health = channel.health().await;

    assert_eq!(identify["op"], 2);
    assert_eq!(resume["op"], 6);
    assert_eq!(resume["d"]["session_id"], "sess-42");
    assert_eq!(health.last_sequence, Some(42));
    assert_eq!(health.state, DiscordGatewayState::Resuming);
}

#[tokio::test]
async fn slash_commands_and_interaction_lifecycle_work() {
    let channel = DiscordChannel::new(config());
    channel
        .register_slash_command("status", serde_json::json!({"name": "status"}))
        .await
        .unwrap();
    channel
        .queue_interaction(DiscordInteraction {
            id: "ix-1".to_string(),
            command_name: "status".to_string(),
            channel_id: "channel-a".to_string(),
            guild_id: Some("guild-1".to_string()),
            user_id: "user-1".to_string(),
            thread_id: Some("thread-9".to_string()),
            options: vec![DiscordCommandOption {
                name: "verbose".to_string(),
                value: "true".to_string(),
            }],
        })
        .await;

    let processed = channel.process_next_interaction().await.unwrap().unwrap();
    channel.defer_interaction("ix-1").await.unwrap();
    channel.respond_to_interaction("ix-1", "status ok").await.unwrap();
    channel.followup_interaction("ix-1", "details").await.unwrap();
    let responses = channel.interaction_responses().await;

    assert_eq!(processed.kind, "slash:status");
    assert_eq!(processed.session_scope, "discord:thread:thread-9");
    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0].kind, DiscordResponseKind::Deferred);
    assert_eq!(responses[1].kind, DiscordResponseKind::Immediate);
    assert_eq!(responses[2].kind, DiscordResponseKind::Followup);
}

#[tokio::test]
async fn send_edit_delete_react_and_history_are_supported() {
    let channel = DiscordChannel::new(config());
    let sent = channel
        .send_message(
            "channel-a",
            Some("guild-1"),
            "bot",
            message("hello discord"),
            vec![DiscordEmbed::builder().title("T").description("D").build()],
            vec![DiscordAttachment {
                id: "att-1".to_string(),
                filename: "note.txt".to_string(),
                content_type: Some("text/plain".to_string()),
                bytes: b"hello".to_vec(),
            }],
            None,
        )
        .await
        .unwrap();

    channel.edit_message(&sent, "edited").await.unwrap();
    channel.add_reaction(&sent, "🔥").await.unwrap();
    channel.add_reaction(&sent, "✅").await.unwrap();
    channel.remove_reaction(&sent, "🔥").await.unwrap();

    let history = channel
        .fetch_message_history("channel-a", Some("guild-1"), 10)
        .await
        .unwrap();
    let reactions = channel.reactions(&sent).await;

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].content, "edited");
    assert_eq!(history[0].attachments.len(), 1);
    assert_eq!(history[0].embeds.len(), 1);
    assert_eq!(reactions, vec!["✅".to_string()]);

    channel.delete_message(&sent).await.unwrap();
    let after_delete = channel
        .fetch_message_history("channel-a", Some("guild-1"), 10)
        .await
        .unwrap();
    assert!(after_delete.is_empty());
}

#[tokio::test]
async fn thread_creation_channel_listing_and_presence_work() {
    let channel = DiscordChannel::new(config());
    let thread_id = channel
        .create_thread("channel-a", Some("guild-1"), "ops")
        .await
        .unwrap();
    channel.update_presence("shipping discord parity").await.unwrap();
    channel
        .send_message(
            "channel-a",
            Some("guild-1"),
            "bot",
            message("inside thread"),
            Vec::new(),
            Vec::new(),
            Some(&thread_id),
        )
        .await
        .unwrap();
    let channels = channel.list_channels("guild-1").await.unwrap();
    let presence = channel.presence().await;
    let threads = channel.threads().await;

    assert!(channels.contains(&"channel-a".to_string()));
    assert_eq!(presence.activity, "shipping discord parity");
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id, thread_id);
}

#[tokio::test]
async fn message_splitting_and_typing_indicator_work() {
    let channel = DiscordChannel::new(config());
    let text = "x".repeat(4500);
    channel
        .send_message("channel-b", Some("guild-1"), "bot", message(&text), Vec::new(), Vec::new(), None)
        .await
        .unwrap();
    let history = channel
        .fetch_message_history("channel-b", Some("guild-1"), 10)
        .await
        .unwrap();
    let processed = channel.processed_events().await;

    assert!(history.len() >= 3);
    assert!(history.iter().all(|m| m.content.len() <= 2000));
    assert!(processed.iter().any(|event| event.kind == "typing"));
}

#[tokio::test]
async fn route_rate_limits_are_respected() {
    let channel = DiscordChannel::new(config());
    channel
        .respect_rate_limit("POST:/messages", Some(1), Some(0), Some(0.2))
        .await
        .unwrap();
    let now = Instant::now();
    channel
        .respect_rate_limit("POST:/messages", Some(1), Some(0), Some(0.2))
        .await
        .unwrap();
    assert!(now.elapsed().as_millis() >= 180);
}

#[test]
fn session_mapping_prefers_thread_then_guild_then_dm() {
    assert_eq!(
        session_scope(ChatType::Group, Some("guild-1"), "channel-a", Some("thread-1"), "user-1"),
        "discord:thread:thread-1"
    );
    assert_eq!(
        session_scope(ChatType::Group, Some("guild-1"), "channel-a", None, "user-1"),
        "discord:guild:guild-1:channel:channel-a"
    );
    assert_eq!(
        session_scope(ChatType::Direct, None, "dm-1", None, "user-9"),
        "discord:dm:user-9"
    );
}

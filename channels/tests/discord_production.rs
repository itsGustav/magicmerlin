#![cfg(feature = "discord")]

use std::time::Instant;

use magicmerlin_channels::discord::audit::{AuditLogAction, AuditLogChange, AuditLogQuery};
use magicmerlin_channels::discord::channel_mgmt::{
    ChannelType, CreateChannelParams, ModifyChannelParams,
};
use magicmerlin_channels::discord::components::{
    ActionRow, Button, ButtonStyle, ComponentInteraction, ComponentInteractionKind, Modal,
    SelectMenu, SelectOption, TextInput,
};
use magicmerlin_channels::discord::guild::{
    GuildInfo, GuildMember, OverwriteKind, PermissionOverwrite, Permissions, Role,
};
use magicmerlin_channels::discord::scheduled_events::{
    CreateEventParams, ScheduledEventEntityType, ScheduledEventStatus,
};
use magicmerlin_channels::discord::voice::VoiceState;
use magicmerlin_channels::discord::webhook::WebhookExecuteParams;
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

#[tokio::test]
async fn allowlists_dm_policy_and_mention_gate_are_enforced() {
    let channel = DiscordChannel::new(config());

    let guild_denied = channel
        .allows_inbound(ChatType::Group, Some("guild-x"), "channel-a", "user-1", Some("@magicmerlin hi"))
        .await;
    assert!(guild_denied.is_err());

    let channel_denied = channel
        .allows_inbound(ChatType::Group, Some("guild-1"), "channel-x", "user-1", Some("@magicmerlin hi"))
        .await;
    assert!(channel_denied.is_err());

    let mention_denied = channel
        .allows_inbound(ChatType::Group, Some("guild-1"), "channel-a", "user-1", Some("hello there"))
        .await;
    assert!(mention_denied.is_err());

    let allowed_group = channel
        .allows_inbound(ChatType::Group, Some("guild-1"), "channel-a", "user-1", Some("@magicmerlin hello"))
        .await;
    assert!(allowed_group.is_ok());

    channel.allow_dm_user("user-9").await;
    let allowed_dm = channel
        .allows_inbound(ChatType::Direct, None, "channel-a", "user-9", Some("hi"))
        .await;
    assert!(allowed_dm.is_ok());
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

// ---------------------------------------------------------------------------
// Pass 7 — Parity Pass 7 tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bulk_delete_removes_multiple_messages() {
    let channel = DiscordChannel::new(config());
    let m1 = channel
        .send_message("channel-a", Some("guild-1"), "bot", message("one"), Vec::new(), Vec::new(), None)
        .await
        .unwrap();
    let m2 = channel
        .send_message("channel-a", Some("guild-1"), "bot", message("two"), Vec::new(), Vec::new(), None)
        .await
        .unwrap();
    let _m3 = channel
        .send_message("channel-a", Some("guild-1"), "bot", message("three"), Vec::new(), Vec::new(), None)
        .await
        .unwrap();

    let deleted = channel.bulk_delete_messages(&[&m1, &m2]).await.unwrap();
    assert_eq!(deleted, 2);

    let history = channel
        .fetch_message_history("channel-a", Some("guild-1"), 10)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].content, "three");

    // Deleting nonexistent returns 0
    let deleted_again = channel.bulk_delete_messages(&[&m1]).await.unwrap();
    assert_eq!(deleted_again, 0);
}

#[tokio::test]
async fn pin_unpin_and_pinned_messages_work() {
    let channel = DiscordChannel::new(config());
    let m1 = channel
        .send_message("channel-a", Some("guild-1"), "bot", message("pin me"), Vec::new(), Vec::new(), None)
        .await
        .unwrap();
    let m2 = channel
        .send_message("channel-a", Some("guild-1"), "bot", message("pin me too"), Vec::new(), Vec::new(), None)
        .await
        .unwrap();

    channel.pin_message("channel-a", &m1).await.unwrap();
    channel.pin_message("channel-a", &m2).await.unwrap();

    let pinned = channel.pinned_messages("channel-a").await;
    assert_eq!(pinned.len(), 2);

    channel.unpin_message("channel-a", &m1).await.unwrap();
    let pinned_after = channel.pinned_messages("channel-a").await;
    assert_eq!(pinned_after.len(), 1);
    assert_eq!(pinned_after[0].content, "pin me too");

    // Pin nonexistent message fails
    let err = channel.pin_message("channel-a", "nonexistent").await;
    assert!(err.is_err());
}

#[tokio::test]
async fn message_components_buttons_and_select_menus() {
    let channel = DiscordChannel::new(config());

    let buttons = ActionRow::buttons(vec![
        Button::primary("btn-yes", "Yes"),
        Button::danger("btn-no", "No"),
        Button::link("https://example.com", "Docs"),
    ]);

    let select = ActionRow::select_menu(
        SelectMenu::string(
            "pick-color",
            vec![
                SelectOption::new("Red", "red"),
                SelectOption::new("Blue", "blue").with_description("Ocean blue"),
            ],
        )
        .with_placeholder("Pick a color"),
    );

    let msg_id = channel
        .send_message_with_components(
            "channel-a",
            Some("guild-1"),
            "bot",
            message("Choose:"),
            Vec::new(),
            Vec::new(),
            vec![buttons.clone(), select.clone()],
            None,
        )
        .await
        .unwrap();

    // Components should be tracked
    let attached = channel.components().components_for(&msg_id).await;
    assert!(attached.is_some());
    assert_eq!(attached.unwrap().len(), 2);

    // Simulate a button click
    channel
        .push_component_interaction(ComponentInteraction {
            id: "ci-1".to_string(),
            custom_id: "btn-yes".to_string(),
            kind: ComponentInteractionKind::Button,
            channel_id: "channel-a".to_string(),
            guild_id: Some("guild-1".to_string()),
            user_id: "user-1".to_string(),
            message_id: Some(msg_id.clone()),
            values: Vec::new(),
        })
        .await;

    let interaction = channel.pop_component_interaction().await;
    assert!(interaction.is_some());
    let ix = interaction.unwrap();
    assert_eq!(ix.custom_id, "btn-yes");
    assert_eq!(ix.kind, ComponentInteractionKind::Button);

    // No more interactions
    assert!(channel.pop_component_interaction().await.is_none());

    // Remove components
    channel.components().remove_components(&msg_id).await;
    assert!(channel.components().components_for(&msg_id).await.is_none());
}

#[tokio::test]
async fn modal_show_and_submit_flow() {
    let channel = DiscordChannel::new(config());

    let modal = Modal::new("feedback-form", "Feedback")
        .add_input(TextInput::short("name", "Your name").with_placeholder("Enter name"))
        .add_input(TextInput::paragraph("comment", "Comments").optional());

    channel.show_modal("user-1", modal).await;

    // Modal should be pending
    let pending = channel.components().pending_modal("user-1").await;
    assert!(pending.is_some());
    assert_eq!(pending.unwrap().title, "Feedback");

    // Submit the modal
    let submit = channel
        .components()
        .submit_modal(
            "modal-ix-1",
            "user-1",
            "channel-a",
            Some("guild-1"),
            vec!["Gustav".to_string(), "Great work!".to_string()],
        )
        .await;
    assert!(submit.is_some());
    let ix = submit.unwrap();
    assert_eq!(ix.kind, ComponentInteractionKind::ModalSubmit);
    assert_eq!(ix.custom_id, "feedback-form");
    assert_eq!(ix.values, vec!["Gustav", "Great work!"]);

    // Modal should be consumed
    assert!(channel.components().pending_modal("user-1").await.is_none());
}

#[tokio::test]
async fn guild_management_members_roles_bans_and_permissions() {
    let channel = DiscordChannel::new(config());
    let gm = channel.guilds();

    // Create a guild
    gm.upsert_guild(GuildInfo {
        id: "guild-1".to_string(),
        name: "Test Guild".to_string(),
        owner_id: "owner-1".to_string(),
        member_count: 3,
        icon: None,
        description: Some("Test guild".to_string()),
        features: vec!["COMMUNITY".to_string()],
    })
    .await;

    // Add @everyone role (id == guild_id)
    gm.add_role(
        "guild-1",
        Role {
            id: "guild-1".to_string(),
            name: "@everyone".to_string(),
            color: 0,
            position: 0,
            permissions: Permissions(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES),
            mentionable: false,
            hoist: false,
            managed: false,
        },
    )
    .await;

    // Add mod role
    gm.add_role(
        "guild-1",
        Role {
            id: "role-mod".to_string(),
            name: "Moderator".to_string(),
            color: 0x00FF00,
            position: 5,
            permissions: Permissions(Permissions::KICK_MEMBERS | Permissions::BAN_MEMBERS | Permissions::MANAGE_MESSAGES),
            mentionable: true,
            hoist: true,
            managed: false,
        },
    )
    .await;

    // Add admin role
    gm.add_role(
        "guild-1",
        Role {
            id: "role-admin".to_string(),
            name: "Admin".to_string(),
            color: 0xFF0000,
            position: 10,
            permissions: Permissions(Permissions::ADMINISTRATOR),
            mentionable: false,
            hoist: true,
            managed: false,
        },
    )
    .await;

    // Add members
    gm.add_member(
        "guild-1",
        GuildMember {
            user_id: "user-1".to_string(),
            nickname: Some("User One".to_string()),
            role_ids: vec!["role-mod".to_string()],
            joined_at: "2026-01-01".to_string(),
            deaf: false,
            mute: false,
        },
    )
    .await;

    gm.add_member(
        "guild-1",
        GuildMember {
            user_id: "user-2".to_string(),
            nickname: None,
            role_ids: vec!["role-admin".to_string()],
            joined_at: "2026-01-02".to_string(),
            deaf: false,
            mute: false,
        },
    )
    .await;

    gm.add_member(
        "guild-1",
        GuildMember {
            user_id: "user-3".to_string(),
            nickname: None,
            role_ids: vec![],
            joined_at: "2026-01-03".to_string(),
            deaf: false,
            mute: false,
        },
    )
    .await;

    // Owner has all permissions
    let owner_perms = gm.compute_base_permissions("guild-1", "owner-1").await;
    assert_eq!(owner_perms, Permissions::ALL);

    // Admin user has all permissions (via ADMINISTRATOR flag)
    let admin_perms = gm.compute_base_permissions("guild-1", "user-2").await;
    assert_eq!(admin_perms, Permissions::ALL);

    // Mod user has @everyone + mod role permissions
    let mod_perms = gm.compute_base_permissions("guild-1", "user-1").await;
    assert!(mod_perms.has(Permissions::VIEW_CHANNEL));
    assert!(mod_perms.has(Permissions::SEND_MESSAGES));
    assert!(mod_perms.has(Permissions::KICK_MEMBERS));
    assert!(mod_perms.has(Permissions::BAN_MEMBERS));
    assert!(mod_perms.has(Permissions::MANAGE_MESSAGES));
    assert!(!mod_perms.has(Permissions::ADMINISTRATOR));

    // Regular user only has @everyone permissions
    let regular_perms = gm.compute_base_permissions("guild-1", "user-3").await;
    assert!(regular_perms.has(Permissions::VIEW_CHANNEL));
    assert!(regular_perms.has(Permissions::SEND_MESSAGES));
    assert!(!regular_perms.has(Permissions::KICK_MEMBERS));

    // Ban user-3
    gm.ban("guild-1", "user-3", Some("spamming".to_string())).await;
    assert!(gm.is_banned("guild-1", "user-3").await);
    assert!(gm.member("guild-1", "user-3").await.is_none()); // removed from members

    // Unban
    gm.unban("guild-1", "user-3").await;
    assert!(!gm.is_banned("guild-1", "user-3").await);

    // Nickname update
    gm.update_nickname("guild-1", "user-1", Some("New Nick".to_string())).await;
    let member = gm.member("guild-1", "user-1").await.unwrap();
    assert_eq!(member.nickname.as_deref(), Some("New Nick"));

    // Role management on member
    gm.add_role_to_member("guild-1", "user-1", "role-admin").await;
    let member = gm.member("guild-1", "user-1").await.unwrap();
    assert!(member.role_ids.contains(&"role-admin".to_string()));

    gm.remove_role_from_member("guild-1", "user-1", "role-admin").await;
    let member = gm.member("guild-1", "user-1").await.unwrap();
    assert!(!member.role_ids.contains(&"role-admin".to_string()));

    // Guild info
    let info = gm.guild("guild-1").await.unwrap();
    assert_eq!(info.name, "Test Guild");
    assert_eq!(gm.all_guilds().await.len(), 1);

    // Role list
    assert_eq!(gm.roles("guild-1").await.len(), 3);
    gm.remove_role("guild-1", "role-mod").await;
    assert_eq!(gm.roles("guild-1").await.len(), 2);
}

#[tokio::test]
async fn channel_permission_overwrites_computed_correctly() {
    let channel = DiscordChannel::new(config());
    let gm = channel.guilds();

    gm.upsert_guild(GuildInfo {
        id: "guild-1".to_string(),
        name: "G".to_string(),
        owner_id: "owner".to_string(),
        member_count: 1,
        icon: None,
        description: None,
        features: Vec::new(),
    })
    .await;

    gm.add_role(
        "guild-1",
        Role {
            id: "guild-1".to_string(),
            name: "@everyone".to_string(),
            color: 0,
            position: 0,
            permissions: Permissions(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES),
            mentionable: false,
            hoist: false,
            managed: false,
        },
    )
    .await;

    gm.add_member(
        "guild-1",
        GuildMember {
            user_id: "user-1".to_string(),
            nickname: None,
            role_ids: vec![],
            joined_at: "2026-01-01".to_string(),
            deaf: false,
            mute: false,
        },
    )
    .await;

    // Deny SEND_MESSAGES for @everyone in channel-secret
    gm.set_channel_overwrites(
        "channel-secret",
        vec![PermissionOverwrite {
            target_id: "guild-1".to_string(),
            target_kind: OverwriteKind::Role,
            allow: Permissions::NONE,
            deny: Permissions(Permissions::SEND_MESSAGES),
        }],
    )
    .await;

    let perms = gm
        .compute_channel_permissions("guild-1", "channel-secret", "user-1")
        .await;
    assert!(perms.has(Permissions::VIEW_CHANNEL));
    assert!(!perms.has(Permissions::SEND_MESSAGES)); // denied by overwrite

    // Member-specific overwrite restores SEND_MESSAGES
    gm.set_channel_overwrites(
        "channel-secret",
        vec![
            PermissionOverwrite {
                target_id: "guild-1".to_string(),
                target_kind: OverwriteKind::Role,
                allow: Permissions::NONE,
                deny: Permissions(Permissions::SEND_MESSAGES),
            },
            PermissionOverwrite {
                target_id: "user-1".to_string(),
                target_kind: OverwriteKind::Member,
                allow: Permissions(Permissions::SEND_MESSAGES),
                deny: Permissions::NONE,
            },
        ],
    )
    .await;

    let perms2 = gm
        .compute_channel_permissions("guild-1", "channel-secret", "user-1")
        .await;
    assert!(perms2.has(Permissions::SEND_MESSAGES)); // restored by member overwrite
}

#[tokio::test]
async fn webhook_create_execute_edit_delete() {
    let channel = DiscordChannel::new(config());
    let wm = channel.webhooks();

    let wh = wm.create("channel-a", Some("guild-1"), "Deploy Bot").await;
    assert_eq!(wh.name.as_deref(), Some("Deploy Bot"));
    assert!(wh.token.is_some());

    // Execute webhook
    let msg = wm
        .execute(
            &wh.id,
            WebhookExecuteParams::text("Deployment complete!")
                .with_username("CI Bot")
                .with_avatar("https://example.com/avatar.png"),
        )
        .await
        .unwrap();
    assert_eq!(msg.content, "Deployment complete!");
    assert_eq!(msg.username.as_deref(), Some("CI Bot"));

    // Execute again
    let msg2 = wm
        .execute(&wh.id, WebhookExecuteParams::text("Another deploy"))
        .await
        .unwrap();

    // List messages
    let msgs = wm.messages_for(&wh.id).await;
    assert_eq!(msgs.len(), 2);

    // Edit message
    assert!(wm.edit_message(&msg.id, "Deployment v2 complete!").await);
    let all = wm.messages().await;
    let edited = all.iter().find(|m| m.id == msg.id).unwrap();
    assert_eq!(edited.content, "Deployment v2 complete!");

    // Delete message
    assert!(wm.delete_message(&msg2.id).await);
    assert_eq!(wm.messages_for(&wh.id).await.len(), 1);

    // Modify webhook
    wm.modify(&wh.id, Some("New Name".to_string()), None).await;
    let updated = wm.get(&wh.id).await.unwrap();
    assert_eq!(updated.name.as_deref(), Some("New Name"));

    // List by channel/guild
    assert_eq!(wm.list_for_channel("channel-a").await.len(), 1);
    assert_eq!(wm.list_for_guild("guild-1").await.len(), 1);

    // Delete webhook
    assert!(wm.delete(&wh.id).await);
    assert!(wm.get(&wh.id).await.is_none());
}

#[tokio::test]
async fn voice_state_tracking_join_move_leave() {
    let channel = DiscordChannel::new(config());
    let vt = channel.voice();

    // User joins voice channel
    vt.update(VoiceState {
        user_id: "user-1".to_string(),
        guild_id: "guild-1".to_string(),
        channel_id: Some("voice-1".to_string()),
        session_id: "sess-1".to_string(),
        deaf: false,
        mute: false,
        self_deaf: false,
        self_mute: false,
        self_stream: false,
        self_video: false,
        suppress: false,
    })
    .await;

    vt.update(VoiceState {
        user_id: "user-2".to_string(),
        guild_id: "guild-1".to_string(),
        channel_id: Some("voice-1".to_string()),
        session_id: "sess-2".to_string(),
        deaf: false,
        mute: false,
        self_deaf: false,
        self_mute: true,
        self_stream: false,
        self_video: false,
        suppress: false,
    })
    .await;

    // Check channel occupants
    let users = vt.users_in_channel("guild-1", "voice-1").await;
    assert_eq!(users.len(), 2);

    // user-2 is self-muted, not audible
    let state = vt.get("guild-1", "user-2").await.unwrap();
    assert!(!state.is_audible());

    // user-1 is audible
    let state = vt.get("guild-1", "user-1").await.unwrap();
    assert!(state.is_audible());
    assert!(state.is_connected());

    // Active channels
    let active = vt.active_channels("guild-1").await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].user_ids.len(), 2);

    // Move user-1 to voice-2
    vt.update(VoiceState {
        user_id: "user-1".to_string(),
        guild_id: "guild-1".to_string(),
        channel_id: Some("voice-2".to_string()),
        session_id: "sess-1".to_string(),
        deaf: false,
        mute: false,
        self_deaf: false,
        self_mute: false,
        self_stream: true,
        self_video: false,
        suppress: false,
    })
    .await;

    let active = vt.active_channels("guild-1").await;
    assert_eq!(active.len(), 2);
    assert_eq!(vt.total_voice_users().await, 2);

    // Disconnect user-1
    vt.update(VoiceState {
        user_id: "user-1".to_string(),
        guild_id: "guild-1".to_string(),
        channel_id: None,
        session_id: "sess-1".to_string(),
        deaf: false,
        mute: false,
        self_deaf: false,
        self_mute: false,
        self_stream: false,
        self_video: false,
        suppress: false,
    })
    .await;

    assert!(vt.get("guild-1", "user-1").await.is_none());
    assert_eq!(vt.total_voice_users().await, 1);

    // Clear guild
    vt.clear_guild("guild-1").await;
    assert_eq!(vt.total_voice_users().await, 0);
}

#[tokio::test]
async fn audit_log_record_and_query() {
    let channel = DiscordChannel::new(config());
    let al = channel.audit();

    // Record various actions
    al.record(
        "guild-1",
        AuditLogAction::MemberBanAdd,
        Some("mod-1"),
        Some("user-3"),
        Some("spamming"),
        vec![],
    )
    .await;

    al.record(
        "guild-1",
        AuditLogAction::ChannelCreate,
        Some("admin-1"),
        Some("channel-new"),
        None,
        vec![AuditLogChange {
            key: "name".to_string(),
            old_value: None,
            new_value: Some(serde_json::json!("new-channel")),
        }],
    )
    .await;

    al.record(
        "guild-1",
        AuditLogAction::MemberBanAdd,
        Some("mod-1"),
        Some("user-4"),
        Some("toxic"),
        vec![],
    )
    .await;

    // Query all
    let all = al.all("guild-1").await;
    assert_eq!(all.len(), 3);
    // Newest first
    assert_eq!(all[0].action, AuditLogAction::MemberBanAdd);

    // Query by action
    let bans = al
        .query("guild-1", AuditLogQuery::new().action(AuditLogAction::MemberBanAdd))
        .await;
    assert_eq!(bans.len(), 2);

    // Query by user
    let admin_actions = al
        .query("guild-1", AuditLogQuery::new().by_user("admin-1"))
        .await;
    assert_eq!(admin_actions.len(), 1);
    assert_eq!(admin_actions[0].action, AuditLogAction::ChannelCreate);

    // Query with limit
    let limited = al
        .query("guild-1", AuditLogQuery::new().with_limit(1))
        .await;
    assert_eq!(limited.len(), 1);

    // Count by action
    let counts = al.count_by_action("guild-1").await;
    assert_eq!(counts[&AuditLogAction::MemberBanAdd], 2);
    assert_eq!(counts[&AuditLogAction::ChannelCreate], 1);

    // Verify changes recorded
    let channel_creates = al
        .query("guild-1", AuditLogQuery::new().action(AuditLogAction::ChannelCreate))
        .await;
    assert_eq!(channel_creates[0].changes.len(), 1);
    assert_eq!(channel_creates[0].changes[0].key, "name");
}

#[tokio::test]
async fn channel_management_create_modify_delete() {
    let channel = DiscordChannel::new(config());
    let cm = channel.channel_mgmt();

    // Create text channel
    let text_ch = cm
        .create("guild-1", CreateChannelParams::text("general").with_topic("General chat"))
        .await;
    assert_eq!(text_ch.name, "general");
    assert_eq!(text_ch.kind, ChannelType::GuildText);
    assert_eq!(text_ch.topic.as_deref(), Some("General chat"));

    // Create voice channel
    let voice_ch = cm
        .create("guild-1", CreateChannelParams::voice("Music").with_user_limit(10))
        .await;
    assert_eq!(voice_ch.kind, ChannelType::GuildVoice);
    assert_eq!(voice_ch.user_limit, Some(10));

    // Create category
    let cat = cm
        .create("guild-1", CreateChannelParams::category("Text Channels"))
        .await;

    // Create channel in category
    let in_cat = cm
        .create(
            "guild-1",
            CreateChannelParams::text("dev").in_category(&cat.id).with_slowmode(5),
        )
        .await;
    assert_eq!(in_cat.parent_id.as_deref(), Some(cat.id.as_str()));

    // Create forum
    let forum = cm
        .create("guild-1", CreateChannelParams::forum("help-forum"))
        .await;
    assert_eq!(forum.kind, ChannelType::GuildForum);

    // List guild channels
    let all = cm.list_guild_channels("guild-1").await;
    assert_eq!(all.len(), 5);

    // List by type
    let text_channels = cm.list_by_type("guild-1", ChannelType::GuildText).await;
    assert_eq!(text_channels.len(), 2); // general + dev

    // List category children
    let children = cm.list_category_children(&cat.id).await;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "dev");

    // Modify channel
    let modified = cm
        .modify(
            &text_ch.id,
            ModifyChannelParams::new().name("announcements").nsfw(true),
        )
        .await
        .unwrap();
    assert_eq!(modified.name, "announcements");
    assert!(modified.nsfw);

    // Delete channel
    assert!(cm.delete(&voice_ch.id).await);
    assert!(cm.get(&voice_ch.id).await.is_none());
    assert_eq!(cm.list_guild_channels("guild-1").await.len(), 4);

    // Clear guild
    cm.clear_guild("guild-1").await;
    assert!(cm.list_guild_channels("guild-1").await.is_empty());
}

#[tokio::test]
async fn scheduled_events_create_lifecycle_and_interest() {
    let channel = DiscordChannel::new(config());
    let em = channel.events();

    // Create voice event
    let voice_evt = em
        .create(
            "guild-1",
            Some("user-1"),
            CreateEventParams::voice("Game Night", "voice-1", "2026-03-25T20:00:00Z")
                .with_description("Weekly game night"),
        )
        .await;
    assert_eq!(voice_evt.name, "Game Night");
    assert_eq!(voice_evt.entity_type, ScheduledEventEntityType::Voice);
    assert_eq!(voice_evt.status, ScheduledEventStatus::Scheduled);

    // Create external event
    let ext_evt = em
        .create(
            "guild-1",
            Some("user-1"),
            CreateEventParams::external(
                "Team Meetup",
                "Berlin Office",
                "2026-04-01T10:00:00Z",
                "2026-04-01T18:00:00Z",
            ),
        )
        .await;
    assert_eq!(ext_evt.entity_type, ScheduledEventEntityType::External);
    assert!(ext_evt.entity_metadata.is_some());

    // List events
    assert_eq!(em.list_guild_events("guild-1").await.len(), 2);

    // Add interested users
    em.add_interested(&voice_evt.id, "user-1").await;
    em.add_interested(&voice_evt.id, "user-2").await;
    em.add_interested(&voice_evt.id, "user-3").await;

    let interested = em.interested_users(&voice_evt.id).await;
    assert_eq!(interested.len(), 3);

    let event = em.get(&voice_evt.id).await.unwrap();
    assert_eq!(event.user_count, 3);

    // Remove interest
    em.remove_interested(&voice_evt.id, "user-2").await;
    assert_eq!(em.interested_users(&voice_evt.id).await.len(), 2);
    assert_eq!(em.get(&voice_evt.id).await.unwrap().user_count, 2);

    // Event lifecycle: start -> complete
    em.start(&voice_evt.id).await;
    assert_eq!(
        em.get(&voice_evt.id).await.unwrap().status,
        ScheduledEventStatus::Active
    );

    em.complete(&voice_evt.id).await;
    assert_eq!(
        em.get(&voice_evt.id).await.unwrap().status,
        ScheduledEventStatus::Completed
    );

    // Cancel the external event
    em.cancel(&ext_evt.id).await;
    assert_eq!(
        em.get(&ext_evt.id).await.unwrap().status,
        ScheduledEventStatus::Cancelled
    );

    // Modify event
    em.modify(&ext_evt.id, Some("Cancelled Meetup".to_string()), None)
        .await;
    assert_eq!(em.get(&ext_evt.id).await.unwrap().name, "Cancelled Meetup");

    // Delete event
    assert!(em.delete(&ext_evt.id).await);
    assert_eq!(em.list_guild_events("guild-1").await.len(), 1);
}

#[tokio::test]
async fn deeper_embeds_with_author_image_timestamp_and_url() {
    let channel = DiscordChannel::new(config());

    let embed = DiscordEmbed::builder()
        .title("Release v2.0")
        .url("https://example.com/release")
        .description("Major release with new features")
        .color(0x5865F2)
        .author("MagicMerlin Bot", Some("https://example.com".to_string()), None)
        .image("https://example.com/banner.png")
        .thumbnail("https://example.com/icon.png")
        .field("Features", "- Button support\n- Webhooks", false)
        .field("Bug Fixes", "42", true)
        .field("Contributors", "7", true)
        .footer("v2.0.0")
        .timestamp("2026-03-20T12:00:00Z")
        .build();

    assert_eq!(embed.url.as_deref(), Some("https://example.com/release"));
    assert_eq!(embed.image_url.as_deref(), Some("https://example.com/banner.png"));
    assert_eq!(embed.timestamp.as_deref(), Some("2026-03-20T12:00:00Z"));
    assert!(embed.author.is_some());
    let author = embed.author.as_ref().unwrap();
    assert_eq!(author.name, "MagicMerlin Bot");
    assert_eq!(author.url.as_deref(), Some("https://example.com"));

    // Send with rich embed
    let _msg_id = channel
        .send_message(
            "channel-a",
            Some("guild-1"),
            "bot",
            message("Check this out:"),
            vec![embed],
            Vec::new(),
            None,
        )
        .await
        .unwrap();

    let history = channel
        .fetch_message_history("channel-a", Some("guild-1"), 1)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    let e = &history[0].embeds[0];
    assert_eq!(e.fields.len(), 3);
    assert_eq!(e.fields[1].inline, true);
}

#[test]
fn button_constructors_and_styles() {
    let primary = Button::primary("id-1", "Click Me");
    assert_eq!(primary.style, ButtonStyle::Primary);
    assert_eq!(primary.custom_id.as_deref(), Some("id-1"));
    assert!(!primary.disabled);

    let secondary = Button::secondary("id-2", "Maybe");
    assert_eq!(secondary.style, ButtonStyle::Secondary);

    let success = Button::success("id-3", "Confirm");
    assert_eq!(success.style, ButtonStyle::Success);

    let danger = Button::danger("id-4", "Delete").with_emoji("🗑️").disabled();
    assert_eq!(danger.style, ButtonStyle::Danger);
    assert!(danger.disabled);
    assert_eq!(danger.emoji.as_deref(), Some("🗑️"));

    let link = Button::link("https://example.com", "Visit");
    assert_eq!(link.style, ButtonStyle::Link);
    assert!(link.custom_id.is_none());
    assert_eq!(link.url.as_deref(), Some("https://example.com"));
}

#[test]
fn select_menu_constructors() {
    let string_menu = SelectMenu::string(
        "pick",
        vec![
            SelectOption::new("A", "a").as_default(),
            SelectOption::new("B", "b").with_description("Option B"),
        ],
    )
    .with_placeholder("Choose one")
    .with_range(1, 2);

    assert_eq!(string_menu.options.len(), 2);
    assert!(string_menu.options[0].default);
    assert_eq!(string_menu.placeholder.as_deref(), Some("Choose one"));
    assert_eq!(string_menu.min_values, 1);
    assert_eq!(string_menu.max_values, 2);

    let user_menu = SelectMenu::user("pick-user");
    assert_eq!(user_menu.kind, magicmerlin_channels::discord::components::SelectMenuKind::User);

    let role_menu = SelectMenu::role("pick-role");
    assert_eq!(role_menu.kind, magicmerlin_channels::discord::components::SelectMenuKind::Role);

    let channel_menu = SelectMenu::channel("pick-channel");
    assert_eq!(
        channel_menu.kind,
        magicmerlin_channels::discord::components::SelectMenuKind::Channel
    );
}

#[test]
fn text_input_constructors() {
    let short = TextInput::short("name", "Your Name")
        .with_placeholder("Enter your name")
        .with_length(1, 100);
    assert!(short.required);
    assert_eq!(short.min_length, Some(1));
    assert_eq!(short.max_length, Some(100));

    let paragraph = TextInput::paragraph("bio", "About You")
        .optional()
        .with_value("Default bio");
    assert!(!paragraph.required);
    assert_eq!(paragraph.value.as_deref(), Some("Default bio"));
}

#[test]
fn permission_bitfield_operations() {
    let perms = Permissions(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES);
    assert!(perms.has(Permissions::VIEW_CHANNEL));
    assert!(perms.has(Permissions::SEND_MESSAGES));
    assert!(!perms.has(Permissions::ADMINISTRATOR));
    assert!(!perms.is_admin());

    let admin = Permissions(Permissions::ADMINISTRATOR);
    assert!(admin.is_admin());

    let combined = perms.add(Permissions::MANAGE_MESSAGES);
    assert!(combined.has(Permissions::MANAGE_MESSAGES));

    let reduced = combined.remove(Permissions::SEND_MESSAGES);
    assert!(!reduced.has(Permissions::SEND_MESSAGES));
    assert!(reduced.has(Permissions::VIEW_CHANNEL));

    let a = Permissions(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES);
    let b = Permissions(Permissions::SEND_MESSAGES | Permissions::MANAGE_MESSAGES);
    let union = a.union(b);
    assert!(union.has(Permissions::VIEW_CHANNEL));
    assert!(union.has(Permissions::SEND_MESSAGES));
    assert!(union.has(Permissions::MANAGE_MESSAGES));

    let inter = a.intersection(b);
    assert!(!inter.has(Permissions::VIEW_CHANNEL));
    assert!(inter.has(Permissions::SEND_MESSAGES));
    assert!(!inter.has(Permissions::MANAGE_MESSAGES));
}

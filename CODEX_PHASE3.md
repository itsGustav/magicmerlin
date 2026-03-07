# Phase 3: Channels — Build Instructions

You are building Magic Merlin, a 100% Rust replacement for OpenClaw. Phases 0-2 are complete (14 crates, 14.8K lines). Now build all chat channel integrations.

## What To Build — new crate: `channels`

Create a `channels` crate with a unified channel framework and implementations for all 8 platforms.

### 3.1 Channel Framework (channels/src/lib.rs + framework/)
1. **Channel trait**:
   ```rust
   #[async_trait]
   trait Channel: Send + Sync {
       fn name(&self) -> &str;
       fn platform(&self) -> Platform; // Telegram, Discord, WhatsApp, Signal, Slack, iMessage, LINE, Web
       async fn start(&mut self) -> Result<()>; // Begin listening
       async fn stop(&mut self) -> Result<()>;
       async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId>;
       async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()>;
       async fn delete(&self, target: &str, message_id: &str) -> Result<()>;
       async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()>;
   }
   ```

2. **Channel registry**: Register channels at startup, route messages by platform
3. **Inbound message normalization**: All platforms → unified `InboundMessage` struct:
   ```rust
   struct InboundMessage {
       id: String,
       platform: Platform,
       chat_id: String,
       chat_type: ChatType, // Direct, Group
       sender: Sender, // id, name, username
       text: Option<String>,
       reply_to: Option<String>,
       media: Vec<MediaAttachment>, // images, voice, documents
       timestamp: DateTime<Utc>,
       raw: Value, // platform-specific raw data
   }
   ```

4. **Outbound message formatting**:
   ```rust
   struct OutboundMessage {
       text: String,
       reply_to: Option<String>,
       media: Vec<MediaAttachment>,
       buttons: Option<Vec<Vec<InlineButton>>>,
       silent: bool,
       parse_mode: Option<ParseMode>, // Markdown, HTML, Plain
   }
   ```
   - Auto-split long messages per platform limits (Telegram 4096, Discord 2000, WhatsApp 65536)
   - Platform-specific markdown conversion

5. **DM policy enforcement**: open (anyone), pairing (approve first), allowlist (specific IDs only)
6. **Mention gating**: In groups, only process messages that @mention the bot
7. **Health monitoring**: Track connection status per channel, auto-reconnect on disconnect

### 3.2 Telegram (channels/src/telegram/)
Full Telegram Bot API integration:
1. Bot API client using reqwest (polling mode via getUpdates, webhook mode optional)
2. Message types: text, photo, voice, document, video, sticker, location
3. Send/edit/delete/react (setMessageReaction API)
4. Inline keyboards (buttons with callback_data)
5. Reply-to and quote support (reply_parameters)
6. Multiple bot accounts (one per agent — iterate config accounts)
7. Group chat: detect @mentions, handle /commands
8. Media download: getFile → download to local path
9. Media upload: sendPhoto, sendDocument, sendVoice, sendVideo
10. Typing indicator (sendChatAction)
11. Parse modes: MarkdownV2, HTML
12. Message effects (sendWithEffect)
13. Topic/forum support (message_thread_id)
14. Poll creation (sendPoll)

### 3.3 Discord (channels/src/discord/)
Discord bot integration:
1. Gateway WebSocket connection (identify, heartbeat, resume)
2. REST API client for messages, channels, guilds
3. Message send/edit/delete with embeds
4. Reactions (add/remove)
5. Thread creation and management
6. Slash command registration and handling
7. Voice channel awareness (not full voice, just presence)
8. Guild member/role management
9. Presence/activity status updates
10. Rate limit handling (per-route buckets)
11. Auto-reconnect with resume

### 3.4 WhatsApp (channels/src/whatsapp/)
WhatsApp Web integration:
1. Wrapper around external WhatsApp bridge process (like Baileys)
2. QR code pairing flow
3. Send/receive text messages
4. Media support (images, voice, documents)
5. Group chat support
6. Message reactions
7. Read receipts

### 3.5 Signal (channels/src/signal/)
Signal messenger integration:
1. Signal CLI wrapper (signal-cli or presage)
2. Send/receive messages
3. Group support
4. Media attachments
5. Reactions
6. Trust/safety number verification

### 3.6 Slack (channels/src/slack/)
Slack integration:
1. Web API client (chat.postMessage, conversations.*, users.*)
2. Socket Mode for real-time events
3. Block Kit message formatting
4. Channel/thread management
5. Slash commands
6. File upload
7. Emoji reactions
8. Rate limiting

### 3.7 iMessage (channels/src/imessage/)
macOS-only iMessage integration:
1. osascript/JXA bridge to Messages.app
2. Monitor for new messages (polling-based)
3. Send text messages
4. Group chat (named conversations)
5. Media: images only (Messages.app limitation)

### 3.8 Web Chat (channels/src/web/)
WebSocket-based web chat:
1. WebSocket server endpoint on gateway
2. Simple message protocol (JSON)
3. Session-based authentication
4. Media upload via HTTP endpoint
5. Typing indicators

### 3.9 LINE (channels/src/line/)
LINE Messaging API:
1. Webhook receiver for inbound messages
2. Reply/push message API
3. Flex Message builder
4. Rich menus
5. Media support

## Integration
- Add `channels` to workspace Cargo.toml
- Each platform is a feature flag: `telegram`, `discord`, `whatsapp`, `signal`, `slack`, `imessage`, `line`, `web`
- Default features: `telegram` (most used)
- Channel registry connects to auto-reply pipeline
- `cargo check`, `cargo test` must pass
- Commit after the framework + each major platform

## Quality Requirements
- Doc comments on all public items
- Error handling with thiserror
- No unwrap() in library code
- Tests: message normalization, platform formatting, DM policy, mention gating, message splitting
- Use `teloxide` for Telegram, `serenity` or custom for Discord

## Key Dependencies
- teloxide = "0.13" (Telegram)
- serenity = "0.12" (Discord) OR custom lightweight client
- reqwest (already available)
- tokio-tungstenite (already available)
- serde + serde_json (already available)

When completely finished, run:
openclaw system event --text "Phase 3 complete: Channels (8 platforms) built and tested" --mode now

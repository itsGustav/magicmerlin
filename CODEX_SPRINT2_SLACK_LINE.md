# Sprint 2 — Agent C: Slack + LINE Deepening

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Slack: `channels/src/slack/mod.rs` (193 lines, basic skeleton).
LINE: `channels/src/line/mod.rs` (169 lines, thin stub).

## Your Mission
Bring Slack to production quality and LINE to functional state.

---

## Part 1: Slack — Full Socket Mode Implementation

### Current state
193 lines — has config structs and a placeholder event loop, but no real Slack API calls.

### Add dependencies to channels/Cargo.toml
```toml
# Slack - use reqwest + serde (no heavy SDK needed)
# We'll implement the Slack API directly
```

### 1A: Slack HTTP API client
```rust
pub struct SlackApiClient {
    bot_token: String,   // xoxb-...
    http: reqwest::Client,
}

impl SlackApiClient {
    // POST https://slack.com/api/{method}
    async fn call(&self, method: &str, params: Value) -> Result<Value>;
    
    // Core methods needed:
    pub async fn chat_post_message(&self, channel: &str, text: &str, blocks: Option<Value>) -> Result<String>;
    pub async fn chat_update(&self, channel: &str, ts: &str, text: &str) -> Result<()>;
    pub async fn chat_delete(&self, channel: &str, ts: &str) -> Result<()>;
    pub async fn reactions_add(&self, channel: &str, ts: &str, name: &str) -> Result<()>;
    pub async fn reactions_remove(&self, channel: &str, ts: &str, name: &str) -> Result<()>;
    pub async fn conversations_list(&self) -> Result<Vec<SlackChannel>>;
    pub async fn users_info(&self, user_id: &str) -> Result<SlackUser>;
    pub async fn files_upload(&self, channel: &str, filename: &str, content: &[u8]) -> Result<String>;
}
```

### 1B: Socket Mode event loop
```rust
// Slack Socket Mode: connect to wss://wss-primary.slack.com/link/...
// Get WSS URL: POST https://slack.com/api/apps.connections.open (app-level token xapp-...)

pub struct SlackSocketMode {
    app_token: String,    // xapp-...
    bot_token: String,    // xoxb-...
    api: SlackApiClient,
}

impl SlackSocketMode {
    pub async fn connect(&self, tx: mpsc::Sender<InboundMessage>) -> Result<()> {
        // 1. POST apps.connections.open → get WSS URL
        // 2. Connect via tokio-tungstenite
        // 3. Handle incoming events:
        //    - message events → normalize to InboundMessage
        //    - hello → ack
        //    - disconnect → reconnect
        // 4. Send envelope ACK for each message received
    }
}
```

### 1C: Message normalization
```rust
// Slack event → InboundMessage
// Handle: message, app_mention, message_changed, message_deleted
// Support: threads (thread_ts), DMs (channel starting with D), channels (C), groups (G)
fn normalize_slack_event(event: &SlackEvent) -> Option<InboundMessage> {
    // Extract: user, channel, text, ts (as message_id), thread_ts
    // chat_type: Direct if channel.starts_with('D'), Group otherwise
    // Strip bot user ID from mentions: <@BOTID> text → text
}
```

### 1D: Blocks builder for outbound
```rust
// Convert plain text / Markdown to Slack blocks
pub fn text_to_blocks(text: &str) -> Value {
    // Simple: one section block with mrkdwn text
    json!({
        "blocks": [{
            "type": "section",
            "text": { "type": "mrkdwn", "text": text }
        }]
    })
}

// Convert markdown: **bold** → *bold*, `code` → `code`, etc.
pub fn format_for_slack(text: &str) -> String;
```

### 1E: Thread support
```rust
// When replying to a threaded message, pass thread_ts
pub async fn reply_in_thread(&self, channel: &str, thread_ts: &str, text: &str) -> Result<String>;
```

### 1F: Upgrade the existing 193-line stub
Replace placeholder bodies in `mod.rs` with real implementations using the above.

---

## Part 2: LINE — Full Messaging API

### Add dependencies to channels/Cargo.toml
```toml
# LINE - implement via HTTP directly (no Rust SDK needed)
```

### 2A: LINE Bot API client
```rust
pub struct LineApiClient {
    channel_access_token: String,
    http: reqwest::Client,
    base_url: String,  // https://api.line.me
}

impl LineApiClient {
    // Reply to webhook event
    pub async fn reply_message(&self, reply_token: &str, messages: Vec<LineMessage>) -> Result<()>;
    
    // Push to user/group (requires push quota)
    pub async fn push_message(&self, to: &str, messages: Vec<LineMessage>) -> Result<()>;
    
    // Get user profile
    pub async fn get_profile(&self, user_id: &str) -> Result<LineProfile>;
}

pub enum LineMessage {
    Text { text: String },
    Image { original_url: String, preview_url: String },
    Audio { url: String, duration_ms: u32 },
    Video { url: String, preview_url: String },
    Flex { alt_text: String, contents: Value },  // Flex Message
}
```

### 2B: Webhook server
LINE sends POST requests to your webhook URL:
```rust
// In gateway/src/main.rs or channels crate:
// POST /webhook/line → parse LINE webhook, dispatch to channel framework

pub async fn handle_line_webhook(
    payload: LineWebhookPayload,
    tx: mpsc::Sender<InboundMessage>,
) -> Result<()> {
    // Validate X-Line-Signature header (HMAC-SHA256 of payload with channel secret)
    // Parse events: message, follow, unfollow, join, leave, postback
    // Normalize message events to InboundMessage
    // Reply events need reply_token (expires in 30s)
}
```

### 2C: Message normalization
```rust
fn normalize_line_event(event: &LineEvent) -> Option<InboundMessage> {
    // LINE event types: message (text/image/audio/video/file/location/sticker)
    // source types: user, group, room
    // chat_type: Direct if source.type == "user", Group if "group"/"room"
}
```

### 2D: Flex message builder
```rust
pub fn text_to_flex(text: &str, alt_text: &str) -> LineMessage {
    // Build a simple bubble flex message for rich formatting
    LineMessage::Flex {
        alt_text: alt_text.to_string(),
        contents: json!({
            "type": "bubble",
            "body": {
                "type": "box",
                "layout": "vertical",
                "contents": [{
                    "type": "text",
                    "text": text,
                    "wrap": true
                }]
            }
        })
    }
}
```

### 2E: Upgrade the existing 169-line stub
Replace placeholder bodies with real implementations.

---

## Rules
- `cargo build --workspace` must pass clean
- Unit tests for: Slack message normalization, LINE webhook signature validation, blocks builder
- Graceful unavailable state if tokens not configured (log warning, don't panic)
- Both channels register in framework registry

## Completion

```bash
openclaw system event --text "Sprint 2 Agent C done: Slack Socket Mode full event loop, blocks builder, thread support, LINE Messaging API webhooks, Flex messages, both production-ready" --mode now
```

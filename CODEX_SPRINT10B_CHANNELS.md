# Sprint 10B — Channel depth: Discord full + Signal stub + WhatsApp wiring

## Goal
Deepen the three most-used channels to production-grade functionality:
1. **Discord** — add threads, embeds, slash commands, full event loop
2. **Signal** — integrate `presage` crate (basic link + send/receive)  
3. **WhatsApp** — wire the existing shell to actually send/receive via subprocess bridge
4. **iMessage** — wire the macOS osascript bridge (#[cfg(target_os = "macos")])

## Working directory
`~/Projects/magicmerlin`

## PART A — Discord full implementation

Read `channels/src/discord/runtime.rs` first (current ~832 lines).

### What to add to Discord:

**1. Thread support**
```rust
// Add to OutboundMessage: thread_id: Option<String>
// When thread_id is Some, send to that thread instead of channel
// Create thread: POST /channels/{channel.id}/threads
// Reply in thread: POST /channels/{thread_id}/messages
```

**2. Embeds**
```rust
// When message.text contains JSON-like embed spec, or message has media,
// build Discord embed object:
// { "embeds": [{ "title": "...", "description": "...", "color": 0x5865F2, "fields": [...] }] }
```

**3. Slash commands registration**
```rust
// On startup, register global application commands:
// POST https://discord.com/api/v10/applications/{app_id}/commands
// Commands: /status, /help, /model, /compact, /sessions
// Handle INTERACTION_CREATE events (type=2 APPLICATION_COMMAND)
// Respond with POST /interactions/{id}/{token}/callback
```

**4. Full event loop improvements**
- Handle `MESSAGE_UPDATE` (edit) events
- Handle `MESSAGE_DELETE` events  
- Handle `GUILD_MEMBER_ADD` / `GUILD_MEMBER_REMOVE`
- Handle `REACTION_ADD` / `REACTION_REMOVE`
- Send typing indicator: `POST /channels/{id}/typing`
- Message chunking: Discord 2000-char limit, split long messages

**5. Channel routing fixes**
- Support `#channel-name` target lookup (resolve name → ID via guild channels list)
- Support `@username` mentions (resolve name → ID via guild members list)
- DM support: `POST /users/@me/channels` to open DM, then send

### Implementation notes:
- Use `serenity` crate if it's already in the workspace, otherwise use raw HTTP with `reqwest`
- Auth header: `Bot {token}`
- Gateway URL: `wss://gateway.discord.gg/?v=10&encoding=json`

---

## PART B — Signal (presage stub — link + basic send/receive)

Signal is complex. Implement the minimum viable version:

Add to `channels/Cargo.toml` (feature-gated):
```toml
[features]
signal = ["dep:presage", "dep:presage-store-sled"]

[dependencies]
presage = { git = "https://github.com/whisperfish/presage", optional = true }
presage-store-sled = { git = "https://github.com/whisperfish/presage", optional = true }
```

If presage compile fails (it's complex), implement a subprocess bridge instead:
- Check if `signal-cli` is installed (`which signal-cli`)
- If yes: use subprocess JSON-RPC mode (`signal-cli -u <number> jsonRpc`)
- If no: return "Signal not configured: install signal-cli" in status

### Minimum Signal implementation:
```rust
pub struct SignalChannel {
    config: SignalConfig,
    // subprocess handle if using signal-cli
    subprocess: Option<Arc<Mutex<tokio::process::Child>>>,
}

impl SignalChannel {
    pub async fn start(&mut self) -> Result<()> {
        // Try signal-cli subprocess first (simpler)
        // Fall back to presage if available
    }
    
    pub async fn send(&self, recipient: &str, text: &str) -> Result<()> {
        // send via subprocess or presage
    }
    
    pub async fn poll_updates(&self) -> Result<Vec<InboundMessage>> {
        // receive from subprocess or presage
    }
}
```

---

## PART C — WhatsApp subprocess bridge

Read `channels/src/whatsapp/mod.rs` (current ~717 lines).

WhatsApp uses proprietary protocol. Best approach: subprocess bridge to `whatsmeow` Go binary.

### Option 1 (preferred): Check if `whatsapp-bridge` binary exists
```bash
which whatsapp-bridge  # homebrew or manual install
```

If exists, wire to it via stdin/stdout JSON protocol:
```
→ {"action":"send","jid":"1234567890@s.whatsapp.net","text":"Hello"}
← {"ok":true,"messageId":"xxx"}

→ {"action":"poll"}
← {"messages":[{"from":"xxx","text":"Hi back","timestamp":1234567890}]}
```

### Option 2 (fallback): Use WhatsApp Business API
If `WHATSAPP_PHONE_ID` and `WHATSAPP_TOKEN` env vars are set, use the official Cloud API:
```
POST https://graph.facebook.com/v18.0/{phone-id}/messages
Authorization: Bearer {token}
{"messaging_product":"whatsapp","to":"{number}","type":"text","text":{"body":"Hello"}}
```

Implement both — try subprocess first, fall back to Cloud API.

---

## PART D — iMessage (macOS only)

Read `channels/src/imessage/mod.rs` (current ~616 lines).

Wire the actual send/receive:

```rust
#[cfg(target_os = "macos")]
mod imessage_impl {
    use std::process::Command;
    
    pub fn send(recipient: &str, text: &str) -> std::io::Result<()> {
        // Use osascript to send via Messages.app
        let script = format!(
            r#"tell application "Messages"
                set targetService to 1st service whose service type = iMessage
                set targetBuddy to buddy "{recipient}" of targetService  
                send "{text}" to targetBuddy
            end tell"#
        );
        Command::new("osascript").arg("-e").arg(&script).output()?;
        Ok(())
    }
    
    pub fn read_chat_db(limit: usize) -> Vec<InboundMessage> {
        // Read from ~/Library/Messages/chat.db (SQLite)
        // SELECT message.text, handle.id, message.date, message.is_from_me
        // FROM message JOIN handle ON message.handle_id = handle.ROWID
        // WHERE message.is_from_me = 0 ORDER BY message.date DESC LIMIT {limit}
    }
}
```

---

## Build & Test

```bash
cargo build --release 2>&1 | tail -20
```

If `presage` fails to compile due to dependency issues, skip it and leave a `// TODO: presage` comment.
The build MUST succeed. Channels with compile errors should be feature-gated or stubbed.

```bash
git add -A
git commit -m "feat(channels): Discord threads/embeds/slash-cmds, Signal stub, WhatsApp bridge, iMessage macOS"
```

```bash
openclaw system event --text "Sprint 10B done: Discord full, Signal stub, WhatsApp bridge, iMessage wired" --mode now
```

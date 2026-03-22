# Sprint 2 — Agent B: iMessage + WhatsApp Hardening

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Channels: `channels/src/whatsapp/mod.rs` (178 lines, thin stub), `channels/src/imessage/` (absent).

## Your Mission
Two channels in one sprint:

---

## Part 1: iMessage (macOS only)

Create `channels/src/imessage/mod.rs`:

### Architecture
iMessage on macOS requires AppleScript via `osascript`. Implement a polling monitor + send.

```rust
// Feature gate for macOS only
#[cfg(target_os = "macos")]
pub mod imessage {
    pub struct IMessageChannel {
        poll_interval: Duration,   // default: 2s
        last_seen_id: Arc<Mutex<Option<String>>>,
        allowed_senders: HashSet<String>,  // phone numbers or emails
    }
}
```

### Send implementation
```rust
pub async fn send_message(to: &str, text: &str) -> Result<()> {
    // Uses osascript:
    // tell application "Messages"
    //   set targetService to 1st service whose service type = iMessage
    //   set targetBuddy to buddy "{to}" of targetService
    //   send "{text}" to targetBuddy
    // end tell
    let script = format!(r#"
        tell application "Messages"
            set targetService to 1st service whose service type = iMessage
            set targetBuddy to buddy "{}" of targetService
            send "{}" to targetBuddy
        end tell
    "#, to, text.replace('"', "\\\""));
    
    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await?;
    // check output.status
}
```

### Receive/monitor implementation
Poll for new messages via osascript:
```applescript
tell application "Messages"
    set recentChats to (every chat whose last modified date > (current date) - 30)
    repeat with aChat in recentChats
        -- get messages
    end repeat
end tell
```

OR use a simpler approach: tail the SQLite database at `~/Library/Messages/chat.db`:
```rust
// More reliable than AppleScript for monitoring
// chat.db has tables: message, handle, chat, chat_message_join
// Query: SELECT m.rowid, m.text, m.date, h.id FROM message m
//        JOIN handle h ON m.handle_id = h.rowid
//        WHERE m.date > ? AND m.is_from_me = 0
//        ORDER BY m.date ASC
```

Use the chat.db approach as primary (more reliable). AppleScript for sending only.

### Group chat support
chat.db `chat` table links to group chats via `chat_message_join`. Support group send:
```rust
pub async fn send_to_group(group_name: &str, text: &str) -> Result<()>;
```

### Register in framework
```rust
// channels/src/lib.rs
#[cfg(target_os = "macos")]
pub mod imessage;
```

---

## Part 2: WhatsApp Hardening

Current state: `channels/src/whatsapp/mod.rs` is 178 lines — structure only, no real implementation.

### Strategy: subprocess wrapper around `go-whatsapp-cli` or `whatsmeow`

Check if any of these are installed at runtime:
1. `whatsapp-bridge` (custom binary wrapping whatsmeow)  
2. `wacli` (from the wacli skill — check if `/opt/homebrew/bin/wacli` exists)

If none available: fall back to a QR-pairing WebSocket bridge:

```rust
pub struct WhatsAppChannel {
    runtime: WhatsAppRuntime,
}

pub enum WhatsAppRuntime {
    WaCli(WaCliRuntime),        // wacli subprocess
    Bridge(BridgeRuntime),       // custom bridge binary
    Unavailable,                 // graceful degradation
}

pub struct WaCliRuntime {
    binary: PathBuf,
    config_dir: PathBuf,
}
```

### wacli integration (primary)
```rust
impl WaCliRuntime {
    // Send: wacli send --to "+1234567890" --message "hello"
    pub async fn send(&self, to: &str, text: &str) -> Result<()> {
        tokio::process::Command::new(&self.binary)
            .args(["send", "--to", to, "--message", text])
            .output().await?;
    }
    
    // Monitor: wacli receive --output json (reads JSON lines from stdout)
    pub async fn receive_loop(&self, tx: mpsc::Sender<InboundMessage>) -> Result<()> {
        let mut child = tokio::process::Command::new(&self.binary)
            .args(["receive", "--output", "json"])
            .stdout(Stdio::piped())
            .spawn()?;
        
        // Read JSON lines, parse, send to tx
        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            if let Ok(msg) = serde_json::from_str::<WaMessage>(&line) {
                tx.send(msg.into_inbound()).await?;
            }
        }
    }
}
```

### Upgrade the existing 178-line stub
Replace the existing stub's unimplemented bodies with the real WhatsAppRuntime dispatch. Keep the existing config struct, just implement the methods.

### QR pairing
```rust
// When first connecting, wacli will print a QR code to stdout.
// Capture it and relay to the user via message tool or log.
pub async fn pair_qr(&self) -> Result<String> {
    // Returns the QR code as ASCII art or PNG path
}
```

---

## Rules
- Both channels must register in `channels/src/framework/registry.rs`
- `cargo build --workspace` must pass on macOS (iMessage) — use `#[cfg(target_os = "macos")]` guards
- Unit tests for: message normalization, AppleScript script generation, wacli command building
- Graceful degradation if runtime deps missing (log warning, mark channel as unavailable)

## Completion

```bash
openclaw system event --text "Sprint 2 Agent B done: iMessage channel (chat.db monitor + osascript send), WhatsApp hardened (wacli integration + QR pairing), both registered in framework" --mode now
```

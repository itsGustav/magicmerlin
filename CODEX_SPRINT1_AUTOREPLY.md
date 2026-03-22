# Sprint 1 — Agent B: Auto-Reply Depth + Agent Engine Injection

## Context
Magic Merlin is a Rust-first OpenClaw-compatible agent runtime at ~/Projects/magicmerlin.
The `auto-reply` crate is at `auto-reply/src/lib.rs` — currently 435 lines, mostly structure.
The `agent` crate is at `agent/src/` — system_prompt.rs is the key file for workspace injection.

## Your Mission
Two parallel tracks:
1. Deep auto-reply pipeline (slash commands, formatting, gating, suppression)
2. Full workspace file injection into agent system prompt

---

## Track 1: Auto-Reply Pipeline

### 1A. Full Slash Command Parser

Expand the existing `parse_slash_command` function to handle ALL 30+ slash commands:

```rust
pub enum SlashCommand {
    // Status & info
    Status,
    Version,
    Help { topic: Option<String> },
    
    // Model control
    Model { name: Option<String> },
    Reasoning { on: Option<bool> },
    Thinking { level: Option<String> },
    Verbose { on: Option<bool> },
    
    // Session management
    Compact,
    Session { key: Option<String> },
    Sessions,
    Memory { query: Option<String> },
    
    // Cron management
    Cron { action: Option<String>, args: Vec<String> },
    
    // Admin
    Approve { code: String, mode: Option<String> },  // /approve <code> [allow-once|allow-always|deny]
    Logs { tail: Option<u32> },
    Debug,
    Reset,
    
    // Channel-specific
    Pause,
    Resume,
    
    // Agent management
    Agents,
    Spawn { prompt: String },
    Kill { session: String },
    
    // Misc
    Ping,
    NoReply,  // internal: reply was NO_REPLY sentinel
    HeartbeatOk,  // internal: reply was HEARTBEAT_OK sentinel
    
    // Unknown
    Unknown { name: String, args: Vec<String> },
}
```

Parse from raw message text: `/status`, `/model sonnet`, `/approve abc123 allow-always`, etc.

### 1B. NO_REPLY and HEARTBEAT_OK Detection

Add to the reply formatting pipeline:

```rust
pub fn is_silent_reply(text: &str) -> bool {
    let t = text.trim();
    t == "NO_REPLY" || t == "HEARTBEAT_OK"
}

pub fn is_heartbeat_ok(text: &str) -> bool {
    text.trim() == "HEARTBEAT_OK"
}
```

In the auto-reply dispatch: if agent returns a silent reply, DO NOT send it to the channel. Log internally at debug level.

### 1C. Telegram MarkdownV2 Escaper

OpenClaw uses Telegram's MarkdownV2 mode. Implement a complete escaper:

```rust
/// Escape all special chars for Telegram MarkdownV2.
/// Special chars: _ * [ ] ( ) ~ ` > # + - = | { } . !
/// EXCEPT when they're part of valid formatting (bold, italic, code, links).
pub fn escape_telegram_v2(text: &str) -> String {
    // Simple approach: escape all special chars in plain text segments
    // Don't escape inside: *bold*, _italic_, `code`, ```block```, [text](url)
    let special = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
    // Parse markdown segments and escape only plain text parts
    todo!("implement state machine parser")
}
```

Also implement:
- `format_for_telegram(text: &str) -> String` — convert standard Markdown to Telegram MarkdownV2
- `format_for_discord(text: &str) -> String` — Discord uses standard Markdown, mostly pass-through
- `format_for_whatsapp(text: &str) -> String` — WhatsApp: *bold*, _italic_, ~strike~, `code`, no headers
- `format_for_platform(text: &str, platform: Platform) -> String` — dispatcher

### 1D. Collect/Debounce Window

Implement the message collect window:

```rust
pub struct CollectWindow {
    pending: Vec<InboundMessage>,
    deadline: Option<Instant>,
    window_ms: u64,  // default: 500ms
}

impl CollectWindow {
    /// Add a message. Returns Some(batch) if window expired, None if still collecting.
    pub fn add(&mut self, msg: InboundMessage) -> Option<Vec<InboundMessage>>;
    
    /// Check if window expired. Returns Some(batch) if yes.
    pub fn check_expiry(&mut self) -> Option<Vec<InboundMessage>>;
}
```

When a message arrives, start a 500ms window. If another message arrives within the window, extend/batch. After window expires, deliver all messages to agent as a single turn.

### 1E. Authorized Sender Enforcement

```rust
pub struct DmGate {
    policy: DmPolicy,
    allowlist: HashSet<String>,  // user IDs or chat IDs
    paired: HashSet<String>,      // paired via approval flow
}

impl DmGate {
    /// Returns true if the sender is allowed to trigger an agent turn.
    pub fn is_allowed(&self, sender_id: &str, chat_type: ChatType) -> bool {
        match self.policy {
            DmPolicy::Open => true,
            DmPolicy::Allowlist => self.allowlist.contains(sender_id),
            DmPolicy::Pairing => self.paired.contains(sender_id) || self.allowlist.contains(sender_id),
        }
    }
}
```

### 1F. Reply-To Construction

```rust
pub struct ReplyRef {
    pub message_id: String,
    pub quote_text: Option<String>,
}

/// Extract [[reply_to_current]] or [[reply_to:<id>]] tag from reply text.
/// Returns (cleaned_text, Option<ReplyRef>).
pub fn extract_reply_tag(text: &str) -> (String, Option<ReplyRef>);
```

The agent output may start with `[[reply_to_current]]` or `[[reply_to:12345]]`. Strip the tag before sending and use it to set the reply_to parameter in the channel dispatch.

---

## Track 2: Agent System Prompt — Full Workspace Injection

In `agent/src/system_prompt.rs`, the system prompt builder needs to inject ALL workspace files properly.

### 2A. Full workspace file injection

Implement `inject_workspace_files(prompt: &mut SystemPromptBuilder, workspace: &Path)`:

For each file in this priority order, read and inject with proper headers:
1. `AGENTS.md` — inject as `## /path/to/AGENTS.md\n{content}`
2. `SOUL.md`
3. `USER.md` 
4. `IDENTITY.md`
5. `TOOLS.md`
6. `HEARTBEAT.md`
7. `MEMORY.md` — **truncate to 14,000 chars max** with `…(truncated MEMORY.md: kept N chars)…` suffix
8. `memory/YYYY-MM-DD.md` for today and yesterday — inject if exist

Rules:
- Skip files that don't exist (no error)
- Truncate each file to **4,000 chars max** (except MEMORY.md which gets 14,000)
- Add a `## Project Context\n` section header before the workspace files block
- Add `[MISSING] Expected at: {path}` for BOOTSTRAP.md if it doesn't exist

### 2B. Skills injection

Implement `inject_skills(prompt: &mut SystemPromptBuilder, skills_dirs: &[PathBuf])`:

1. Scan each skills dir for subdirs containing `SKILL.md`
2. For each skill, read the `name`, `description` from frontmatter
3. Build the XML block:
```xml
<available_skills>
  <skill>
    <name>weather</name>
    <description>Get current weather...</description>
    <location>/path/to/skills/weather/SKILL.md</location>
  </skill>
  ...
</available_skills>
```
4. Inject into system prompt after the workspace files block

### 2C. Reply tag instructions

Inject a static `## Reply Tags` section into the system prompt explaining:
- `[[reply_to_current]]` — replies to triggering message
- `[[reply_to:<id>]]` — replies to specific message id
- Must be first token in response

### 2D. Silent reply instructions

Inject `## Silent Replies` section:
```
When you have nothing to say, respond with ONLY: NO_REPLY
Rules:
- It must be your ENTIRE message — nothing else
- Never append it to an actual response
```

And HEARTBEAT_OK instructions for heartbeat context.

---

## Implementation Rules

1. Every new function must have a `#[cfg(test)]` unit test
2. `cargo build --workspace` must pass with no errors
3. No `unwrap()` in production paths
4. Keep existing code that works — only ADD, don't break

## Completion

When all tasks are done and `cargo build --workspace` succeeds:

```bash
openclaw system event --text "Sprint 1 Agent B done: auto-reply slash commands(30+), NO_REPLY/HEARTBEAT_OK suppression, Telegram MarkdownV2 escaper, collect/debounce window, authorized sender gate, reply-to tags, full workspace file injection, skills XML injection" --mode now
```

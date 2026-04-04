# Sprint 9B — Wire Telegram Channel → AgentEngine (end-to-end message loop)

## Goal
Wire the Telegram channel runtime (`channels/src/telegram/runtime.rs`) so that inbound Telegram messages 
flow through auto-reply policy → AgentEngine → back to Telegram. After this sprint, sending a message 
to the configured Telegram bot must produce a real LLM reply in the chat, not silence.

Also: fix the 5 most impactful stub gaps in `agent-tools/src/tools.rs`:
1. `web_fetch` — real HTML extraction
2. `sessions_spawn` — ACP runtime wiring
3. `tts` — audio output
4. `image_generate` — generation pipeline stub → real gateway call
5. `session_status` — real data from session + model config

## Working directory
`~/Projects/magicmerlin`

## Step 1 — Read first
Read these files before writing:
- `channels/src/telegram/runtime.rs` — `TelegramChannel`, `recv_updates`, `handle_update`, `send_message`
- `auto-reply/src/lib.rs` — `AutoReplyEngine`, `evaluate_inbound`, `parse_slash_command`, `DmGate`
- `gateway/src/main.rs` — how channels are initialized, the event loop, `AppState`
- `agent-tools/src/tools.rs` — stubs for WebFetchTool, TtsTool, SessionsSpawnTool, SessionStatusTool

## Step 2 — Telegram inbound event loop

In `gateway/src/main.rs` (or a new `gateway/src/channel_loop.rs`), add a background tokio task that:

1. Initializes `TelegramChannel` from config (`config.channels.telegram`)
2. Starts polling: call `telegram_channel.start_polling()` or equivalent
3. In a loop: `telegram_channel.recv_update()` (or drain pending updates)
4. For each inbound update with text:
   a. Build `InboundMessage { channel: "telegram", user_id, chat_id, text, is_dm, mentioned, priority: 1 }`
   b. Run through `AutoReplyEngine::evaluate_inbound()`
   c. If `PipelineDecision::Queue { session_key }`:
      - Enqueue an `agent.run` call to `run_agent_turn(state, "telegram", params)`
      - Send reply back via `telegram_channel.send_message(chat_id, reply_text)`
   d. If `PipelineDecision::Command(cmd)`:
      - Handle locally (e.g., `/status` → send status card back)
   e. If `PipelineDecision::Ignore`: skip

Spawn this task in the gateway startup `main()` after state is built:
```rust
let channel_state = state.clone();
tokio::spawn(async move {
    run_telegram_loop(channel_state).await;
});
```

## Step 3 — DmGate / Authorized Sender enforcement

Wire `DmGate` from `auto-reply`:
- Read `config.channels.telegram.accounts[*].allowFrom` as the allowlist
- Build `DmGate` with `DmPolicy::Allowlist` if allowFrom is set, else `DmPolicy::Open`
- Call `gate.is_allowed(sender_id, chat_type)` before queuing any agent turn
- If not allowed: send "⛔ You're not authorized to use this bot." and skip

## Step 4 — HEARTBEAT_OK and NO_REPLY suppression

After getting the agent reply string:
```rust
let reply = reply.text.trim();
if reply == "HEARTBEAT_OK" || reply == "NO_REPLY" {
    // Do not send anything to Telegram
    continue;
}
// Also strip [[reply_to_current]] and [[reply_to:<id>]] tags (use auto_reply::extract_reply_tag)
let (clean_reply, reply_ref) = magicmerlin_auto_reply::extract_reply_tag(reply);
// Send clean_reply, optionally as a reply_to if reply_ref is Some
```

## Step 5 — Fix WebFetchTool

In `agent-tools/src/tools.rs`, find `WebFetchTool::execute`. Replace the stub with:

```rust
async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
    let url = required_string(&params, "url", self.name())?;
    let max_chars = params.get("maxChars").and_then(Value::as_u64).unwrap_or(10_000) as usize;
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (compatible; MagicMerlin/1.0)")
        .build()
        .map_err(|e| ToolError::Execution(e.to_string()))?;
    
    let resp = client.get(&url).send().await.map_err(|e| ToolError::Execution(e.to_string()))?;
    let status = resp.status().as_u16();
    let content_type = resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = resp.text().await.map_err(|e| ToolError::Execution(e.to_string()))?;
    
    // HTML → plain text extraction (simple approach: strip tags)
    let text = if content_type.contains("html") || body.trim_start().starts_with('<') {
        html_to_text(&body)
    } else {
        body
    };
    
    let truncated = if text.len() > max_chars {
        format!("{}...[truncated]", &text[..max_chars])
    } else {
        text
    };
    
    Ok(ToolResult::success(json!({
        "url": url,
        "status": status,
        "content": truncated,
        "chars": truncated.len(),
    })))
}
```

Add a simple `html_to_text` helper that:
1. Removes `<script>`, `<style>` blocks and their content (regex or manual scan)
2. Strips all remaining HTML tags (`<[^>]+>` → "")
3. Collapses whitespace
4. Returns clean text

Use `reqwest` (already likely in deps). Add to `agent-tools/Cargo.toml` if missing.

## Step 6 — Fix SessionStatusTool

Replace stub with a real implementation that returns:
```json
{
  "model": "current model from ctx.model_config",
  "sessionKey": "ctx.session_key",
  "contextTokens": "estimated from session transcript length",
  "contextMax": "120000",
  "costEstimate": null,
  "thinking": false,
  "reasoning": false
}
```

Use `ctx.session_key` and `ctx.gateway_url` to query the session from gateway if available, 
or compute from transcript length estimate.

## Step 7 — Fix TtsTool (basic implementation)

If `OPENAI_API_KEY` is set: call OpenAI `/v1/audio/speech` API with the text, save MP3 to 
a temp file in `ctx.workspace_dir/media/`, return the path.

If no API key: return a text result indicating TTS is not configured rather than panic.

```rust
async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
    let text = required_string(&params, "text", self.name())?;
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Ok(ToolResult::success(json!({
            "ok": false,
            "error": "TTS not configured: set OPENAI_API_KEY",
            "text": text,
        })));
    }
    // POST to https://api.openai.com/v1/audio/speech
    // model: tts-1, voice: alloy, input: text, response_format: mp3
    // Save to workspace_dir/media/<uuid>.mp3
    // Return {"ok": true, "path": "<path>", "format": "mp3"}
}
```

## Step 8 — Fix SessionsSpawnTool

Replace stub with a real gateway call:
```rust
async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
    // Call gateway: sessions.spawn with the full params
    gateway_call(ctx, "sessions.spawn", params).await
}
```

## Step 9 — Compile clean

```bash
cargo build --release 2>&1 | tail -40
```

Fix all errors. No new `unimplemented!()` or `todo!()`.

## Step 10 — Integration test

1. Start the gateway with a real Telegram bot token (from env `TELEGRAM_BOT_TOKEN`):
```bash
TELEGRAM_BOT_TOKEN=<token> ./target/release/magicmerlin-gateway --serve 19002 &
sleep 3
```

2. Send a test message to the bot via Telegram API:
```bash
# Simulate inbound update via the /call endpoint
curl -s -X POST http://127.0.0.1:19002/call \
  -H "Content-Type: application/json" \
  -d '{"method":"agent.run","params":{"session_id":"tg:test","message":"Hello, say the word PINEAPPLE","timeout_seconds":30}}'
```

The reply must contain "PINEAPPLE" (or similar real LLM response), not the echo stub.

3. Kill: `pkill -f "magicmerlin-gateway.*19002"`

## Step 11 — Commit

```bash
git add -A
git commit -m "feat(channels): wire Telegram inbound loop → AgentEngine; fix web_fetch, tts, sessions_spawn, session_status tools"
```

## When done

```bash
openclaw system event --text "Sprint 9B done: Telegram channel wired to AgentEngine end-to-end; web_fetch/tts/sessions_spawn/session_status fixed" --mode now
```

## Notes
- If Telegram config is missing/empty, log a warning and skip the channel loop (don't crash)
- DM policy: default to Open if no allowFrom configured
- Do not modify the test files (deep_*_matrix.rs)
- Keep all existing compat snapshot handling intact
- `reqwest` should already be in workspace deps; add it to `agent-tools/Cargo.toml` if missing

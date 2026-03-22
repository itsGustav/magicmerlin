# Sprint 1 — Agent A: Tool Completions

## Context
Magic Merlin is a Rust-first OpenClaw-compatible agent runtime at ~/Projects/magicmerlin.
All tools are in `agent-tools/src/tools.rs`. Currently 23 tools are registered but most are stubs.

## Your Mission
Implement REAL tool functionality for the 6 highest-priority tools listed below.
These are P0 blockers — nothing works end-to-end until these are real.

## Tool 1: `exec` — REAL PTY execution

The current `ExecTool` in tools.rs runs commands with `tokio::process::Command` but lacks:
- PTY support (needed for interactive CLIs)
- Background session management (sessionId → child process map)
- poll/log/write/send-keys/kill via the `ProcessTool`

### Implementation steps:

1. Add `portable-pty = "0.8"` to `agent-tools/Cargo.toml`
2. In `tools.rs`, upgrade `ExecTool::execute`:
   - When `background=true`: spawn process, store in a `Arc<Mutex<HashMap<String, BackgroundSession>>>` in ToolContext, return `{"sessionId": uuid, "pid": pid}`
   - When `pty=true` (and not background): allocate a PTY via `portable_pty`, run command, capture output, return when done
   - When `background=true` + `pty=true`: spawn in PTY, store session handle
   - Always support: `cmd`, `cwd`, `timeout_ms`, `env`, `workdir`
3. Upgrade `ProcessTool::execute` to handle all actions:
   - `list` → return all active sessions as JSON array
   - `poll` → check if session is still running, return `{"running": bool, "exitCode": int|null}`
   - `log` → return stdout/stderr captured so far, support `offset` and `limit` params
   - `write` → write raw data to stdin
   - `submit` → write data + "\n" to stdin
   - `send-keys` → send special key sequences (Ctrl-C = `\x03`, Enter = `\n`, etc.)
   - `kill` → SIGTERM the process
   - `paste` → write text, optionally wrapped in bracketed paste mode (`\x1b[200~...text...\x1b[201~`)

```rust
// BackgroundSession to store per sessionId:
struct BackgroundSession {
    pid: u32,
    stdin_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    log_buf: Arc<Mutex<Vec<u8>>>,
    exit_code: Arc<Mutex<Option<i32>>>,
}
```

## Tool 2: `memory_search` — Semantic embedding search

The current stub returns empty results. Implement real semantic search:

1. Add to `agent-tools/Cargo.toml`:
   ```toml
   fastembed = "3"
   ```
2. At startup (or first call), index these files:
   - `~/.openclaw/workspace-main/MEMORY.md`
   - `~/.openclaw/workspace-main/memory/*.md`
   - Any `MEMORY.md` in the current workspace
3. Chunk each file into ~200-word segments, embed with `fastembed` (AllMiniLML6V2 model)
4. On `memory_search { query, maxResults?, minScore? }`:
   - Embed the query
   - Cosine similarity against all chunks
   - Return top-N results as:
   ```json
   {
     "results": [
       {
         "path": "memory/2026-03-22.md",
         "startLine": 5,
         "endLine": 30,
         "score": 0.89,
         "snippet": "...text...",
         "source": "memory",
         "citation": "memory/2026-03-22.md#L5-L30"
       }
     ]
   }
   ```
5. Cache the index in a `Arc<RwLock<MemoryIndex>>` in ToolContext to avoid re-embedding on every call
6. Auto-reindex when files are modified (check mtime on each call, reindex if stale)

## Tool 3: `memory_get` — Safe snippet reader

Current stub just returns empty. Implement:

```rust
// Params: { path: String, from?: u32, lines?: u32 }
// - Validate path is under workspace dir (no path traversal)
// - Resolve relative paths against workspace root
// - Read file, extract lines [from .. from+lines] (1-indexed)
// - Return { content: String, path: String, totalLines: u32 }
```

## Tool 4: `message` tool — Channel dispatch

Current stub does nothing. Wire to channel registry:

The gateway has channel configs (Telegram bot tokens, etc.) in the config. The `message` tool needs to:

1. Accept params: `action` (send/react/delete/edit/poll), `channel`, `target`, `message`, `messageId`, `emoji`, etc.
2. For `action=send`: make HTTP call to gateway's `/call` method `channels.send` with the params
3. For `action=react`: call gateway's `channels.react`
4. Use `reqwest` to POST to `http://127.0.0.1:18789/call` (or whatever port is in config/env `MAGICMERLIN_GATEWAY_URL`)
5. Return the gateway response as the tool result

```rust
// In ToolContext, add:
pub gateway_url: String,  // e.g. "http://127.0.0.1:18789"

// MessageTool::execute:
let resp = ctx.http_client
    .post(format!("{}/call", ctx.gateway_url))
    .json(&json!({ "method": format!("channels.{}", action), "params": params }))
    .send().await?;
```

## Tool 5: `cron` tool — Scheduler dispatch

Same pattern as message tool — wire to gateway:

```rust
// CronTool handles actions: status/list/add/update/remove/run/runs/wake
// Map to gateway methods: cron.status, cron.list, cron.add, etc.
// POST to {gateway_url}/call with method + params
// Return gateway response
```

## Tool 6: `session_status` — Session info card

Return real data about the current session:

```rust
// Params: { sessionKey?: String, model?: String }
// Return:
{
  "sessionKey": "telegram:8527778539",
  "model": "anthropic/claude-sonnet-4-6",
  "contextTokens": 12400,
  "contextPercent": 15,
  "messageCount": 47,
  "startTime": "2026-03-22T08:17:00Z",
  "cost": null  // populate if cost tracking is available
}
// Get from: ctx.session_id, ctx.model_name, ctx.turn_count
// Token estimate: sum of all message content lengths / 4 (rough heuristic)
```

## Implementation Rules

1. Every tool must compile clean (`cargo build --workspace`)
2. Every tool must have at least one `#[cfg(test)]` unit test
3. Use `anyhow::Result` for error handling everywhere
4. No `unwrap()` in production paths — use `?` operator
5. No `todo!()` or `unimplemented!()` macros in paths touched by these tools
6. Keep existing tool stubs for tools NOT in this list — don't break them

## Completion

When all 6 tools are implemented and `cargo build --workspace` succeeds with no errors:

```bash
openclaw system event --text "Sprint 1 Agent A done: exec(PTY+background), memory_search(embeddings), memory_get, message, cron, session_status tools all implemented" --mode now
```

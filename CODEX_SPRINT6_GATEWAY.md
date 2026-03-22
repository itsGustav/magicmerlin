# Sprint 6 — Gateway Method Hardening (All 108 → Real Implementations)

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Gateway: `gateway/src/main.rs` (4,597 lines) + `gateway/src/methods/mod.rs` (176 lines).
Currently 108 methods registered but many return stub responses like `{"ok": true}` or empty arrays.

## Your Mission
Make every gateway method return real data. Prioritize by call frequency.

## Group 1: Agent methods (highest priority)

### `agent.run`
```rust
// Params: { message: String, sessionKey?: String, model?: String }
// - Look up or create session
// - Build context from transcript
// - Call provider with system prompt + context + message
// - Execute tool calls in loop
// - Save reply to transcript
// - Return: { reply: String, sessionKey: String, toolCallCount: u32, tokens: u32 }
// Note: This is the core agent turn. Wire to agent::Engine.
```

### `agent.status`
```rust
// Return: { agents: [{id, model, channels, last_active, session_count}] }
// Read from AgentRegistry
```

### `agent.model`
```rust
// Params: { agentId?: String, model?: String }
// GET: return current model for agent
// SET: update agent's model in config
```

---

## Group 2: Sessions methods

### `sessions.list`
```rust
// Return array of: { key, messageCount, lastActive, model, contextPercent }
// Scan state_dir/sessions/*.jsonl
// Estimate context% per session
```

### `sessions.show`
```rust
// Params: { sessionKey: String, limit?: u32 }
// Read transcript, return last N messages
// Return: { messages: [{role, content, timestamp}], total: u32 }
```

### `sessions.delete`
```rust
// Params: { sessionKey: String }
// Remove transcript file + lock file
```

### `sessions.compact`
```rust
// Params: { sessionKey: String }
// Run session compaction (already implemented in sessions crate)
// Return CompactionResult
```

### `sessions.history`
```rust
// Same as sessions.show but with more history context
```

### `sessions.send`
```rust
// Params: { sessionKey: String, message: String }
// Inject message into session (without triggering a new agent turn)
```

### `sessions.spawn`
```rust
// Params: { task: String, model?: String, agentId?: String, ... }
// Spawn a new isolated agent session
// Return: { sessionKey: String }
```

---

## Group 3: Channels methods

### `channels.list`
```rust
// Return: [{ name, platform, status: "connected"|"disconnected"|"error", lastMessage }]
// Query channel registry for status of each configured channel
```

### `channels.send`
```rust
// Params: { channel, target, message, ... }
// Dispatch through channel registry → real send
// Return: { ok: true, messageId: String }
```

### `channels.react`
```rust
// Params: { channel, messageId, emoji }
// Dispatch reaction through channel
```

### `channels.delete`
```rust
// Params: { channel, messageId }
```

### `channels.edit`
```rust
// Params: { channel, messageId, message }
```

---

## Group 4: Memory methods

### `memory.search`
```rust
// Params: { query: String, maxResults?: u32, minScore?: f32 }
// Call agent-tools memory_search implementation
// Return same format as memory_search tool
```

### `memory.get`
```rust
// Params: { path: String, from?: u32, lines?: u32 }
// Return file content snippet
```

---

## Group 5: Cron methods (most already wired to scheduler — verify and fix)

### `cron.list` → verify returns real jobs from SQLite
### `cron.add` → verify job persists and fires
### `cron.remove` → verify removes from DB
### `cron.run` → verify triggers immediate execution
### `cron.runs` → verify returns run history
### `cron.status` → return scheduler health (running, job count, next firing)
### `cron.wake` → inject system event into main session

---

## Group 6: Nodes methods

### `nodes.list`
```rust
// Return: [{ id, name, platform, lastSeen, paired }]
// Read from node-host registry or config
```

### `nodes.describe`
```rust
// Params: { node: String }
// Return full node info
```

### `nodes.run`
```rust
// Params: { node: String, command: [String] }
// Execute command on remote node via node-host protocol
```

### `nodes.notify`
```rust
// Params: { node: String, title: String, body: String }
// Send push notification to node
```

---

## Group 7: Browser methods

### `browser.status` → return running state, profile, tab count
### `browser.tabs` → return open tabs list
### `browser.snapshot` → delegate to media::browser
### `browser.screenshot` → delegate to media::browser
### `browser.act` → delegate to media::browser
### `browser.navigate` → delegate to media::browser

---

## Group 8: Gateway control methods

### `gateway.status`
```rust
// Return: { version, uptime, pid, port, bind, agents: u32, sessions: u32 }
```

### `gateway.restart`
```rust
// Send SIGUSR1 to self (triggers restart)
// Or: spawn new process then exit
```

### `config.get`
```rust
// Return full config as JSON (mask secrets)
```

### `config.patch`
```rust
// Params: { patch: Object }
// Deep merge patch into config, write to disk, return new config
```

---

## Group 9: Subagents methods

### `subagents.list`
```rust
// Return: [{ sessionKey, task, model, startTime, status }]
// Query ACP runtime for active sub-agent sessions
```

### `subagents.steer`
```rust
// Params: { target: String, message: String }
// Send steering message to sub-agent session
```

### `subagents.kill`
```rust
// Params: { target: String }
// Terminate sub-agent session
```

---

## Group 10: Security/Approvals

### `security.audit`
```rust
// Run checks: config file permissions, exposed API keys, open ports
// Return: [{ check, status: "pass"|"warn"|"fail", detail }]
```

### `approvals.list`
```rust
// Return pending exec approval requests from DB
```

### `approvals.respond`
```rust
// Params: { code: String, decision: "allow-once"|"allow-always"|"deny" }
// Update approval record, unblock waiting exec
```

---

## Implementation Strategy

For each method:
1. Check if it currently returns a real response or a stub
2. If stub: wire to the appropriate crate (agent, sessions, channels, cron scheduler, etc.)
3. All methods should return `{"error": "method not available"}` with HTTP 200 (not panic) if underlying service is unavailable

Helper macro for stubs-to-real conversion:
```rust
// Before (stub):
"sessions.list" => Ok(json!([])),

// After (real):
"sessions.list" => {
    let sessions = list_sessions(&state.state_dir).await?;
    Ok(json!(sessions))
}
```

## Rules
- `cargo build --workspace` must pass clean
- No panics in method handlers — always return error JSON
- Methods that need I/O must be async
- Prioritize the groups in order listed above (Group 1 most important)

## Completion
```bash
openclaw system event --text "Sprint 6 done: all 108 gateway methods real — agent.run, sessions CRUD, channels dispatch, memory search/get, cron verified, nodes, browser, gateway control, subagents, security audit, approvals" --mode now
```

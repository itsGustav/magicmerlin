# Sprint 6 — Gateway Method Hardening

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Gateway has 108 methods registered in `gateway/src/methods/mod.rs`.
Many return stub JSON like `{"ok": true}` or `{"result": []}`.
This sprint makes them all return real data.

## Your Mission
Wire every gateway method to real data sources. Group by subsystem.

---

## Group 1: Agent Methods (gateway/src/main.rs + methods/)

### `agent.run { message, sessionKey?, model?, thinking? }`
```rust
// Already partially implemented — ensure it:
// 1. Creates/looks up session by sessionKey
// 2. Appends user message to JSONL transcript
// 3. Calls agent engine (AgentEngine::run_turn)
// 4. Returns { reply: String, sessionKey: String, toolCalls: [...] }
```

### `agent.status`
```rust
// Return: { agents: [{ id, model, channels, lastActive, sessionCount }] }
// Read from agent registry (agents in config)
```

### `agent.list`
```rust
// Return array of all configured agents with id, model, state
```

### `agent.model.set { agentId, model }`
```rust
// Update agent config, persist to disk
// Return: { ok: true, previous: oldModel, current: newModel }
```

---

## Group 2: Sessions Methods

### `sessions.list { activeMinutes?, limit? }`
```rust
// Scan {state_dir}/sessions/*.jsonl
// For each: read last line for metadata, return array
// Return: [{ key, messageCount, lastActive, model, tokensUsed }]
```

### `sessions.show { sessionKey, limit? }`
```rust
// Read JSONL transcript for sessionKey
// Return last N messages: [{ role, content, timestamp, toolCalls? }]
```

### `sessions.delete { sessionKey }`
```rust
// Delete {state_dir}/sessions/{key}.jsonl
// Delete lock file if exists
// Return: { ok: true, deleted: sessionKey }
```

### `sessions.compact { sessionKey }`
```rust
// Call sessions::compact_session()
// Return: CompactionResult as JSON
```

### `sessions.history { sessionKey, limit?, includeTools? }`
```rust
// Same as sessions.show but with includeTools filter
```

### `sessions.send { sessionKey, message, timeoutSeconds? }`
```rust
// Inject message into session as inbound message
// Run agent turn
// Return: { reply: String }
```

### `sessions.spawn { task, agentId?, mode?, model?, runtime?, timeoutSeconds?, label? }`
```rust
// Create isolated session
// For runtime=subagent: create new session, inject task as first message, run agent turn
// For runtime=acp: delegate to ACP runtime
// Return: { sessionKey, sessionId }
```

### `sessions.yield { message? }`
```rust
// Mark session as yielded (waiting for sub-agent)
// Return: { ok: true }
```

---

## Group 3: Memory Methods

### `memory.search { query, maxResults?, minScore? }`
```rust
// Delegate to agent-tools memory search logic
// Scan workspace MEMORY.md + memory/*.md
// TF-IDF chunked search, return ranked results with citations
```

### `memory.get { path, from?, lines? }`
```rust
// Safe file read within workspace
// Return: { content, path, totalLines, from }
```

---

## Group 4: Channels Methods

### `channels.list`
```rust
// Return all configured channels with their status
// [{ name, platform, status, lastMessage, lastError? }]
// Status: "connected" | "disconnected" | "error" | "unconfigured"
```

### `channels.status { channel? }`
```rust
// Specific channel status or all if omitted
```

### `channels.send { channel?, target, message, replyTo? }`
```rust
// Dispatch to channel registry
// channels::dispatch_send(channel_name, target, message)
// Return: { ok: true, messageId? }
```

### `channels.react { channel, messageId, emoji }`
```rust
// channels::dispatch_react(...)
```

### `channels.delete { channel, messageId }`
```rust
// channels::dispatch_delete(...)
```

---

## Group 5: Nodes Methods

### `nodes.list`
```rust
// Return all configured nodes: [{ id, name, url, paired, lastSeen }]
```

### `nodes.describe { node }`
```rust
// GET {node.url}/api/describe
// Return node capabilities + info
```

### `nodes.run { node, command, cwd?, env?, timeoutMs? }`
```rust
// POST {node.url}/api/run
// Return: { exitCode, stdout, stderr }
```

### `nodes.invoke { node, invokeCommand, invokeParamsJson?, invokeTimeoutMs? }`
```rust
// POST {node.url}/api/invoke
```

### `nodes.notify { node, title, body, priority?, sound?, delivery? }`
```rust
// POST {node.url}/api/notify
```

### `nodes.location_get { node, desiredAccuracy?, locationTimeoutMs? }`
```rust
// GET {node.url}/api/location
```

### `nodes.screen_record { node, durationMs, screenIndex? }`
```rust
// POST {node.url}/api/screen/record
// Return: { videoPath or base64 }
```

### `nodes.camera_snap { node, facing?, maxWidth?, quality? }`
```rust
// POST {node.url}/api/camera/snap
// Return: { image: base64, mimeType }
```

---

## Group 6: Browser Methods

### `browser.status`
```rust
// Return: { running: bool, profiles: [...], tabs: [...] }
```

### `browser.tabs { profile? }`
```rust
// List open tabs for profile
// Return: [{ id, title, url, active }]
```

### `browser.open { url, profile? }`
```rust
// Open URL in browser
// Return: { targetId }
```

### `browser.snapshot { targetId?, profile?, refs? }`
```rust
// Accessibility tree snapshot
// Return: { snapshot: String, url, title }
```

### `browser.screenshot { targetId?, profile?, type? }`
```rust
// Return: { data: base64, mimeType }
```

### `browser.navigate { targetId, url }`
```rust
// Navigate tab to URL
```

### `browser.act { targetId, request, profile? }`
```rust
// Execute action: click/type/press/etc.
// request: { kind, ref?, selector?, text?, key? }
```

---

## Group 7: Subagents Methods

### `subagents.list { recentMinutes? }`
```rust
// Return active sub-agent sessions from ACP runtime
// [{ sessionKey, label, status, startedAt, model }]
```

### `subagents.steer { target, message }`
```rust
// Inject a steering message into a running sub-agent session
```

### `subagents.kill { target }`
```rust
// Terminate sub-agent session
```

---

## Group 8: Gateway Control Methods

### `gateway.status`
```rust
// Return: { version, uptime, pid, model, channels: [...], agents: [...] }
```

### `gateway.restart { reason?, delayMs? }`
```rust
// Schedule restart after delayMs (default 500ms)
// Return: { ok: true, restarting_in_ms: N }
// Use tokio::spawn + sleep + std::process::exit(0) (supervisor will restart)
```

### `gateway.config.get { path? }`
```rust
// Return full config or specific path
```

### `gateway.config.patch { raw, note? }`
```rust
// Merge patch into config, persist, optionally restart
```

---

## Group 9: Approvals Methods

### `approvals.list`
```rust
// Return pending approval requests from approvals store
// [{ id, code, command, requestedAt, sessionKey }]
```

### `approvals.set { code, mode }` where mode = allow-once|allow-always|deny
```rust
// Update approval decision in store
// Return: { ok: true, code, decision }
```

### `approvals.pending`
```rust
// Same as list but filtered to unresolved
```

---

## Implementation Approach

For each method group:
1. Read the existing stub in `gateway/src/methods/mod.rs` or `gateway/src/main.rs`
2. Replace stub body with real data read
3. Data sources: SQLite DB for sessions/cron, config manager for config, channel registry for channels, node configs for nodes, ACP runtime for subagents

---

## Rules
- `cargo build --workspace` clean
- Every method must return real data (no more `{"result": []}` stubs)
- Error responses: `{"error": {"code": N, "message": "..."}}`

## Completion
```bash
openclaw system event --text "Sprint 6 done: all 108 gateway methods return real data — sessions, memory, channels, nodes, browser, subagents, agent, gateway control, approvals all wired" --mode now
```

# Phase 2: Gateway — Build Instructions

You are building Magic Merlin, a 100% Rust replacement for OpenClaw. Phases 0-1 are complete (config, logging, infra, storage, providers, agent, agent-tools — 13K lines, 12 crates). Now build the Gateway layer.

## Existing Crates
- `magicmerlin-config`: Config loading, profiles, secrets
- `magicmerlin-logging`: Structured tracing
- `magicmerlin-infra`: HTTP client, text/time/markdown utils
- `magicmerlin-storage`: SQLite, JSONL transcripts, file locks, memory files
- `magicmerlin-providers`: 10 LLM providers, router with retry+failover, model registry
- `magicmerlin-agent`: Agent turn loop, session mgmt, system prompt assembly, heartbeat
- `magicmerlin-agent-tools`: Tool trait+registry, exec/process/read/write/edit tools
- `magicmerlin-gateway` (existing): Basic axum HTTP server, scheduler, pairing — EXTEND this crate

## What To Build (Phase 2) — Extend gateway + 2 new crates

### 2.1 WebSocket Gateway Server (extend `gateway`)
Replace/extend the existing gateway with a full WebSocket JSON-RPC server:

1. **WebSocket server** using `axum` + `tokio-tungstenite`:
   - Listen on configurable port (default 18789, dev 19001)
   - JSON-RPC protocol: `{method, params, id}` → `{result, id}` or `{error, id}`
   - Authentication: token-based (gateway.token config) and password-based
   - Concurrent client connections with broadcast capability
   - Keepalive pings every 30s

2. **Method routing** — implement these gateway methods:
   - `health` — returns ok + channel status + uptime
   - `status` — full system status (agents, sessions, models, config)
   - `system-presence` — heartbeat/presence info
   - `agent.run` — execute one agent turn (message → response)
   - `agent.abort` — abort in-progress agent run
   - `sessions.list` — list sessions with filters
   - `sessions.get` — get session details
   - `sessions.send` — send message to session
   - `sessions.spawn` — spawn sub-agent session
   - `sessions.compact` — compact a session
   - `sessions.delete` — delete a session
   - `cron.list` / `cron.add` / `cron.edit` / `cron.rm` / `cron.run` / `cron.enable` / `cron.disable` — full cron CRUD
   - `config.get` / `config.set` / `config.unset` — config operations
   - `approvals.list` / `approvals.approve` / `approvals.deny` — exec approvals
   - `plugins.list` / `plugins.enable` / `plugins.disable` — plugin management

3. **Run queue**:
   - Queue incoming agent.run requests
   - One active run per session at a time
   - Abort/timeout handling (configurable per-run timeout)
   - Stream partial results back to client as they arrive

4. **Service management**:
   - Daemon mode (background process with PID file)
   - macOS LaunchAgent plist generation and install/uninstall
   - Linux systemd unit generation
   - Graceful shutdown on SIGTERM/SIGINT
   - Auto-restart on crash (via LaunchAgent/systemd)
   - Bind address configuration (127.0.0.1 for local, 0.0.0.0 for LAN)

### 2.2 Auto-Reply Engine (new crate: `auto-reply`)
The conversation pipeline that connects channels to agents:

1. **Inbound pipeline**:
   - Receive normalized message from any channel
   - Determine target agent and session key
   - Apply DM policy (open, pairing, allowlist)
   - Apply mention gating (group chats: only respond when @mentioned)
   - Queue message in collect buffer

2. **Collect/debounce**:
   - Batch messages arriving within a window (configurable, default 2s)
   - Start agent turn after debounce period
   - Cancel pending turn if new message with higher priority arrives

3. **Slash command handling**:
   - `/status` — show session status
   - `/compact` — force session compaction
   - `/reasoning` — toggle reasoning mode
   - `/model` — show/change model
   - `/reset` — reset session
   - `/help` — list available commands

4. **Reply formatting**:
   - Platform-specific formatting (Telegram markdown, Discord markdown, WhatsApp plain)
   - NO_REPLY / HEARTBEAT_OK detection and suppression
   - Reply-to/quote support
   - Reaction handling (emoji reactions per platform)
   - Media attachment routing (images, voice, documents)
   - Message splitting for platform length limits (Telegram 4096, Discord 2000)

5. **Delivery context**:
   - Track which channel/chat a session is associated with
   - Route agent responses back to the correct channel
   - Support announce mode (deliver to a different channel)
   - Inline button support (Telegram inline keyboards)

### 2.3 Session Engine (new crate: `sessions`)
Extended session management beyond what `agent` crate provides:

1. **Session resolution**:
   - Map inbound messages to session keys
   - `agent:<name>:main` for DM sessions
   - `telegram:<chat_id>` for group sessions
   - `telegram:slash:<user_id>` for slash command sessions
   - Custom session key patterns

2. **Session lifecycle**:
   - Create on first message
   - Load transcript from JSONL
   - Append messages atomically
   - Compact when context window threshold reached (default 80%)
   - Pre-compaction memory flush
   - Delete/archive old sessions

3. **Transcript repair**:
   - Detect broken tool_use/tool_result pairs (CRITICAL — this was a real bug that broke Henry)
   - Auto-repair by inserting synthetic error results
   - Validate transcript integrity on load

4. **Sub-agent sessions**:
   - Spawn isolated sessions for sub-agent work
   - Track parent-child relationships
   - Auto-cleanup of stale sub-agent sessions
   - Send messages between sessions

5. **Session state tracking**:
   - Token usage per session
   - Cost accumulation
   - Last activity timestamp
   - Compaction count
   - Model override per session

## Integration
- Wire WebSocket server into gateway main.rs (replace or augment existing HTTP-only server)
- Connect auto-reply to gateway (messages come in via channels, get routed through auto-reply to agent)
- Wire sessions crate into agent turn loop
- Keep existing HTTP endpoints working alongside WebSocket
- All new crates added to workspace Cargo.toml
- `cargo check`, `cargo test` must pass clean
- Commit after each major component

## Quality Requirements
- Every public function has a doc comment
- Error handling: `thiserror` for custom, `anyhow` for application
- No unwrap() in library code
- Tests for: WebSocket message routing, slash command parsing, session resolution, transcript repair, collect/debounce logic

## Dependencies
- `tokio-tungstenite` for WebSocket
- `futures` for stream handling
- Existing workspace crates

When completely finished, run:
openclaw system event --text "Phase 2 complete: Gateway (WebSocket server, auto-reply engine, sessions) built and tested" --mode now

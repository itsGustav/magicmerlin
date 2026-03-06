# Magic Merlin — Comprehensive Engineering Plan
## 100% OpenClaw Parity in Rust

**Created**: 2026-03-06
**Target**: Full drop-in replacement for OpenClaw 2026.3.2
**Current state**: ~6K lines Rust (5%). Target: ~150-200K lines Rust.

---

## Architecture Overview

OpenClaw is a **1,752-module TypeScript monolith** (50MB compiled) with:
- 932 JS files, ~1.2M lines
- 40+ CLI commands with subcommands
- 327 agent modules (brain, tools, sessions, skills, sandbox)
- 173 auto-reply modules (the conversation engine)
- 91 channel modules + 6 channel plugins (Telegram, Discord, WhatsApp, Signal, Slack, iMessage, LINE, Web)
- 78 browser automation modules
- 48 gateway modules (WebSocket server, protocol, methods)
- 48 memory modules
- 39 media understanding providers (10 providers: OpenAI, Anthropic, Google, Groq, Deepgram, Mistral, MiniMax, Moonshot, xAI)
- 23 cron modules
- 18 ACP (Agent Control Protocol) modules
- 36 plugin runtime modules
- Control UI (React SPA, 18K+ lines)
- Canvas host (18K+ lines)
- 50+ agent tools (browser, web, exec, sessions, memory, nodes, message, TTS, PDF, image, etc.)

---

## Phase 0: Foundation (Week 1-2)
**Goal**: Core infrastructure that everything else depends on.

### 0.1 Configuration System
- [ ] TOML/JSON config loader (`openclaw.json` compatible)
- [ ] Zod-equivalent schema validation (use `serde` + custom validators)
- [ ] Config get/set/unset CLI commands
- [ ] Environment variable overlay (`OPENCLAW_*`)
- [ ] Profile isolation (`--profile`, `--dev`)
- [ ] State directory management (`~/.openclaw/`)
- [ ] Secrets runtime (`secrets.env` loader, reload on SIGHUP)
- **Files**: 118 config modules to replicate
- **Est**: 3,000 lines Rust

### 0.2 Logging & Infrastructure
- [ ] Structured logging (file + console, levels: silent→trace)
- [ ] Log rotation and gateway log paths
- [ ] Network utilities (TLS, outbound HTTP client, proxy support)
- [ ] Time formatting utilities
- [ ] Markdown processing (for system prompts)
- [ ] Text utilities (truncation, sanitization, encoding)
- **Files**: 161 infra + 14 logging + 24 shared + 18 utils + 7 markdown
- **Est**: 4,000 lines Rust

### 0.3 Storage Layer
- [ ] SQLite database (sessions, cron, approvals, plugins)
- [ ] JSONL transcript read/write/compact/repair
- [ ] Session file locking (pid-based)
- [ ] Memory file management (MEMORY.md, daily files, memory_search embeddings)
- **Est**: 2,500 lines Rust

**Phase 0 total: ~9,500 lines | 2 weeks**

---

## Phase 1: Agent Runtime (Week 3-5)
**Goal**: The brain — LLM calls, tool execution, session management.

### 1.1 Provider Routing
- [ ] Multi-provider LLM client (OpenAI, Anthropic, Google, xAI, Groq, Mistral, MiniMax, Moonshot, DeepSeek, local/Ollama)
- [ ] API key auth + OAuth token refresh (OpenAI Codex, GitHub Copilot, Qwen Portal)
- [ ] Model alias resolution (`gpt` → `openai/gpt-5.2`, `sonnet` → `anthropic/claude-sonnet-4-6`)
- [ ] Failover chain (primary → fallback → fallback2)
- [ ] Rate limiting, retry with backoff
- [ ] Streaming response handling (SSE)
- [ ] Token counting and context window management
- [ ] Cost tracking per session
- [ ] Auth profiles (`auth-profiles.json` per agent)
- [ ] API key rotation
- **Files**: 4 providers + agents/models-config + auth-profiles (20+ modules)
- **Est**: 8,000 lines Rust

### 1.2 Agent Engine
- [ ] System prompt assembly (AGENTS.md, SOUL.md, USER.md, IDENTITY.md, TOOLS.md, MEMORY.md, HEARTBEAT.md, BOOTSTRAP.md injection)
- [ ] Workspace file injection with truncation limits
- [ ] Skills discovery and prompt injection
- [ ] Tool schema generation (JSON Schema for each tool)
- [ ] Tool call → tool result loop (the core agent turn)
- [ ] Context pruning / compaction (pre-compaction memory flush)
- [ ] Session scope (`per-channel-peer`, `per-agent`, etc.)
- [ ] Agent isolation (separate workspaces, configs, sessions per agent)
- [ ] Multi-agent support (main, paylobster, henry, etc.)
- [ ] Announce/delivery context (route replies to correct channel)
- [ ] Heartbeat system (poll → HEARTBEAT_OK or action)
- [ ] Collect queue (message batching before agent turn)
- **Files**: 327 agent modules
- **Est**: 15,000 lines Rust

### 1.3 Tool Execution Engine
- [ ] **exec**: Shell command execution with PTY support, background processes, timeouts
- [ ] **read/write/edit**: File operations with workspace sandboxing
- [ ] **process**: Background session management (list, poll, log, write, send-keys, kill)
- [ ] **web_search**: Brave Search API integration
- [ ] **web_fetch**: URL fetch with markdown/text extraction (readability)
- [ ] **browser**: Playwright-based browser automation (snapshot, screenshot, act, navigate)
- [ ] **canvas**: Canvas present/eval/snapshot/A2UI
- [ ] **nodes**: Node device control (camera, screen, location, run, invoke)
- [ ] **message**: Channel message send/react/delete/edit + inline buttons
- [ ] **tts**: Text-to-speech routing
- [ ] **image**: Vision model image analysis
- [ ] **pdf**: PDF analysis (native + extraction fallback)
- [ ] **memory_search**: Semantic search over memory files (embedding model)
- [ ] **memory_get**: Safe snippet read from memory files
- [ ] **session_status**: Session status card with model/cost/context info
- [ ] **sessions_list/history/send/spawn**: Sub-agent orchestration
- [ ] **subagents**: List/steer/kill sub-agent sessions
- [ ] **agents_list**: List available agent IDs
- [ ] **cron_tool**: Cron management from agent context
- [ ] Tool permission system (deny lists, sandbox mode, workspace-only FS)
- [ ] Tool result size limits and truncation
- **Files**: 50+ tool modules
- **Est**: 20,000 lines Rust

**Phase 1 total: ~43,000 lines | 3 weeks**

---

## Phase 2: Gateway (Week 6-7)
**Goal**: WebSocket server that orchestrates everything.

### 2.1 WebSocket Server
- [ ] JSON-RPC over WebSocket protocol
- [ ] Authentication (token, password)
- [ ] Method routing (health, status, agent.run, cron.*, sessions.*, etc.)
- [ ] Concurrent session management
- [ ] Run queue with abort/timeout
- [ ] Streaming responses to clients
- **Files**: 48 gateway modules
- **Est**: 6,000 lines Rust

### 2.2 Cron Service
- [ ] Cron expression parsing (standard + extended)
- [ ] Job persistence (SQLite)
- [ ] Isolated agent runs per cron job
- [ ] Retry logic with exponential backoff
- [ ] Dead letter queue
- [ ] Consecutive error tracking
- [ ] Model/channel override per job
- **Files**: 23 cron modules
- **Est**: 3,000 lines Rust (mostly done — extend existing)

### 2.3 Auto-Reply Engine
- [ ] Inbound message → agent turn pipeline
- [ ] Reply routing (back to source channel)
- [ ] Command parsing (slash commands: /status, /reasoning, /compact, etc.)
- [ ] Message queue with collect/debounce
- [ ] Reply formatting per platform (Telegram markdown, Discord, WhatsApp)
- [ ] Silent reply handling (NO_REPLY, HEARTBEAT_OK)
- [ ] Reaction system (minimal mode, platform-specific emoji)
- [ ] Reply-to/quote support
- [ ] Media attachment handling (inbound images, voice, documents)
- [ ] ACP command routing
- [ ] Sub-agent command routing
- **Files**: 173 auto-reply modules
- **Est**: 10,000 lines Rust

### 2.4 Session Management
- [ ] Session creation/lookup/compaction
- [ ] Session key resolution (`agent:henry:main`, `telegram:8527778539`)
- [ ] JSONL transcript persistence
- [ ] Token tracking per session
- [ ] Compaction triggers (context % threshold)
- [ ] Pre-compaction memory flush
- [ ] Session locks (prevent concurrent writes)
- **Files**: 7 session modules + agents session code
- **Est**: 3,000 lines Rust

**Phase 2 total: ~22,000 lines | 2 weeks**

---

## Phase 3: Channels (Week 8-10)
**Goal**: All chat platform integrations.

### 3.1 Channel Framework
- [ ] Channel registry (plugin-based architecture)
- [ ] Inbound message normalization (unified envelope format)
- [ ] Outbound message formatting (per-platform)
- [ ] DM policy enforcement (open, pairing, allowlist)
- [ ] Mention gating (group chat @mention detection)
- [ ] Session envelope mapping (chat → session key)
- [ ] Media download/upload pipeline
- [ ] Stall watchdog (detect stuck channels)
- [ ] Health monitoring per channel
- **Files**: 91 channel modules
- **Est**: 5,000 lines Rust

### 3.2 Telegram
- [ ] Bot API client (polling + webhook modes)
- [ ] Message send/edit/delete/react
- [ ] Inline keyboards/buttons
- [ ] Media handling (photos, voice, documents, video)
- [ ] Group chat support (mentions, replies, topics)
- [ ] Multiple bot accounts (one per agent)
- [ ] Pairing flow (DM approval)
- [ ] Slash command registration
- **Files**: 52 telegram modules
- **Est**: 6,000 lines Rust

### 3.3 Discord
- [ ] Discord.js equivalent (gateway WebSocket + REST)
- [ ] Message/embed/reaction support
- [ ] Voice channel support
- [ ] Thread management
- [ ] Guild management (roles, channels, moderation)
- [ ] Slash commands
- [ ] Presence/activity status
- [ ] Monitor (reconnect, health)
- **Files**: 77 discord modules
- **Est**: 8,000 lines Rust

### 3.4 WhatsApp
- [ ] WhatsApp Web client (Baileys equivalent in Rust)
- [ ] QR code pairing
- [ ] Message send/receive/react
- [ ] Media handling
- [ ] Group support
- **Files**: 2 whatsapp modules (likely thin wrapper)
- **Est**: 4,000 lines Rust

### 3.5 Signal
- [ ] Signal CLI integration
- [ ] Message send/receive
- [ ] Group support
- [ ] Media handling
- [ ] Monitor daemon
- **Files**: 17 signal modules
- **Est**: 3,000 lines Rust

### 3.6 Slack
- [ ] Slack Web API + Events API
- [ ] Socket mode
- [ ] Message formatting (blocks)
- [ ] Channel/thread management
- [ ] Monitor with event handlers
- **Files**: 61 slack modules
- **Est**: 5,000 lines Rust

### 3.7 iMessage
- [ ] AppleScript/JXA bridge for Messages.app
- [ ] Monitor for new messages
- [ ] Send/receive
- [ ] Group chat support
- **Files**: 16 imessage modules
- **Est**: 2,000 lines Rust

### 3.8 LINE
- [ ] LINE Messaging API
- [ ] Flex message templates
- [ ] Rich menus
- **Files**: 28 line modules
- **Est**: 3,000 lines Rust

### 3.9 Web Chat
- [ ] WebSocket-based web chat client
- [ ] Inbound message handling
- [ ] Auto-reply integration
- **Files**: 42 web modules
- **Est**: 3,000 lines Rust

**Phase 3 total: ~39,000 lines | 3 weeks**

---

## Phase 4: Media & Intelligence (Week 11-12)
**Goal**: Media processing, browser automation, understanding.

### 4.1 Media Understanding
- [ ] Image analysis (10 provider adapters)
- [ ] Audio transcription (Whisper, Deepgram, Groq)
- [ ] Video frame extraction + analysis
- [ ] PDF native analysis (Anthropic, Google) + fallback extraction
- [ ] Provider routing (cheapest/fastest/best per media type)
- **Files**: 39 media-understanding modules
- **Est**: 5,000 lines Rust

### 4.2 Browser Automation
- [ ] Playwright/CDP integration from Rust
- [ ] Page snapshot (accessibility tree → text)
- [ ] Screenshot capture
- [ ] Action execution (click, type, press, hover, drag, select, fill)
- [ ] Tab management (open, close, focus, list)
- [ ] Profile management (openclaw, chrome relay)
- [ ] Browser process lifecycle (start, stop)
- **Files**: 78 browser modules
- **Est**: 8,000 lines Rust

### 4.3 Canvas Host
- [ ] HTML canvas rendering engine
- [ ] A2UI (Agent-to-UI) push protocol
- [ ] Screenshot/snapshot of rendered content
- [ ] JavaScript evaluation in canvas context
- **Files**: 18K+ lines in canvas-host
- **Est**: 5,000 lines Rust

### 4.4 Link Understanding
- [ ] URL preview/metadata extraction
- [ ] OG tag parsing
- [ ] Content summarization pipeline
- **Files**: 5 link-understanding modules
- **Est**: 1,000 lines Rust

### 4.5 TTS (Text-to-Speech)
- [ ] Provider routing (ElevenLabs, OpenAI, etc.)
- [ ] Voice selection
- [ ] Audio format conversion
- [ ] Channel-specific output (Telegram voice note, etc.)
- **Files**: 2 tts modules
- **Est**: 800 lines Rust

**Phase 4 total: ~19,800 lines | 2 weeks**

---

## Phase 5: CLI (Week 13-14)
**Goal**: Full CLI with all 40+ commands matching OpenClaw exactly.

### 5.1 Core Commands
- [ ] `status` — Channel health, session info, model info
- [ ] `setup` / `configure` — Interactive wizard
- [ ] `onboard` — First-run setup
- [ ] `health` / `doctor` — System diagnostics
- [ ] `dashboard` — Open Control UI
- [ ] `tui` — Terminal UI (Ratatui)
- [ ] `help` — Full help tree
- [ ] `completion` — Shell completions (bash, zsh, fish)
- [ ] `version` — Version output
- [ ] `update` — Self-update mechanism
- [ ] `reset` / `uninstall` — Cleanup

### 5.2 Agent Commands
- [ ] `agent` — Run one agent turn
- [ ] `agents list/add/remove/config` — Agent management
- [ ] `models list/set/test` — Model configuration

### 5.3 Gateway Commands
- [ ] `gateway start/stop/restart/status` — Service management
- [ ] `gateway call <method>` — Direct method invocation
- [ ] `daemon` — Legacy alias

### 5.4 Channel Commands
- [ ] `channels login/logout/status` — Per-channel auth
- [ ] `message send` — Direct message sending
- [ ] `directory` — Contact/group lookup
- [ ] `pairing` — DM pairing management

### 5.5 Data Commands
- [ ] `sessions list/show/delete/compact` — Session management
- [ ] `memory search/get` — Memory operations
- [ ] `cron list/add/remove/run` — Cron management
- [ ] `logs` — Log viewer
- [ ] `hooks` — Webhook management
- [ ] `webhooks` — Webhook helpers

### 5.6 Security/Admin Commands
- [ ] `config get/set/unset/file/validate` — Config management
- [ ] `security audit` — Security scanner
- [ ] `secrets reload` — Runtime secrets
- [ ] `sandbox` — Container management
- [ ] `approvals` — Exec approval management
- [ ] `plugins` — Plugin management
- [ ] `skills` — Skill inspection
- [ ] `dns` — DNS/Tailscale helpers
- [ ] `devices` — Device pairing
- [ ] `nodes` — Node management
- [ ] `qr` — QR code generation
- [ ] `browser` — Browser management
- [ ] `acp` — Agent Control Protocol tools
- [ ] `docs` — Documentation browser
- [ ] `system` — System events/heartbeat/presence

- **Files**: 57 command + 23 CLI modules
- **Est**: 12,000 lines Rust

**Phase 5 total: ~12,000 lines | 2 weeks**

---

## Phase 6: Plugins, Skills & ACP (Week 15-16)
**Goal**: Extension system, skill loading, Agent Control Protocol.

### 6.1 Plugin System
- [ ] Plugin discovery and loading
- [ ] Plugin lifecycle (init, start, stop)
- [ ] Plugin runtime isolation
- [ ] Bundled plugins (session-memory, command-logger, boot-md, bootstrap-extra-files)
- **Files**: 36 plugin modules
- **Est**: 3,000 lines Rust

### 6.2 Skills System
- [ ] Skill discovery (bundled, workspace, clawhub)
- [ ] SKILL.md loading and prompt injection
- [ ] Skill dependency resolution (requiredEnv, primaryEnv)
- [ ] Skill scripts execution
- **Files**: agents/skills modules
- **Est**: 2,000 lines Rust

### 6.3 ACP (Agent Control Protocol)
- [ ] ACP runtime (sub-agent spawn via external harnesses)
- [ ] ACP control plane
- [ ] ACPX extension integration
- [ ] Thread-bound sessions
- **Files**: 18 acp modules
- **Est**: 3,000 lines Rust

### 6.4 Node Host
- [ ] Remote node communication
- [ ] Camera/screen/location/run/invoke commands
- [ ] Device pairing protocol
- **Files**: 1 node-host + nodes tool modules
- **Est**: 2,000 lines Rust

**Phase 6 total: ~10,000 lines | 2 weeks**

---

## Phase 7: Control UI & Polish (Week 17-18)
**Goal**: Web dashboard, LaunchAgent, production hardening.

### 7.1 Control UI
- [ ] React SPA (or Leptos/Yew for Rust-native)
- [ ] Dashboard: agents, sessions, cron, logs, config
- [ ] Real-time WebSocket updates
- [ ] Dark/light theme
- [ ] Mobile responsive
- **Files**: 18K+ lines
- **Est**: 8,000 lines Rust/HTML/JS (could embed OpenClaw's UI or rebuild in Leptos)

### 7.2 Service Management
- [ ] macOS LaunchAgent generation and management
- [ ] Linux systemd unit generation
- [ ] PID file management
- [ ] Graceful shutdown
- [ ] Auto-restart on crash
- **Est**: 1,500 lines Rust

### 7.3 Security
- [ ] Sandbox mode (container isolation via Docker/nsjail)
- [ ] Workspace-only filesystem restrictions
- [ ] Tool deny lists
- [ ] Trusted proxy configuration
- [ ] Security audit scanner
- **Files**: 8 security modules
- **Est**: 2,000 lines Rust

### 7.4 Testing & CI
- [ ] Unit tests for all modules (target: 90%+ coverage)
- [ ] Integration tests (agent turn, channel round-trip, cron execution)
- [ ] Parity tests (diff output against real OpenClaw)
- [ ] CI pipeline (GitHub Actions: build, test, lint, release)
- [ ] Cross-compilation (macOS ARM/x86, Linux ARM/x86, Windows)
- [ ] Docker image
- **Est**: 15,000 lines Rust (tests)

### 7.5 Documentation
- [ ] README with quickstart
- [ ] Migration guide (OpenClaw → Magic Merlin)
- [ ] Architecture docs
- [ ] API reference
- [ ] Man pages
- **Est**: 5,000 lines Markdown

**Phase 7 total: ~31,500 lines | 2 weeks**

---

## Summary

| Phase | Focus | Lines (est.) | Duration |
|-------|-------|-------------|----------|
| 0 | Foundation (config, logging, storage) | 9,500 | 2 weeks |
| 1 | Agent Runtime (providers, engine, tools) | 43,000 | 3 weeks |
| 2 | Gateway (WS server, cron, auto-reply, sessions) | 22,000 | 2 weeks |
| 3 | Channels (Telegram, Discord, WhatsApp, Signal, Slack, iMessage, LINE, Web) | 39,000 | 3 weeks |
| 4 | Media & Intelligence (understanding, browser, canvas, TTS) | 19,800 | 2 weeks |
| 5 | CLI (40+ commands, TUI) | 12,000 | 2 weeks |
| 6 | Plugins, Skills & ACP | 10,000 | 2 weeks |
| 7 | Control UI, Security, Testing, Docs | 31,500 | 2 weeks |
| **Total** | | **~186,800** | **18 weeks** |

### With Parallel Agents (Aggressive Timeline)

Using 3-4 Claude Code / Codex agents in parallel:

| Approach | Timeline | Cost |
|----------|----------|------|
| Sequential (1 agent) | 18 weeks | Subscription only |
| 2 parallel agents | 10 weeks | Subscription only |
| 4 parallel agents | 6 weeks | Subscription only (watch OOM) |
| 4 agents + Codex cloud | 4-5 weeks | Subscription only |

### Recommended Execution Order

```
Week 1-2:  Phase 0 (Foundation) — 1 agent
Week 3-5:  Phase 1 (Agent Runtime) — 2 agents (providers + tools in parallel)
Week 5-7:  Phase 2 (Gateway) — 2 agents (gateway + auto-reply in parallel)
Week 7-10: Phase 3 (Channels) — 3 agents (Telegram+Discord, WhatsApp+Signal, Slack+iMessage+LINE)
Week 10-12: Phase 4 (Media) — 2 agents
Week 12-14: Phase 5+6 (CLI + Plugins) — 2 agents in parallel
Week 14-16: Phase 7 (UI + Testing) — 2 agents
Week 16-18: Integration testing, parity validation, release
```

### Critical Dependencies

```
Phase 0 → everything (must be first)
Phase 1 → Phase 2 (gateway needs agent runtime)
Phase 1 → Phase 3 (channels need tool execution)
Phase 2 → Phase 3 (channels plug into gateway)
Phase 1 → Phase 4 (media tools need tool engine)
Phase 0+1+2 → Phase 5 (CLI wraps everything)
Phase 1 → Phase 6 (plugins/skills need agent engine)
Phase 2+5 → Phase 7 (UI needs gateway + CLI)
```

### Risk Factors

1. **Playwright/CDP from Rust**: No mature Rust equivalent. Options: chromiumoxide crate, or shell out to playwright CLI.
2. **WhatsApp Web protocol**: Baileys is complex JS. Options: wrap existing Baileys via wasm/subprocess, or use whatsmeow (Go) via FFI.
3. **iMessage**: Requires macOS-specific AppleScript/JXA bridge. Use `std::process::Command` to call `osascript`.
4. **Discord Gateway**: Complex WebSocket protocol with sharding. Use `serenity` or `twilight` crates.
5. **Control UI**: Rebuilding in Rust (Leptos) is clean but slow. Alternative: embed OpenClaw's existing React UI and serve statically.
6. **Mac OOM**: 4 parallel Claude Code agents crashed Mac mini before. Use Codex cloud or limit to 2-3 local agents.

### Existing Crate Recommendations

| Need | Crate | Why |
|------|-------|-----|
| HTTP server | `axum` ✅ (already using) | Best Rust web framework |
| WebSocket | `tokio-tungstenite` | Gateway WS server |
| CLI | `clap` ✅ (already using) | Standard CLI parser |
| Database | `rusqlite` ✅ (already using) | Lightweight, embedded |
| HTTP client | `reqwest` ✅ (already using) | Async HTTP with TLS |
| JSON | `serde_json` ✅ (already using) | Standard |
| Telegram | `teloxide` | Best Telegram bot framework |
| Discord | `serenity` or `twilight` | Full Discord API |
| Cron parsing | `cron` ✅ (already using) | Cron expressions |
| TUI | `ratatui` | Terminal UI (for `tui` command) |
| Markdown | `pulldown-cmark` | Markdown parsing |
| Browser/CDP | `chromiumoxide` | Chrome DevTools Protocol |
| Embeddings | `fastembed-rs` | Local embedding model |
| Process/PTY | `portable-pty` | PTY support for exec |
| Template | `tera` or `askama` | System prompt assembly |
| Signal | `presage` | Signal protocol in Rust |
| QR code | `qrcode` | QR generation |

---

## Definition of Done (100% Parity)

- [ ] All 40+ CLI commands produce identical output to OpenClaw
- [ ] Gateway accepts same WebSocket protocol, same methods, same params
- [ ] All 50+ agent tools work identically
- [ ] All 8 channel integrations connect and relay messages
- [ ] Config format is 100% compatible (`openclaw.json` works in both)
- [ ] Session JSONL format is compatible (can read OpenClaw sessions)
- [ ] Cron jobs import and run correctly
- [ ] Skills load and inject correctly
- [ ] Plugins load and run correctly
- [ ] Control UI serves and functions
- [ ] `cargo install magicmerlin` → drop-in replacement for `openclaw`
- [ ] Parity sentinel shows 0 diffs on all surfaces
- [ ] All existing tests pass + 90%+ coverage
- [ ] Cross-platform binaries (macOS ARM/x86, Linux ARM/x86)
- [ ] Docker image published

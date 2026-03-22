# Magic Merlin — Completion Strategy
## Full Functional Parity with OpenClaw

**Written**: 2026-03-22
**Author**: Gustav (AI orchestrator)
**Target**: Drop-in replacement for OpenClaw 2026.3.x — fully functional, production-ready

---

## 1. Current State Audit

### What exists (post Parity Pass 7)

| Crate | Lines | Status | Assessment |
|-------|-------|--------|------------|
| `config` | ~1,500 | ✅ Done | Config loader, env overlay, secrets, profiles |
| `logging` | ~400 | ✅ Done | Structured logging, file + console |
| `infra` | ~500 | ✅ Done | Network utils, TLS, HTTP client |
| `storage` | ~600 | ✅ Done | SQLite + JSONL |
| `providers` | ~3,000 | ✅ Done | 12 providers (OAI, Anthropic, Google, xAI, Groq, Mistral, MiniMax, Moonshot, DeepSeek, local) |
| `agent` | 2,337 | 🟡 70% | Engine, system_prompt, session, queue, registry, heartbeat — missing workspace injection depth, compaction |
| `agent-tools` | 2,957 | 🟡 40% | 23 tools registered, most are stubs — exec/read/write are real, others thin |
| `sessions` | 413 | 🔴 25% | Basic session lookup — missing JSONL compaction, context window mgmt, locks |
| `auto-reply` | 435 | 🔴 20% | Structure only — missing command parsing depth, DM policy, reaction routing, platform formatters |
| `gateway` | 8,343 | 🟡 65% | WS server, scheduler, run_queue, pairing — 108 methods but many return stubs |
| `channels` | 8,638 | 🟡 50% | Telegram deep (✅), Discord tranche 1 (🟡), Slack/WhatsApp/LINE/Web stubs (🔴), Signal/iMessage absent |
| `media` | 4,674 | 🟡 60% | Understanding, browser CDP, canvas, TTS, links — browser likely incomplete |
| `acp` | 704 | 🟡 55% | Runtime exists, needs full harness routing |
| `plugins` | 1,102 | 🟡 60% | Registry + skills loader — bundled plugins not implemented |
| `cli` | 1,486 | 🔴 30% | 193 commands scaffolded but monolithic, thin implementations |
| `docs` | 0 | 🔴 0% | 332 pages needed, 5 partial |

**Total Rust**: ~86,750 lines  
**Target**: ~186,800 lines  
**Code gap**: ~100,000 lines remaining  
**Functional completeness**: ~45%

---

## 2. Gap Inventory (Ranked by Impact)

### P0 — Blockers (nothing works end-to-end without these)

**A. Tool implementations (agent-tools)** — Most impactful gap
- `exec`: Needs real PTY support (`portable-pty` crate), background process management, exec sessions with poll/log/write/send-keys
- `browser`: CDP integration in `media` exists but needs to wire into ToolContext, handle profiles, snapshots with refs
- `memory_search`: Currently stub — needs embedding model (`fastembed-rs`), cosine similarity, file indexing
- `sessions_spawn`: Stub — needs to wire to ACP runtime + subagent lifecycle tracking
- `cron` tool: Stub — needs to call gateway scheduler over HTTP/WS
- `message` tool: Stub — needs to dispatch through channel registry
- `nodes` tool: Stub — needs node-host protocol
- `canvas` tool: Stub — needs canvas host process
- `tts` tool: Needs ElevenLabs + OpenAI routing, channel-format output

**B. Auto-reply depth**
- Slash command parsing (`/status`, `/model`, `/compact`, `/reasoning`, `/approve`, `/session`, `/sessions`, `/memory`, `/cron`, `/logs`, `/debug`, `/reset`, `/help`) — 30+ commands, currently 3-4
- Heartbeat processing (HEARTBEAT_OK suppression, action detection)
- Reply-to/quote construction (Telegram `reply_parameters`, Discord reply embed)
- Reaction routing (minimal mode — platform emoji dispatch)
- Collect/debounce window (batch messages before triggering agent turn)
- Platform formatter completeness (Telegram MarkdownV2 escaping, Discord embed building, WhatsApp plain text)
- Authorized sender enforcement (allowlist check before any agent turn)
- NO_REPLY detection and suppression
- Inbound media routing (voice → transcription → agent context)

**C. Session management completeness**
- JSONL compaction (pre-compaction memory flush → compact → resume)
- Context window % tracking (trigger compaction at threshold)
- Session file locking (PID-based, prevent concurrent writes)
- Pre-compaction hook (extract memories before squashing history)
- Session key resolution across all formats (telegram:CHATID, agent:henry:main, etc.)

### P1 — Core features (make it useful)

**D. Channel completeness**
- **Signal**: Completely absent. Needs `presage` crate integration (Signal protocol in Rust). Complex.
- **iMessage**: Completely absent. macOS-only `osascript` bridge.
- **WhatsApp**: 178 lines — needs full Baileys-equivalent or subprocess wrapper around `whatsmeow` (Go).
- **Discord**: Tranche 1 done but missing voice, sharding, all 77 original modules worth of functionality
- **Slack**: 193 lines — needs Socket Mode, blocks, thread handling, full event loop
- **LINE**: 169 lines — needs full Messaging API, flex messages
- **Web channel**: 151 lines — needs full WebSocket relay, session mapping

**E. Agent engine depth**
- Workspace file injection (AGENTS.md, SOUL.md, USER.md, IDENTITY.md, TOOLS.md, MEMORY.md, HEARTBEAT.md) with per-agent truncation
- Skills discovery injection (available_skills XML block)
- Project context injection (CLAUDE.md-equivalent)
- Context pruning/compaction logic
- Multi-agent management (spawn, list, per-agent config isolation)
- Auth profile rotation per agent
- Announce delivery (route reply to specific channel after cron/isolated turn)

**F. CLI depth**
- 193 commands exist but most are clap stubs calling unimplemented!()
- Need to wire each command to its gateway method or direct library call
- `gateway start/stop/restart` → service management (LaunchAgent/systemd)
- `sessions compact/show/delete` → session JSONL operations
- `config get/set/validate` → ConfigManager ops
- `cron list/add/remove/run/runs` → scheduler HTTP calls
- `security audit` → security scanner
- `channels login/logout/status` → per-provider auth flows
- `tui` → Ratatui terminal dashboard

### P2 — Completeness (parity definition)

**G. Control UI**
- React SPA not started
- Options: (1) Serve OpenClaw's UI statically (fast), (2) Rebuild in Leptos (clean Rust), (3) Minimal custom HTML
- Recommendation: Serve OpenClaw UI statically in gateway with a thin shim for API differences

**H. Documentation site (332 pages)**
- Not a runtime blocker but required for "full parity" definition
- Strategy: generate from code + templates; automate the bulk with a Codex agent

**I. Parity testing**
- Sentinel exists but needs integration test harness
- Wire `cargo test` to actually launch gateway and run round-trip tests
- Diff output from real OpenClaw vs Magic Merlin on same inputs

---

## 3. Execution Plan

### Parallel track model (4 Claude Code agents + 1 orchestrator)

```
ORCHESTRATOR (Gustav/main session)
├── AGENT A: Core runtime (agent engine + tool completions)
├── AGENT B: Channel completions (Signal, iMessage, WhatsApp, Slack deepening)
├── AGENT C: CLI wiring + auto-reply depth
└── AGENT D: Testing + CI + docs generator
```

Run tranches of 2-4 weeks. Each agent has a scoped CODEX instruction file.

---

### Sprint 1 — Foundation Fixes (Week 1, ~2 agents)

**Agent A: Tool completions — P0 blockers**
File: `CODEX_SPRINT1_TOOLS.md`

Priority order:
1. `exec` tool — PTY via `portable-pty`, background session store (HashMap keyed by sessionId), poll/log/write/send-keys, elevated/host/node routing stubs
2. `memory_search` — embed MEMORY.md + memory/*.md at startup with `fastembed-rs` (MiniLM-L6), cosine similarity, return top-N snippets with file+line citations
3. `memory_get` — safe line-range reader with path validation
4. `message` tool — dispatch to channel registry by channel name, send/react/delete/edit
5. `cron` tool — HTTP client calls to gateway scheduler (add/list/remove/run/runs/status/wake)
6. `session_status` — query current session context size, model, cost; return formatted card

**Agent B: Auto-reply depth**
File: `CODEX_SPRINT1_AUTOREPLY.md`

1. Full slash command parser (30 commands, type-safe enum)
2. Heartbeat response handling (HEARTBEAT_OK suppression)
3. NO_REPLY handling (suppress channel output)
4. Telegram MarkdownV2 complete escaper (all special chars)
5. Reaction dispatch through channel registry
6. Collect/debounce window (tokio timer, configurable ms)
7. Authorized sender check before any agent turn
8. Reply-to construction (Telegram `reply_parameters`, Discord reply ref)
9. Inbound media pipeline (photo/voice/document → route to tools)
10. AGENTS.md/SOUL.md injection into system prompt (full depth, not stubs)

**Duration**: 1 week | **Output**: Working end-to-end agent turns over Telegram

---

### Sprint 2 — Channel Completions (Week 2-3, ~3 agents)

**Agent A: Signal channel**
File: `CODEX_SPRINT2_SIGNAL.md`

- Integrate `presage` crate (Signal Rust implementation)
- Message receive loop, send, group support, media
- Monitor with reconnect logic
- Wire into channel framework registry

**Agent B: iMessage + WhatsApp hardening**
File: `CODEX_SPRINT2_IMSG_WA.md`

- iMessage: `osascript` bridge for Messages.app — send, receive monitor via AppleScript, group support
- WhatsApp: Upgrade 178-line stub to subprocess wrapper around `whatsmeow` Go binary, or implement full HTTP REST wrapper for whatsapp-web.js subprocess

**Agent C: Slack + LINE deepening**
File: `CODEX_SPRINT2_SLACK_LINE.md`

- Slack: Socket Mode event loop, blocks message builder, thread handling, slash commands, full monitor
- LINE: Messaging API webhooks, Flex messages, rich menus, send/receive full implementation

**Duration**: 2 weeks | **Output**: All 8 channels functional

---

### Sprint 3 — Agent Engine Depth (Week 3-4, ~2 agents)

**Agent A: Agent engine completions**
File: `CODEX_SPRINT3_AGENT.md`

1. Full workspace file injection pipeline (AGENTS.md, SOUL.md, USER.md, IDENTITY.md, TOOLS.md, HEARTBEAT.md, MEMORY.md with 14K char truncation logic)
2. Skills discovery + XML block injection (`<available_skills>`)
3. Project context injection (CLAUDE.md/AGENTS.md per project)
4. Compaction trigger: when context > 80% of model limit, flush memories then compact
5. Pre-compaction memory extractor (summarize session to MEMORY.md)
6. Multi-agent isolation (per-agent workspaces, configs, session scopes)
7. Announce delivery (cron/isolated job result → route to delivery.channel)
8. OAuth token refresh (OpenAI Codex OAuth, GitHub Copilot)

**Agent B: Session management completions**
File: `CODEX_SPRINT3_SESSIONS.md`

1. JSONL compaction (compact transcript, preserve memory flush, write new base)
2. Context window % calculator per provider (token counting via tiktoken-rs or character heuristic)
3. Session file locking (`.lock` file with PID, timeout-based release)
4. Session key resolver for all formats
5. `sessions_spawn` tool implementation — wire to ACP runtime, return sessionKey
6. `sessions_list/history/send` tool implementations — wire to session store

**Duration**: 2 weeks | **Output**: Full agent turn lifecycle functional

---

### Sprint 4 — CLI Wiring (Week 5, ~2 agents)

**Agent A: CLI gateway commands**
File: `CODEX_SPRINT4_CLI_GATEWAY.md`

Wire every CLI command to its implementation:
- `gateway start/stop/restart/status` → macOS LaunchAgent + Linux systemd generation
- `cron list/add/remove/run/runs/status` → scheduler HTTP calls
- `sessions list/show/delete/compact` → session ops
- `config get/set/unset/validate/file` → ConfigManager
- `security audit` → run security scanner, format report
- `channels login/logout/status` → per-provider auth CLI flows
- `agents list/add/remove` → agent registry
- `models list/set/test` → provider routing
- `logs` → tail log file with streaming output
- `status` → health/channel/session/model summary card

**Agent B: CLI advanced + TUI**
File: `CODEX_SPRINT4_CLI_ADVANCED.md`

- `approvals list/set/allow/deny` → approval store
- `plugins list/install/remove` → plugin registry
- `skills list/show` → skills discovery
- `memory search/get` → memory ops
- `message send` → channel dispatch
- `browser start/stop/status/tabs` → browser manager
- `nodes list/describe/notify` → node-host
- `docs` → serve/open docs site
- `tui` → Ratatui dashboard (agents, sessions, cron, logs, live updates)
- Shell completions (zsh, bash, fish)

**Duration**: 1 week | **Output**: Full CLI functional

---

### Sprint 5 — Tool Completions (Week 6, ~2 agents)

**Agent A: Media tools**
File: `CODEX_SPRINT5_MEDIA.md`

1. `browser` tool — wire `media::browser` into ToolContext; implement all actions (status/start/stop/profiles/tabs/open/focus/close/snapshot/screenshot/navigate/console/pdf/upload/dialog/act); snapshot with `refs=role|aria`
2. `canvas` tool — wire canvas host process; present/hide/navigate/eval/snapshot/a2ui_push/a2ui_reset
3. `tts` tool — wire ElevenLabs (`sag`) + OpenAI TTS; channel-format output (Telegram voice note vs text)
4. `image` tool — wire media understanding routing for image analysis
5. `pdf` tool — wire native Anthropic/Google PDF + text extraction fallback

**Agent B: Node + orchestration tools**
File: `CODEX_SPRINT5_NODES.md`

1. `nodes` tool — node-host HTTP protocol; status/describe/pending/approve/reject/notify/camera_snap/screen_record/location_get/notifications_list/run/invoke
2. `sessions_yield` tool — inject turn-end signal into session
3. `subagents` tool — list/steer/kill running sub-agents (query ACP runtime process table)
4. `agents_list` tool — enumerate available ACP harness IDs from config
5. `web_search` — harden existing (Brave API, handle 429, format snippets)
6. `web_fetch` — harden existing (readability extraction, maxChars truncation, timeout)

**Duration**: 1 week | **Output**: All 23+ tools functional

---

### Sprint 6 — Gateway Method Completions (Week 7, ~1 agent)

**Agent A: Gateway methods hardening**
File: `CODEX_SPRINT6_GATEWAY.md`

Current: 108 methods registered, many return stubs. Target: all return real data.

Priority groups:
1. `agent.*` (run, status, model, config, list) — wire to agent engine
2. `sessions.*` (list, show, delete, compact, history, send, spawn) — wire to session store
3. `cron.*` (add, list, remove, run, runs, status, wake, deadLetters) — wire to scheduler
4. `memory.*` (search, get) — wire to memory search
5. `channels.*` (status, list, send, react, delete) — wire to channel registry
6. `nodes.*` (list, describe, run, invoke, camera, screen, location) — wire to node-host
7. `browser.*` (status, tabs, snapshot, screenshot, act, navigate) — wire to media::browser
8. `gateway.*` (status, restart, config.get/patch/apply, update.run) — wire to service manager
9. `subagents.*` (list, steer, kill) — wire to ACP runtime
10. `approvals.*` (list, set, pending, respond) — wire to approvals store

**Duration**: 1 week | **Output**: Gateway 100% functional (108+ real methods)

---

### Sprint 7 — Control UI (Week 8, ~1 agent)

**Option A (fast, 3 days)**: Serve OpenClaw's React UI from gateway
- Copy OpenClaw's compiled Control UI assets into `gateway/static/`
- Patch API calls to route through `/call` method shim
- Ship static HTML fallback for core features
- Estimated effort: ~500 lines Rust + asset patching

**Option B (clean, 2 weeks)**: Leptos SPA
- Build in Rust/WASM for zero JS runtime dependency
- Full feature parity with OpenClaw Control UI
- Harder but future-proof

**Recommendation**: Ship Option A in Sprint 7, migrate to Option B post-launch.

---

### Sprint 8 — Testing + CI + Docs (Week 9-10, ~2 agents)

**Agent A: Parity test suite**
File: `CODEX_SPRINT8_TESTS.md`

1. Integration tests: launch gateway → send WS message → assert response
2. Channel round-trip tests (Telegram mock)
3. Tool execution tests (each tool with mocked external calls)
4. Parity sentinel CI: auto-run on every commit, fail if regressions
5. Compaction cycle test (create session → fill context → trigger compact → verify resume)
6. Cron execution test (schedule 1s job → verify fires → verify delivery)
7. Cross-compile CI: macOS ARM, Linux ARM/x86 (GitHub Actions matrix)

**Agent B: Docs generation**
File: `CODEX_SPRINT8_DOCS.md`

1. Build a docs generator: reads `parity/openclaw_docs_index.json`, maps each page to MagicMerlin equivalent
2. Generate 332 pages from templates — most are structural (install, concepts, CLI commands)
3. Auto-generate CLI reference from clap `#[command]` attributes
4. Auto-generate gateway method reference from SUPPORTED_METHODS
5. Auto-generate tool reference from ToolRegistry schema definitions
6. Publish docs site (MkDocs Material or mdBook) — host on GitHub Pages

**Duration**: 2 weeks | **Output**: 90%+ test coverage, 332 docs pages

---

## 4. Dependency Graph

```
Sprint 1 (Tools + Auto-Reply)
    ↓
Sprint 2 (Channels) ←→ Sprint 3 (Agent Engine)  [parallel]
    ↓
Sprint 4 (CLI)
    ↓
Sprint 5 (Tool Completions)
    ↓
Sprint 6 (Gateway Methods)
    ↓
Sprint 7 (Control UI)
    ↓
Sprint 8 (Tests + Docs)
    ↓
SHIP 1.0
```

Sprint 2 and Sprint 3 can run fully in parallel.  
Sprint 5 and Sprint 6 can overlap in the second half.

---

## 5. Estimated Completion Timeline

| Sprint | Focus | Agents | Duration | Lines Added |
|--------|-------|--------|----------|-------------|
| 1 | Tools + Auto-reply | 2 | 1 week | ~12,000 |
| 2 | Channel completions | 3 | 2 weeks | ~18,000 |
| 3 | Agent + Sessions | 2 | 2 weeks | ~10,000 |
| 4 | CLI wiring | 2 | 1 week | ~8,000 |
| 5 | Tool completions | 2 | 1 week | ~6,000 |
| 6 | Gateway methods | 1 | 1 week | ~5,000 |
| 7 | Control UI | 1 | 1 week | ~3,000 |
| 8 | Tests + Docs | 2 | 2 weeks | ~20,000 |
| **Total** | | **~16 agent-weeks** | **10 weeks** | **~82,000** |

**Final estimate: ~168,750 lines (~90% of target)**  
**Timeline with 2-3 parallel agents: ~7-8 weeks calendar time**

---

## 6. Crate Gaps to Create

These crates need to be created (currently absent):
- `memory` — embedding index, search, get (extracted from agent-tools stub)
- `node-host` — remote device protocol
- `canvas-host` — canvas rendering process manager
- `tui` — Ratatui terminal dashboard (can be in `cli` as a module)
- `docs` — auto-generated docs site builder (build-time tool, not runtime)

---

## 7. External Dependencies to Add

```toml
# PTY support for exec
portable-pty = "0.8"

# Signal channel
presage = { git = "https://github.com/whisperfish/presage" }
presage-store-sled = { git = "https://github.com/whisperfish/presage" }

# Discord full support (tranche 2+)
serenity = { version = "0.12", features = ["gateway", "model", "cache"] }

# Memory embeddings
fastembed = "3"

# TUI
ratatui = "0.27"

# Token counting
tiktoken-rs = "0.5"

# QR codes
qrcode = "0.14"
image = "0.25"  # QR → PNG

# Browser CDP
chromiumoxide = "0.7"

# Markdown
pulldown-cmark = "0.11"

# Slack
slack-morphism = "2"

# LINE
line-bot-sdk-rust = "0.1"  # or manual HTTP client
```

---

## 8. Definition of Done — v1.0

### Functional Parity Checklist

**Runtime**
- [ ] Agent turns work end-to-end (inbound message → tool calls → reply sent to channel)
- [ ] All 23+ tools execute with real implementations (no stubs)
- [ ] Memory search works with semantic embeddings
- [ ] Compaction cycle works (fill → compact → resume without context loss)
- [ ] Heartbeat system works (fires on schedule, HEARTBEAT_OK suppressed)
- [ ] Cron fires on schedule, delivers to correct channel
- [ ] Sub-agent spawn/steer/kill works via ACP runtime
- [ ] All slash commands handled (/status, /compact, /model, /sessions, etc.)

**Channels**
- [ ] Telegram: messages, inline buttons, media, voice, groups, reactions ✅ (almost there)
- [ ] Discord: messages, threads, embeds, slash commands, voice ✅ (tranche 1 + more needed)
- [ ] Signal: send/receive, groups, media
- [ ] WhatsApp: send/receive, groups, media, QR pairing
- [ ] Slack: Socket Mode, blocks, threads
- [ ] iMessage: send/receive, groups (macOS)
- [ ] LINE: Flex messages, webhooks
- [ ] Web: WebSocket relay

**CLI**
- [ ] All 257 commands wired to real implementations
- [ ] `magicmerlin gateway start` → daemon running
- [ ] `magicmerlin status` → health card
- [ ] `magicmerlin tui` → terminal dashboard
- [ ] `magicmerlin cron list` → shows live cron jobs

**Config Compatibility**
- [ ] `openclaw.json` works verbatim in Magic Merlin
- [ ] Session JSONL format compatible (can replay OpenClaw sessions)
- [ ] Cron import from OpenClaw format works

**Quality**
- [ ] Parity sentinel shows 0 critical diffs
- [ ] 90%+ test coverage on core crates
- [ ] Cross-platform builds pass: macOS ARM, Linux ARM/x86
- [ ] Docker image published
- [ ] 332 docs pages generated

**Packaging**
- [ ] `cargo install --git ... magicmerlin` works
- [ ] Homebrew formula draft
- [ ] GitHub Releases with prebuilt binaries (via `cargo-dist`)

---

## 9. First Move (Monday)

Spawn 2 Claude Code agents:

**Agent 1 (Tools)**:
> "Read CODEX_SPRINT1_TOOLS.md and implement all tasks in order. Work in ~/Projects/magicmerlin. Focus on: (1) real PTY exec via portable-pty, (2) memory_search with fastembed, (3) message tool wired to channel registry, (4) cron tool wired to gateway HTTP, (5) session_status tool. Make it compile clean. Run cargo test."

**Agent 2 (Auto-reply)**:
> "Read CODEX_SPRINT1_AUTOREPLY.md and implement all tasks in order. Work in ~/Projects/magicmerlin. Focus on: (1) full slash command parser with 30+ commands, (2) HEARTBEAT_OK + NO_REPLY suppression, (3) Telegram MarkdownV2 complete escaper, (4) collect/debounce window, (5) authorized sender enforcement, (6) workspace file injection into system prompt. Make it compile clean."

Create the CODEX instruction files before spawning.

---

## 10. Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Signal protocol complexity | Use `presage` (Rust) — active project, works |
| WhatsApp protocol complexity | Wrap `whatsmeow` (Go) as subprocess, pass JSON |
| Browser CDP from Rust | `chromiumoxide` is mature; fall back to Playwright subprocess if needed |
| Mac OOM with 4 parallel agents | Max 2 local Claude Code, use Codex cloud for 3rd/4th agent |
| iMessage macOS-only | Feature-flag with `#[cfg(target_os = "macos")]`, skip on Linux |
| Discord sharding complexity | Start single-shard, add sharding in v1.1 |
| Control UI rebuild cost | Serve OpenClaw UI statically in v1.0, rebuild in v1.1 |
| Docs 332 pages | Use AI generation — 80% of docs pages are structured/templated |

---

*This strategy supersedes ENGINEERING_PLAN.md (2026-03-06) which was written before Phases 1-7 and Parity Passes 1-7 were completed. Re-read current crate state before spawning agents.*

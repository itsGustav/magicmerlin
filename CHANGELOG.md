# Changelog

## [1.0.0] - 2026-04-04

### Added
- Full AgentEngine integration — real LLM loop with tool execution (no more echo stub)
- Telegram channel end-to-end wiring (inbound → agent → reply)
- Discord: threads, embeds, slash commands, full event loop
- Signal channel (signal-cli subprocess bridge)
- WhatsApp channel (subprocess bridge + Cloud API fallback)
- iMessage channel (macOS, osascript bridge)
- Semantic memory search via fastembed (AllMiniLML6V2, cosine similarity)
- Session compaction with automatic memory extraction to daily files
- Multi-agent config support (named agents: merlin, henry, paylobster, lobsterprime)
- All 257 CLI commands wired — zero unimplemented!()
- All gateway methods returning real data — zero stubs
- ChannelRegistry integrated into AppState for live channel dispatch
- HEARTBEAT_OK and NO_REPLY reply suppression
- [[reply_to_current]] / [[reply_to:<id>]] tag extraction and Telegram reply threading
- DmGate / authorized sender enforcement
- Control UI: 8-tab dark SPA with live polling
- Session compaction with memory candidate extraction
- Docker multi-stage image (debian:bookworm-slim)
- GitHub Actions CI matrix (macOS ARM + Linux x86)
- OpenClaw openclaw.json config file compatibility (parse real configs)
- 13,198 tests passing
- 331 docs pages generated

### Changed
- Version 0.0.0 → 1.0.0
- Config model: removed deny_unknown_fields; bindings accepts array or object
- AgentDefaults.model: accepts plain string or OpenClaw object {primary, fallbacks}

### Fixed
- Session file locking
- Memory search upgraded from BM25 to fastembed semantic embeddings
- web_fetch real HTML extraction via scraper
- TTS audio delivery via OpenAI audio speech API
- Config parsing of real openclaw.json files

## [0.2.0] - 2026-03-08

### Added
- New `magicmerlin-plugins` crate:
  - Plugin trait and lifecycle runtime (`init/start/stop`)
  - Bundled plugins (`session-memory`, `command-logger`, `boot-md`, `bootstrap-extra-files`)
  - Plugin discovery and manifest scanning
  - Plugin registry with enable/disable and isolated config namespaces
  - Skills subsystem: discovery, `SKILL.md` metadata parsing, dependency checks, XML prompt block generation, script execution
- New `magicmerlin-acp` crate:
  - ACP runtime for spawning external coding-agent subprocesses
  - Session control plane with event streaming and persistent thread-bound sessions
  - ACPX dispatch integration and harness policy config (`allowedAgents`, `maxConcurrentSessions`, `ttlSeconds`)
- Gateway integration:
  - ACP endpoints and JSON-RPC methods (`acp.spawn`, `acp.sessions.list`, `acp.cleanup`)
  - Embedded Control UI with overview/sessions/cron/config/logs pages
  - Live event log polling endpoint (`/events`)
  - Security audit endpoint (`/security/audit`) and RPC method (`security.audit`)
- Security module in `magicmerlin-config`:
  - Audit checks for DM policy, sandbox presence, weak auth, exposed bind, stale sessions, trusted proxy validation
  - Workspace path restriction validation helper
  - Tool deny-list helper per agent/global scope

### Changed
- `gateway` plugin access now delegates to the new `magicmerlin-plugins` crate.
- CLI `security audit` now calls gateway `security.audit` instead of returning placeholder data.
- README rewritten with installation, quick start, migration, architecture, and full CLI command map.

### Testing
- Workspace test count increased to 85.
- Added extensive unit coverage for plugins, skills, ACP runtime, and security auditing.

[0.2.0]: https://example.invalid/magicmerlin/releases/0.2.0

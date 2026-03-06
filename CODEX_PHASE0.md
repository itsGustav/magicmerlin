# Phase 0: Foundation — Build Instructions

You are building Magic Merlin, a 100% Rust replacement for OpenClaw (a TypeScript AI agent runtime). 
This is Phase 0: Foundation infrastructure that everything else depends on.

## What Already Exists
- Workspace with 5 crates: cli, gateway, compat, sentinel, tools
- Gateway has: axum HTTP server, cron scheduler, pairing, sessions, approvals, plugins (basic)
- ~6K lines of Rust total, compiles clean

## What To Build (Phase 0)

### 0.1 Configuration System (new crate: `config`)
Create a new `config` crate in the workspace. It must:
1. Load `openclaw.json` (JSON format) from `~/.openclaw/openclaw.json` (or `OPENCLAW_CONFIG_PATH` env)
2. Parse into strongly-typed Rust structs using serde. Top-level keys: `meta`, `wizard`, `auth`, `acp`, `models`, `agents`, `tools`, `bindings`, `messages`, `commands`, `channels`, `talk`, `gateway`, `skills`, `plugins`
3. Support `config get <path>` (dot-notation like `agents.defaults.model`), `config set <path> <value>`, `config unset <path>`
4. Environment variable overlay: any `OPENCLAW_*` env var overrides config (e.g., `OPENCLAW_GATEWAY_PORT=18789`)
5. Profile isolation: `--profile <name>` uses `~/.openclaw-<name>/` instead of `~/.openclaw/`
6. `--dev` flag uses `~/.openclaw-dev/` with port 19001
7. State directory helpers: resolve paths for agents, sessions, logs, media, secrets within state dir
8. Secrets loader: read `~/.openclaw/secrets.env` (KEY=VALUE format), expose via `secrets::get("KEY")`
9. Validation: reject unknown keys, validate types, ranges

### 0.2 Logging (new crate: `logging`)
Create a `logging` crate:
1. Use `tracing` + `tracing-subscriber` for structured logging
2. Levels: silent, fatal, error, warn, info, debug, trace
3. Console output (with optional ANSI colors, respecting `--no-color`)
4. File output to `~/.openclaw/logs/gateway.log` and `gateway.err.log`
5. Log rotation (by size, keep last 5)
6. `--log-level <level>` CLI flag support
7. Initialization helper: `logging::init(level, color, file_path)`

### 0.3 Infrastructure Utilities (new crate: `infra`)
Create an `infra` crate with:
1. HTTP client wrapper (reqwest-based) with:
   - Default timeouts (30s connect, 120s read)
   - TLS configuration
   - Proxy support (HTTP_PROXY, HTTPS_PROXY env)
   - User-Agent header ("MagicMerlin/0.1")
   - JSON request/response helpers
2. Time formatting utilities:
   - Human-readable durations ("2h 15m ago")
   - ISO 8601 formatting
   - Timezone-aware display
3. Text utilities:
   - Truncation with ellipsis (respecting word boundaries)
   - Markdown stripping
   - JSON pretty-printing
   - Base64 encode/decode
   - Unicode sanitization
4. Markdown processing:
   - Parse markdown to extract sections
   - Strip markdown to plain text
   - Extract frontmatter (YAML)

### 0.4 Storage Layer (new crate: `storage`)
Create a `storage` crate:
1. SQLite database manager:
   - Connection pool (r2d2 or deadpool)
   - Auto-migration on startup
   - Tables: sessions, cron_jobs, cron_runs, approvals, plugins, dead_letters
   - Schema matches what gateway/src/scheduler.rs already uses (extend it)
2. JSONL transcript handler:
   - Read/write JSONL files (one JSON object per line)
   - Append-only writing with fsync
   - Read with offset/limit
   - Compact: summarize old messages, rewrite file
   - Repair: detect and fix broken tool_use/tool_result pairs (this is a real bug we hit!)
   - Count tokens per message (approximate)
3. Session file locking:
   - PID-based lock files (`.jsonl.lock`)
   - Stale lock detection (check if PID is alive)
   - Lock acquisition with timeout
4. Memory file manager:
   - Read/write MEMORY.md
   - Read/write daily files (`memory/YYYY-MM-DD.md`)
   - Append-only daily log entries
   - Create memory/ directory if needed

### Integration
- Add all new crates to workspace Cargo.toml
- Wire config loading into existing gateway main.rs
- Wire logging into existing gateway
- Add basic integration tests for each crate
- All code must compile clean with `cargo check`
- Run `cargo test` and ensure all tests pass
- Use these specific crate versions where applicable:
  - serde = "1" + serde_json = "1"
  - tracing = "0.1" + tracing-subscriber = "0.3"
  - reqwest = "0.12" (already in workspace)
  - rusqlite = "0.32" (already in workspace)
  - pulldown-cmark = "0.12"
  - chrono = "0.4"
  - base64 = "0.22"
  - dotenv = "0.15" (for secrets.env)

## Quality Requirements
- Every public function has a doc comment
- Every module has a module-level doc comment
- Error handling uses `thiserror` for custom errors, `anyhow` for application errors
- No unwrap() in library code (only in tests and main)
- Clippy clean (`cargo clippy -- -D warnings`)

When completely finished, run this command to notify me:
openclaw system event --text "Phase 0 complete: Foundation crates (config, logging, infra, storage) built and tested" --mode now

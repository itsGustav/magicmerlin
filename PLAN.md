# MagicMerlin — Milestone Plan (Parity v0 → v1)

**Mode:** compat-first. No feature work until we can prove (and continuously test) compatibility against OpenClaw snapshots.

## Definitions

- **v0 (scaffold):** repo structure + compatibility contract + frozen snapshots.
- **v1 (parity substrate):** minimal runnable components + snapshot-driven models + adapter interfaces so we can safely iterate toward deeper parity.

## Principles

1. **Snapshots are the contract** (until we have a formal spec).
2. **Adapters over re-implementation:** first build a thin translation layer that can map OpenClaw concepts to MagicMerlin internals.
3. **Tolerant readers:** accept unknown fields for forward-compat.
4. **No silent drift:** any snapshot change requires an explicit diff + version bump.

---

## v0 — Scaffold (DONE by this kickoff)

Deliverables:
- Cargo workspace with crates: `gateway`, `compat`, `tools`, `sentinel`
- `compat/OPENCLAW_COMPATIBILITY_CONTRACT.md`
- `compat/snapshots/` with:
  - `openclaw --help`
  - `openclaw cron --help`
  - `openclaw status --json`
  - runtime tool surface snapshot (from prompt)

Acceptance:
- Repo layout exists and is readable.
- Snapshots are committed as raw captures.

---

## v0.1 — Snapshot model (data-only)

Goal: represent OpenClaw’s surface area as typed Rust structs **without** enforcing strict parsing yet.

Tasks:
- Add `serde` + `serde_json` to `magicmerlin-compat`.
- Define types:
  - `CliHelpSnapshot` (command tree nodes + raw text)
  - `CronHelpSnapshot` (raw)
  - `StatusSnapshot` (partial typed + `serde_json::Value` passthrough)
  - `ToolSurfaceSnapshot` (tool list; schema as relaxed JSON)
- Implement loaders that read from `compat/snapshots/`.

Acceptance:
- `magicmerlin-compat` can load all snapshot files and print a short summary (counts, versions).

---

## v0.2 — Diff + drift detection

Goal: make compatibility drift impossible to miss.

Tasks:
- Add a tiny CLI command (in `gateway` or a new `magicmerlin` bin) that:
  - validates snapshot presence
  - computes hashes (sha256) per snapshot
  - prints a single "compat fingerprint" (hash-of-hashes)
- Add a `SNAPSHOT_UPDATE.md` describing how to recapture.

Acceptance:
- One command produces a stable fingerprint for the current snapshot set.

---

## v0.3 — Compat adapter interfaces

Goal: lock the interfaces we’ll implement to emulate OpenClaw.

Tasks:
- Define traits/interfaces (in `compat`):
  - `StatusProvider`
  - `CronProvider`
  - `ToolRegistryProvider`
  - `CliProvider`
- Provide a `SnapshotBacked*Provider` implementation that returns data from snapshot files.

Acceptance:
- We can instantiate providers and serve their data without any "real" runtime.

---

## v0.4 — Minimal gateway process

Goal: have something runnable that exposes the compat substrate.

Tasks:
- In `magicmerlin-gateway`, add:
  - `--version`
  - `--print-compat` (prints compat version + fingerprint)
  - `--serve` (optional): HTTP server exposing `/health`, `/status`, `/tools` (snapshot-backed)

Acceptance:
- Running the gateway prints deterministic info derived from snapshots.

---

## v0.5 — First “real” wedge: scheduler + cron + Discord webhook

Goal: move from snapshot-only to *operational utility* without breaking compat.

Deliverables:
- SQLite-backed cron job store (`magicmerlin.db` by default; override `MAGICMERLIN_DB_PATH`).
- CLI commands: `status`, `cron list/add/remove/run`.
- Daemon mode: `--serve <port> --daemon` runs HTTP API + scheduler loop.
- HTTP endpoints: `GET /cron`, `POST /cron/run/:id`.
- Discord outbound: `discord_webhook` job kind posts to a Discord webhook URL.

### Runbook (copy/paste)

Build + run compat fingerprint:
```bash
cargo run -p magicmerlin-gateway -- --print-compat --json
```

List jobs:
```bash
cargo run -p magicmerlin-gateway -- cron list --json
```

Add a Discord webhook job (every 5 seconds):
```bash
cargo run -p magicmerlin-gateway -- cron add \
  --name "hello-discord" \
  --schedule "*/5 * * * * *" \
  --kind discord_webhook \
  --payload '{"webhook_url":"https://discord.com/api/webhooks/XXX/YYY","content":"hello from MagicMerlin"}'
```

Run a job immediately:
```bash
cargo run -p magicmerlin-gateway -- cron run 1
```

Start daemon (HTTP + scheduler):
```bash
# localhost only:
cargo run -p magicmerlin-gateway -- --serve 8099 --daemon

# or LAN-accessible:
cargo run -p magicmerlin-gateway -- --serve 8099 --bind 0.0.0.0 --daemon

# then:
curl -s http://127.0.0.1:8099/cron | jq
curl -s -X POST http://127.0.0.1:8099/cron/run/1 | jq
```

---

## v0.6 — Cron parity + OpenClaw import + agent_turn shell compat

Goal: establish MagicMerlin as a credible OpenClaw cron fallback with CLI parity,
OpenClaw job import, extended schedule parsing, run history, and a minimal
agent-turn shell execution wedge.

Deliverables:
- **CLI parity shim:** OpenClaw-like cron verb aliases — `add`, `list`, `edit`,
  `enable`/`disable`, `rm`, `run`, `runs`, `status`.
  - `enable` → `resume`, `disable` → `pause`, `rm` → `remove`.
  - `edit` patches a job: update name, schedule, kind, payload, maxAttempts, backoffSeconds.
  - `status` prints scheduler state (jobCount, nextRunAt) with `--json`.
  - `runs` shows recent run history from the new `runs` SQLite table.
- **Schedule parsing extensions:**
  - Plain cron (5-field auto-normalized to 6-field by prepending seconds=0).
  - Interval: `every:<seconds>@<anchor_ts>` (anchor optional; defaults to now).
  - Cron + timezone: `cron:<expr>@<IANA_tz>` via chrono-tz.
- **Runs table:** SQLite `runs` table recording every job execution (started_at,
  ended_at, status, error, metadata JSON). Backward-compatible schema migration.
- **OpenClaw cron importer:** `cron import-openclaw --file <path>` or `--stdin`.
  Parses `openclaw cron list --json` output and maps:
  - `schedule.kind=every` → `every:<s>@<anchor>`
  - `schedule.kind=cron` → `cron:<expr>@<tz>` (or plain if no tz)
  - `payload.kind=agentTurn` → kind `agent_turn` with `{message, timeoutSeconds, model?, thinking?}`
- **Agent-turn compat runner (Phase 2 wedge):** job kind `agent_turn`.
  - Executes shell commands from `payload.message` only if prefixed with `Run:` or `Run `.
  - Gated behind `MAGICMERLIN_ALLOW_SHELL=1`.
  - Enforces `timeoutSeconds`, captures stdout/stderr (truncated to 8 KB each),
    stores in `runs.metadata`.
- Retry with exponential backoff + dead-letter queue (carried forward from v0.5).
- Optional API key protection for cron HTTP endpoints (carried forward).

### Runbook

```bash
# Edit a job
cargo run -p magicmerlin-gateway -- cron edit 1 --schedule "0 */10 * * * *" --name "renamed"

# OpenClaw-like aliases
cargo run -p magicmerlin-gateway -- cron enable 1
cargo run -p magicmerlin-gateway -- cron disable 1
cargo run -p magicmerlin-gateway -- cron rm 1

# Scheduler status
cargo run -p magicmerlin-gateway -- cron status --json

# Run history
cargo run -p magicmerlin-gateway -- cron runs --json
cargo run -p magicmerlin-gateway -- cron runs --job-id 1 --limit 10

# Import from OpenClaw
openclaw cron list --json | cargo run -p magicmerlin-gateway -- cron import-openclaw --stdin
cargo run -p magicmerlin-gateway -- cron import-openclaw --file ./oc-crons.json

# Agent-turn shell jobs (requires env gate)
MAGICMERLIN_ALLOW_SHELL=1 cargo run -p magicmerlin-gateway -- --serve 8099 --daemon
```

---

## v0.7 — Operability + portability (DONE — merged into v0.5/v0.6)

Deliverables (all shipped earlier than planned):
- `cron export --file <path>` and `cron import --file <path> [--replace]`.
- New job kind: `discord_bot` (Discord Bot API send-message mode).
- CLI and daemon options for deployment flexibility: `--bind`, `--db-path`.

### Import / export
```bash
cargo run -p magicmerlin-gateway -- cron export --file ./jobs.json
cargo run -p magicmerlin-gateway -- cron import --file ./jobs.json --replace
```

### Discord bot job
Payload requires `channel_id` and either `bot_token` or env var `MAGICMERLIN_DISCORD_BOT_TOKEN`.
```bash
cargo run -p magicmerlin-gateway -- cron add \
  --name "bot-msg" \
  --schedule "0 */5 * * * *" \
  --kind discord_bot \
  --payload '{"channel_id":"1234567890","content":"hello from MagicMerlin"}'
```

---

## v1 — Parity substrate complete

Goal: MagicMerlin has a stable "OpenClaw-shaped" surface to build on.

Deliverables:
- Snapshot-backed providers are stable + tested.
- Gateway can expose status + tools derived from snapshots.
- A documented process exists for updating snapshots and bumping compat version.

Acceptance criteria:
- CI-style check (local for now) verifies:
  - all required snapshot files exist
  - loaders parse/validate
  - fingerprint matches expected value
- Human review checklist for any snapshot update is written.

---

## Next after v1 (not in scope here)

- Replace snapshot-backed providers with real implementations incrementally.
- Add CLI command parity starting with: `status`, `cron`, and a tool registry export.
- Formalize a machine-readable tool schema export.

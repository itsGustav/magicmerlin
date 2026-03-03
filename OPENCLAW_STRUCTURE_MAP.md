# OpenClaw → MagicMerlin structure map

Source repo: https://github.com/openclaw/openclaw (MIT)

This doc maps **OpenClaw’s** repository structure to the **MagicMerlin** Rust workspace structure, so we can track full-clone parity while still innovating.

> MagicMerlin does **not** need to copy file-for-file. The goal is: identical *capabilities* + compatible CLI/HTTP surfaces, backed by a safer/faster Rust runtime.

## OpenClaw top-level (high signal)
OpenClaw is a pnpm monorepo. Key top-level directories:
- `src/` — main TypeScript runtime (gateway, agents, channels, cron, security, plugins, etc.)
- `extensions/` — channel/provider extensions (telegram, slack, discord, whatsapp, …)
- `skills/` — packaged skills (CLI wrappers and local tools)
- `ui/` + `apps/` — dashboard/control UI + mobile/desktop apps
- `docs/` — docs site (mint)

## Target MagicMerlin workspace layout (Rust)
Current MagicMerlin crates:
- `magicmerlin` (CLI) ✅
- `magicmerlin-gateway` ✅
- `magicmerlin-compat` ✅
- `magicmerlin-tools` ✅
- `magicmerlin-sentinel` (stub) ⚠️

Planned crates/modules to reach full-clone parity:

### CLI / commands
OpenClaw:
- `openclaw.mjs` (node bootstrap)
- `src/cli/*`, `src/commands/*`

MagicMerlin:
- `cli/` (binary: `magicmerlin`) ✅
- `cli` subcommands to be added: `models`, `channels`, `pairing`, `nodes`, `browser`, `update`, `logs`, `security`, `secrets`

### Gateway runtime
OpenClaw:
- `src/gateway/*` (HTTP server, WS, Control UI routes, auth, rate limits)

MagicMerlin:
- `gateway/` (axum) ✅
- Expand to: websockets/SSE streaming, auth tokens, trusted proxies, rate limiting, reverse-proxy hardening

### Sessions / agents
OpenClaw:
- `src/sessions/*`, `src/agents/*`

MagicMerlin:
- `gateway/src/sessions.rs` ✅ (basic session DB)
- Add: agent registry, per-peer DM scoping, per-agent model policies, usage metering

### Cron / scheduler
OpenClaw:
- `src/cron/*`

MagicMerlin:
- `gateway/src/scheduler.rs` ✅
- Already ahead in some areas: runs table + dead letters + portable import/export + OpenClaw cron importer

### Channels + transports
OpenClaw:
- `src/channels/*` + `extensions/*` (telegram/slack/discord/etc)

MagicMerlin:
- planned `channels/` crate: core channel abstractions + transport engine
- planned per-channel crates or plugins:
  - `channels-telegram`
  - `channels-discord`
  - `channels-slack`

### Pairing / allowlists
OpenClaw:
- `src/pairing/*` + `src/channels/allowlists/*`

MagicMerlin:
- planned `pairing/` crate: DM gating, allowlists, per-channel-peer session scoping

### Browser + nodes
OpenClaw:
- `src/browser/*`, `src/node-host/*`, `src/nodes/*`

MagicMerlin:
- planned `browser/` crate (Playwright or CDP proxy)
- planned `nodes/` crate (pairing + remote exec/camera/screen/canvas)

### Plugins / tool surface
OpenClaw:
- `src/plugins/*`, `src/plugin-sdk/*`, `extensions/*`

MagicMerlin:
- `gateway/src/plugins.rs` ✅ (local registry stub)
- planned `plugin-sdk/` crate: typed tool schemas + stable RPC
- innovation target: WASM tool plugins (signed + sandboxed)

### Secrets + security
OpenClaw:
- `src/secrets/*`, `src/security/*`

MagicMerlin:
- `gateway/src/approvals.rs` ✅ (approvals/allowlist persisted)
- planned `secrets/` crate: SecretRefs, precedence, reload
- planned `security/` crate: audits + policy enforcement

## What we already implemented from OpenClaw "Getting Started"
- Local Control UI (`/`) + chat endpoint (`/chat`) backed by Codex CLI (OAuth) ✅
- `magicmerlin onboard`, `magicmerlin gateway status/run`, `magicmerlin dashboard` ✅
- State/config env vars (`MAGICMERLIN_*`) ✅

## Next parity milestone
- Implement the *minimum* of OpenClaw’s `channels + pairing + dmScope` so MagicMerlin can fully replace DM flows (Telegram-first).

# Getting Started parity (OpenClaw → MagicMerlin)

Reference: https://docs.openclaw.ai/start/getting-started

Goal: ensure the **Getting Started** experience works in MagicMerlin so we can operate without OpenClaw.

## 1) Fastest chat: Control UI (no channel setup)
- [ ] `magicmerlin dashboard` opens a local Control UI
- [ ] Gateway serves `/` (static HTML)
- [ ] Gateway supports `POST /chat` (local chat endpoint)
- [ ] Chat backend uses **Codex CLI** (ChatGPT OAuth) by default

## 2) Prereqs
OpenClaw requires Node 22+. MagicMerlin is Rust-first.
- [ ] Document Rust + binary install prereqs
- [ ] (Optional) Provide install scripts (bash + PowerShell)

## 3) Quick setup (CLI)
OpenClaw steps:
- install
- `openclaw onboard --install-daemon`
- `openclaw gateway status`
- `openclaw dashboard`

MagicMerlin equivalents:
- [ ] `magicmerlin onboard --install-daemon`
- [ ] `magicmerlin gateway status`
- [ ] `magicmerlin dashboard`

## 4) Optional checks and extras
- [ ] Run gateway in foreground: `magicmerlin gateway --port 18789`
- [ ] Send a test message: `magicmerlin message send ...` (minimal local + webhook targets)

## 5) Useful environment variables
OpenClaw:
- `OPENCLAW_HOME`
- `OPENCLAW_STATE_DIR`
- `OPENCLAW_CONFIG_PATH`

MagicMerlin:
- [ ] `MAGICMERLIN_HOME`
- [ ] `MAGICMERLIN_STATE_DIR`
- [ ] `MAGICMERLIN_CONFIG_PATH`

## 6) What you will have
- [ ] A running MagicMerlin gateway
- [ ] A working local chat UI (no channel)
- [ ] A configured state dir + database

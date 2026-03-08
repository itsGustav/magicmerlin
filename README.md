# MagicMerlin

MagicMerlin is a Rust-first, OpenClaw-shaped runtime with a typed gateway, plugin/skill extension system, ACP subprocess control plane, and compatibility-focused CLI.

## Installation

### Cargo (from source)
```bash
git clone <your-fork-or-origin> magicmerlin
cd magicmerlin
cargo install --path cli --locked
cargo install --path gateway --locked
```

### Docker (local image build)
```bash
docker build -t magicmerlin:local -f - . <<'DOCKER'
FROM rust:1.76 as build
WORKDIR /app
COPY . .
RUN cargo build --release -p magicmerlin -p magicmerlin-gateway

FROM debian:bookworm-slim
RUN useradd -m merlin
USER merlin
WORKDIR /home/merlin
COPY --from=build /app/target/release/magicmerlin /usr/local/bin/magicmerlin
COPY --from=build /app/target/release/magicmerlin-gateway /usr/local/bin/magicmerlin-gateway
ENTRYPOINT ["magicmerlin"]
DOCKER
```

## Quick Start

```bash
# 1) Start gateway
magicmerlin-gateway --serve 18789 --bind 127.0.0.1 --daemon

# 2) Open dashboard UI
magicmerlin dashboard

# 3) Check status
magicmerlin status

# 4) Run an agent turn
magicmerlin agent run "Summarize today's pending cron jobs"
```

## Migration from OpenClaw

1. Keep your OpenClaw profile and state directories as-is.
2. Set `OPENCLAW_STATE_DIR` and optionally `OPENCLAW_CONFIG_PATH` to your existing paths.
3. Import OpenClaw cron jobs:
   `magicmerlin-gateway cron import-openclaw --file /path/to/openclaw-cron-list.json`
4. Validate compatibility snapshots:
   `cargo run -p magicmerlin-sentinel -- methods-diff --openclaw-src ../tmp/openclaw-src`
5. Run `magicmerlin status` and `magicmerlin security audit` before exposing any public channel bindings.

## Architecture Overview

- `cli`: user-facing command surface (`magicmerlin`)
- `gateway`: HTTP/JSON-RPC runtime, scheduler, control UI, pairing/session APIs
- `plugins`: plugin lifecycle + registry + skill discovery/injection/execution
- `acp`: Agent Control Protocol runtime for subprocess coding-agent sessions
- `config`: typed config model, env overlays, security audit helpers
- `providers`, `agent`, `sessions`, `storage`, `channels`, `media`: model routing and execution stack
- `compat` + `sentinel`: compatibility snapshots and drift detection

## Control UI

The gateway root (`/`) serves an embedded dark-theme control dashboard with pages for:

- Overview (health, scheduler, compat fingerprint, ACP session counts)
- Sessions (list/view/compact/delete)
- Cron (jobs + run history + enable/disable/run)
- Config (get/set/unset paths)
- Logs (live event polling from `/events`)

## CLI Commands

Top-level commands currently exposed by `magicmerlin`:

- `status`, `setup`, `onboard`, `health`, `dashboard`, `tui`
- `completion`, `version`, `update`, `reset`, `help`
- `agent`, `agents`, `models`, `gateway`, `daemon`
- `channels`, `message`, `directory`, `pairing`, `sessions`
- `memory`, `cron`, `logs`, `hooks`, `config`, `security`
- `secrets`, `sandbox`, `approvals`, `plugins`, `skills`
- `dns`, `devices`, `nodes`, `qr`, `browser`, `acp`, `docs`, `system`

For command-specific help:
```bash
magicmerlin <command> --help
```

## Testing and Quality

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Current workspace test target is 80+ tests; this branch includes 85.

## Parity Utilities

```bash
bash scripts/recapture_openclaw_snapshots.sh
cargo run -p magicmerlin-sentinel -- methods-diff --openclaw-src ../tmp/openclaw-src
cargo run -p magicmerlin-sentinel -- cli-diff --openclaw-help-tree compat/snapshots/2026-03-02_openclaw_help_tree.json
```

## License

MIT (see `LICENSE`).

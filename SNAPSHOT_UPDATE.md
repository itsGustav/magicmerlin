# MagicMerlin — Snapshot Update Procedure

MagicMerlin is **compat-first**. The snapshot set under `magicmerlin/compat/snapshots/` is the golden reference for OpenClaw’s surface area.

## TL;DR

```bash
cd /Users/gustav/.openclaw/workspace-main
bash magicmerlin/scripts/recapture_openclaw_snapshots.sh

# Verify loaders + fingerprint
cd magicmerlin
. "$HOME/.cargo/env"
cargo test
cargo run -p magicmerlin-gateway -- --print-compat --json
```

## What gets captured automatically

The recapture script updates:
- `openclaw --help`
- `openclaw cron --help`
- `openclaw status --json`
- `openclaw --version`

It also regenerates:
- `manifest.json` (points to the current snapshot files)
- `*_openclaw_status.header.txt` (metadata)

## Manual snapshot

`*_runtime_tools_surface.md` is currently a **manual copy** (from the running agent’s tool definitions).

Until OpenClaw exposes a CLI/API to export the tool schema surface deterministically, this file is updated manually.

## Change management rules

- Snapshots should be treated as **raw outputs**.
- Any snapshot update must be accompanied by:
  1) a diff review
  2) an updated fingerprint (`magicmerlin-gateway --print-compat`)
  3) a compat version bump **only if** the compatibility contract changes materially.

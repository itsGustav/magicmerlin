# MagicMerlin

MagicMerlin is an **independent** Rust project that aims to provide an OpenClaw-compatible surface ("compat-first") and a production-grade scheduler wedge.

**Not affiliated with PayLobster.**

## What it is (today)
- Snapshot-backed compatibility layer (`magicmerlin-compat`)
- Gateway + SQLite cron scheduler wedge (`magicmerlin-gateway`)

## Compatibility
Compatibility is defined by captured snapshots in `magicmerlin/compat/snapshots/` and enforced via hashes + a stable fingerprint.

Update snapshots:
```bash
cd /Users/gustav/.openclaw/workspace-main
# Fast snapshots (help/cron/status/version) + manifest hashes
bash magicmerlin/scripts/recapture_openclaw_snapshots.sh

# Optional: full CLI help tree (200+ commands)
MAGICMERLIN_CAPTURE_HELP_TREE=1 bash magicmerlin/scripts/recapture_openclaw_snapshots.sh
```

Verify:
```bash
cd magicmerlin
cargo test
cargo run -p magicmerlin-gateway -- --print-compat --json
```

# MagicMerlin

MagicMerlin is a **Rust-first, OpenClaw-shaped** runtime.

- **Goal:** a full OpenClaw-compatible clone (CLI + gateway surface), but *better* (typed, deterministic, safer).
- **Status:** active build.

## Non-affiliations
- **Not affiliated with OpenClaw** (separate project)
- **Not affiliated with PayLobster**

## What it is (today)
- Snapshot-backed compatibility layer (`magicmerlin-compat`)
- Gateway + scheduler wedge (`magicmerlin-gateway`) with:
  - cron parity shim + OpenClaw cron importer
  - run history + retries + dead letters
- CLI (`magicmerlin`) with `onboard`, `gateway`, `dashboard`, etc.

## Getting started

```bash
# Run the gateway
cargo run -p magicmerlin-gateway -- --serve 18789 --bind 127.0.0.1 --daemon

# Open local Control UI
cargo run -p magicmerlin -- dashboard --port 18789
```

## Parity / correctness

Compatibility is enforced via snapshots and parity tools.

### Update OpenClaw snapshots
```bash
# Fast snapshots (help/cron/status/version) + manifest hashes
bash scripts/recapture_openclaw_snapshots.sh

# Optional: full CLI help tree (200+ commands)
MAGICMERLIN_CAPTURE_HELP_TREE=1 bash scripts/recapture_openclaw_snapshots.sh
```

### Run parity sentinel
```bash
cargo run -p magicmerlin-sentinel -- methods-diff --openclaw-src ../tmp/openclaw-src
cargo run -p magicmerlin-sentinel -- cli-diff --openclaw-help-tree compat/snapshots/2026-03-02_openclaw_help_tree.json
cargo run -p magicmerlin-sentinel -- docs-diff --index parity/openclaw_docs_index.json --coverage parity/docs_coverage.json
```

## License
MIT — see `LICENSE`.

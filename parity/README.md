# Docs Parity Workflow

This folder tracks docs parity progress against OpenClaw docs.

## Files
- `openclaw_docs_index.json`: Canonical URL index generated from OpenClaw docs inputs.
- `docs_coverage.json`: URL -> status mapping (`done`, `partial`, `todo`).
- `docs_coverage_summary.json`: Section-level rollup with counts per prefix.

## Status semantics
- `done`: MagicMerlin docs surface is implemented and parity-reviewed.
- `partial`: Surface exists but parity is incomplete; still blocks green parity.
- `todo`: Not implemented yet.

## Commands
```bash
# Refresh docs index from a downloaded llms.txt
cargo run -p magicmerlin-sentinel -- docs-index \
  --llms-txt parity/openclaw_llms_2026-03-03.txt \
  --out parity/openclaw_docs_index.json

# Check URL-level coverage gate
cargo run -p magicmerlin-sentinel -- docs-diff \
  --index parity/openclaw_docs_index.json \
  --coverage parity/docs_coverage.json

# Build section rollup (cli/, gateway/, channels/, ...)
cargo run -p magicmerlin-sentinel -- docs-coverage-summary \
  --index parity/openclaw_docs_index.json \
  --coverage parity/docs_coverage.json \
  --out parity/docs_coverage_summary.json
```

## Parity gate
`docs-diff` only returns `ok=true` when both are true:
- `todo == 0`
- `partial == 0`

This means `partial` is intentionally visible debt and still fails the check.

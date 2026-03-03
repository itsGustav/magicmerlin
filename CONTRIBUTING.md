# Contributing to MagicMerlin

MagicMerlin is a Rust-first, OpenClaw-shaped runtime focused on correctness, determinism, and safety.

## Development setup

- Rust stable (latest)

Run:
```bash
cargo fmt
cargo test
```

## Philosophy

- Compat-first: don’t break parity contracts.
- Determinism: avoid non-deterministic outputs in snapshots/CI.
- Safety: no hidden secret exfiltration, no unsafe defaults.

## PR guidelines

- Include tests when behavior changes.
- Keep commits small and readable.
- Document new endpoints/commands in `README.md` or parity docs.

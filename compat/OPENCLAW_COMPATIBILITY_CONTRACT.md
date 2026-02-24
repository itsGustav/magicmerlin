# OpenClaw Compatibility Contract (MagicMerlin)

**Purpose:** MagicMerlin is being built *compat-first*. This document defines what it means for MagicMerlin (Rust) to be "compatible with OpenClaw" and how we prove it.

This contract is intentionally conservative: **if something is ambiguous, we prefer strict compatibility over new behavior.**

## Scope (Parity v0 → v1)

In the v0→v1 window, **compatibility is defined against captured snapshots** of the OpenClaw CLI and runtime tool surface.

MagicMerlin v1 is considered compatible when it can:

1. **Load and reason about** the snapshots in `magicmerlin/compat/snapshots/`.
2. Provide **stable adapters** for:
   - CLI help/command tree discovery (OpenClaw-style UX)
   - Cron job lifecycle concepts (list/create/delete/status)
   - Tool registry exposure (names + short descriptions + argument shapes)
3. Run a minimal gateway-like process (`magicmerlin-gateway`) that can:
   - expose a health/status endpoint
   - surface tool schemas and versions

> Note: v1 does **not** need to implement every OpenClaw command. It must implement the *compatibility substrate* so later work can be incremental and safe.

## Non-goals (explicitly out of scope for v1)

- Reproducing OpenClaw's full feature set (channels, browser relay, nodes, etc.)
- Perfect textual parity of `openclaw --help` output formatting
- Long-running production reliability work (that comes after parity is established)

## Definitions

- **Snapshot:** A raw, timestamped capture of OpenClaw outputs (help text, JSON status, tool list) stored under `magicmerlin/compat/snapshots/`.
- **Golden reference:** The snapshot set is the authoritative reference for expected surface area.
- **Compat adapter:** A MagicMerlin module that maps MagicMerlin’s internal model to the snapshot-defined OpenClaw surface.

## Compatibility Requirements

### 1) Snapshot integrity

- Snapshots MUST be stored as **raw outputs**, not hand-edited.
- Each snapshot file MUST include a short header block indicating:
  - capture date/time + timezone
  - OpenClaw version string (when available)
  - capture command

### 2) CLI surface (discovery parity)

MagicMerlin must model the OpenClaw CLI as:

- a command tree
- per-command help text
- per-command flags/options

For v1, it is sufficient that MagicMerlin can **represent** the tree and help text (even if some commands are stubbed).

### 3) Status JSON (shape stability)

MagicMerlin must:

- treat `openclaw status --json` as a schema-like contract
- allow *tolerant reading* of unknown fields (forward compatibility)
- provide explicit mapping for the fields we rely on (version/build, gateway health, connected channels summary)

### 4) Cron model

MagicMerlin must support the same *conceptual* cron model as OpenClaw:

- uniquely identified jobs
- schedule description
- enabled/disabled state
- last/next run metadata

The exact CLI and persistence can diverge internally, but the external contract must remain mappable.

### 5) Tool registry surface

MagicMerlin must expose a tool registry compatible with the OpenClaw runtime prompt’s tool surface:

- tool name
- brief description
- argument schema (names + types; JSON-schema-like representation is acceptable)

For v1 we accept a "frozen" tool list sourced from snapshots.

## Change Management

- Any time OpenClaw outputs change materially, we:
  1) re-capture snapshots
  2) note the diff in `magicmerlin/PLAN.md` (or a dedicated CHANGELOG later)
  3) update adapters
  4) bump `magicmerlin-compat::COMPAT_VERSION`

## Security / Redaction

Snapshots may contain local machine details (paths, versions, channel identifiers).

- Do **not** publish snapshots externally.
- Before sharing outside the local repo, add a redaction step (or recapture with a redaction mode if OpenClaw adds one).

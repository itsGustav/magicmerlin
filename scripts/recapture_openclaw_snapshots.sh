#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SNAP_DIR="$ROOT_DIR/compat/snapshots"

DATE="$(date +%F)"
CAPTURED_AT="$(date -Iseconds)"

mkdir -p "$SNAP_DIR"

HELP_FILE="${DATE}_openclaw_help.txt"
CRON_HELP_FILE="${DATE}_openclaw_cron_help.txt"
STATUS_FILE="${DATE}_openclaw_status.json"
STATUS_HEADER_FILE="${DATE}_openclaw_status.header.txt"
VERSION_FILE="${DATE}_openclaw_version.txt"

# Tool surface snapshot is currently manual. Keep whatever manifest points to if present.
RUNTIME_TOOLS_FILE="${DATE}_runtime_tools_surface.md"

capture_with_header() {
  local cmd="$1"
  local out="$2"

  {
    echo "# Snapshot: $cmd"
    echo "# command: $cmd"
    echo "# ---"
    # Normalize non-deterministic slogan lines (OpenClaw prints a random quip after an em dash).
    # Keep version+commit, strip the quip so snapshot hashes are meaningful.
    eval "$cmd" | sed -E 's/^(🦞 OpenClaw [^—]+) — .*/\1/'
  } >"$out"
}

# Capture
capture_with_header "openclaw --help" "$SNAP_DIR/$HELP_FILE"
capture_with_header "openclaw cron --help" "$SNAP_DIR/$CRON_HELP_FILE"
openclaw status --json >"$SNAP_DIR/$STATUS_FILE"
capture_with_header "openclaw --version" "$SNAP_DIR/$VERSION_FILE"

# Optional: capture full CLI help tree (expensive; 200+ commands)
# Enable with: MAGICMERLIN_CAPTURE_HELP_TREE=1
HELP_TREE_FILE="${DATE}_openclaw_help_tree.json"
if [[ "${MAGICMERLIN_CAPTURE_HELP_TREE:-}" == "1" ]]; then
  bash "$ROOT_DIR/scripts/capture_help_tree.sh"
fi
HAS_HELP_TREE=false
if [[ -f "$SNAP_DIR/$HELP_TREE_FILE" ]]; then
  HAS_HELP_TREE=true
fi

# Metadata header for status json (keep JSON file raw)
cat >"$SNAP_DIR/$STATUS_HEADER_FILE" <<EOF
# Snapshot: openclaw status --json
# captured_at: $CAPTURED_AT
# command: openclaw status --json
# file: $STATUS_FILE
EOF

# Extract version from the last non-comment line of the version snapshot.
OPENCLAW_VERSION="$(grep -v '^#' "$SNAP_DIR/$VERSION_FILE" | tail -n 1 | tr -d '\n' | sed 's/"/\\"/g')"

# Decide runtime tool surface file:
# - if a same-day file exists, point to it
# - else if manifest exists, reuse its value
# - else fall back to same-day name (best-effort)
TOOLS_SURFACE=""
if [[ -f "$SNAP_DIR/$RUNTIME_TOOLS_FILE" ]]; then
  TOOLS_SURFACE="$RUNTIME_TOOLS_FILE"
elif [[ -f "$SNAP_DIR/manifest.json" ]]; then
  # Best-effort reuse of prior tool surface snapshot filename.
  # Do NOT fail the whole recapture if manifest.json is invalid.
  set +e
  TOOLS_SURFACE="$(python3 - <<PY
import json
try:
  m=json.load(open("$SNAP_DIR/manifest.json"))
  print(m.get("files",{}).get("runtimeToolSurfaceMd", ""))
except Exception:
  print("")
PY
)"
  set -e
fi
if [[ -z "$TOOLS_SURFACE" ]]; then
  TOOLS_SURFACE="$RUNTIME_TOOLS_FILE"
fi

# Ensure TOOLS_SURFACE points to an existing file; otherwise reuse the most recent snapshot.
if [[ ! -f "$SNAP_DIR/$TOOLS_SURFACE" ]]; then
  LATEST_TOOLS_SURFACE="$(ls -1t "$SNAP_DIR"/*_runtime_tools_surface.md 2>/dev/null | head -n 1)"
  if [[ -n "$LATEST_TOOLS_SURFACE" ]]; then
    TOOLS_SURFACE="$(basename "$LATEST_TOOLS_SURFACE")"
  fi
fi

if [[ ! -f "$SNAP_DIR/$TOOLS_SURFACE" ]]; then
  echo "ERROR: runtime tool surface snapshot not found (expected $SNAP_DIR/$TOOLS_SURFACE)" >&2
  echo "Tip: copy current tool surface into $SNAP_DIR/$RUNTIME_TOOLS_FILE or keep an older *_runtime_tools_surface.md" >&2
  exit 1
fi

sha256_file() {
  # macOS: shasum is available by default.
  shasum -a 256 "$1" | awk '{print $1}'
}

# Compute logical snapshot hashes (matches magicmerlin-compat names)
OPENCLAW_HELP_SHA="$(sha256_file "$SNAP_DIR/$HELP_FILE")"
OPENCLAW_CRON_HELP_SHA="$(sha256_file "$SNAP_DIR/$CRON_HELP_FILE")"
OPENCLAW_STATUS_SHA="$(sha256_file "$SNAP_DIR/$STATUS_FILE")"
OPENCLAW_VERSION_SHA="$(sha256_file "$SNAP_DIR/$VERSION_FILE")"
RUNTIME_TOOLS_SHA="$(sha256_file "$SNAP_DIR/$TOOLS_SURFACE")"

OPENCLAW_HELP_TREE_SHA=""
if [[ "$HAS_HELP_TREE" == "true" ]]; then
  OPENCLAW_HELP_TREE_SHA="$(sha256_file "$SNAP_DIR/$HELP_TREE_FILE")"
fi

export CAPTURED_AT OPENCLAW_VERSION HELP_FILE CRON_HELP_FILE STATUS_FILE STATUS_HEADER_FILE VERSION_FILE TOOLS_SURFACE HAS_HELP_TREE HELP_TREE_FILE
export OPENCLAW_HELP_SHA OPENCLAW_CRON_HELP_SHA OPENCLAW_STATUS_SHA OPENCLAW_VERSION_SHA RUNTIME_TOOLS_SHA OPENCLAW_HELP_TREE_SHA

python3 - "$SNAP_DIR/manifest.json" <<'PY'
import json, hashlib, os, sys

out_path = sys.argv[1]

snapshot_hashes = {
  "openclawHelp": os.environ["OPENCLAW_HELP_SHA"],
  "openclawCronHelp": os.environ["OPENCLAW_CRON_HELP_SHA"],
  "openclawStatusJson": os.environ["OPENCLAW_STATUS_SHA"],
  "openclawVersionTxt": os.environ["OPENCLAW_VERSION_SHA"],
  "runtimeToolSurfaceMd": os.environ["RUNTIME_TOOLS_SHA"],
}

files = {
  "openclawHelp": os.environ["HELP_FILE"],
  "openclawCronHelp": os.environ["CRON_HELP_FILE"],
  "openclawStatusJson": os.environ["STATUS_FILE"],
  "openclawStatusHeader": os.environ["STATUS_HEADER_FILE"],
  "openclawVersionTxt": os.environ["VERSION_FILE"],
  "runtimeToolSurfaceMd": os.environ["TOOLS_SURFACE"],
}

if os.environ.get("HAS_HELP_TREE") == "true":
  sha = os.environ.get("OPENCLAW_HELP_TREE_SHA", "")
  if sha:
    snapshot_hashes["openclawHelpTreeJson"] = sha
    files["openclawHelpTreeJson"] = os.environ["HELP_TREE_FILE"]

# Stable fingerprint: sha256 over sorted lines "name sha256\n"
lines = [f"{k} {snapshot_hashes[k]}\n" for k in sorted(snapshot_hashes.keys())]
fingerprint = hashlib.sha256("".join(lines).encode("utf-8")).hexdigest()

manifest = {
  "capturedAt": os.environ["CAPTURED_AT"],
  "openclawVersion": os.environ["OPENCLAW_VERSION"],
  "fingerprint": fingerprint,
  "snapshotHashes": snapshot_hashes,
  "files": files,
}

with open(out_path, "w", encoding="utf-8") as f:
  json.dump(manifest, f, indent=2, ensure_ascii=False)
PY

echo "Updated snapshots in: $SNAP_DIR"
echo "Updated manifest.json"

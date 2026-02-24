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

# Capture
openclaw --help >"$SNAP_DIR/$HELP_FILE"
openclaw cron --help >"$SNAP_DIR/$CRON_HELP_FILE"
openclaw status --json >"$SNAP_DIR/$STATUS_FILE"
openclaw --version >"$SNAP_DIR/$VERSION_FILE"

# Metadata header for status json
cat >"$SNAP_DIR/$STATUS_HEADER_FILE" <<EOF
# Snapshot: openclaw status --json
# captured_at: $CAPTURED_AT
# command: openclaw status --json
# file: $STATUS_FILE
EOF

OPENCLAW_VERSION="$(tr -d '\n' <"$SNAP_DIR/$VERSION_FILE" | sed 's/"/\\"/g')"

# Decide runtime tool surface file:
# - if a same-day file exists, point to it
# - else if manifest exists, reuse its value
# - else fall back to existing dated file if present
TOOLS_SURFACE=""
if [[ -f "$SNAP_DIR/$RUNTIME_TOOLS_FILE" ]]; then
  TOOLS_SURFACE="$RUNTIME_TOOLS_FILE"
elif [[ -f "$SNAP_DIR/manifest.json" ]]; then
  TOOLS_SURFACE="$(node -e "const m=require('${SNAP_DIR}/manifest.json'); console.log(m.files.runtimeToolSurfaceMd)")"
fi
if [[ -z "$TOOLS_SURFACE" ]]; then
  # best effort fallback
  TOOLS_SURFACE="$RUNTIME_TOOLS_FILE"
fi

cat >"$SNAP_DIR/manifest.json" <<EOF
{
  "capturedAt": "${CAPTURED_AT}",
  "openclawVersion": "${OPENCLAW_VERSION}",
  "files": {
    "openclawHelp": "${HELP_FILE}",
    "openclawCronHelp": "${CRON_HELP_FILE}",
    "openclawStatusJson": "${STATUS_FILE}",
    "openclawStatusHeader": "${STATUS_HEADER_FILE}",
    "openclawVersionTxt": "${VERSION_FILE}",
    "runtimeToolSurfaceMd": "${TOOLS_SURFACE}"
  }
}
EOF

echo "Updated snapshots in: $SNAP_DIR"
echo "Updated manifest.json"

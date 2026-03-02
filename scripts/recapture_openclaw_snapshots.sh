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
    echo "# captured_at: $CAPTURED_AT"
    echo "# command: $cmd"
    echo "# ---"
    eval "$cmd"
  } >"$out"
}

# Capture
capture_with_header "openclaw --help" "$SNAP_DIR/$HELP_FILE"
capture_with_header "openclaw cron --help" "$SNAP_DIR/$CRON_HELP_FILE"
openclaw status --json >"$SNAP_DIR/$STATUS_FILE"
capture_with_header "openclaw --version" "$SNAP_DIR/$VERSION_FILE"

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
  TOOLS_SURFACE="$(node -e "const m=require('${SNAP_DIR}/manifest.json'); console.log(m.files.runtimeToolSurfaceMd)")"
fi
if [[ -z "$TOOLS_SURFACE" ]]; then
  TOOLS_SURFACE="$RUNTIME_TOOLS_FILE"
fi

sha256_file() {
  # macOS: shasum is available by default.
  shasum -a 256 "$1" | awk '{print $1}'
}

# Compute logical snapshot hashes (matches magicmerlin-compat names)
OPENCLAW_HELP_SHA="$(sha256_file "$SNAP_DIR/$HELP_FILE")"
OPENCLAW_CRON_HELP_SHA="$(sha256_file "$SNAP_DIR/$CRON_HELP_FILE")"
OPENCLAW_STATUS_SHA="$(sha256_file "$SNAP_DIR/$STATUS_FILE")"
OPENCLAW_STATUS_HEADER_SHA="$(sha256_file "$SNAP_DIR/$STATUS_HEADER_FILE")"
OPENCLAW_VERSION_SHA="$(sha256_file "$SNAP_DIR/$VERSION_FILE")"
RUNTIME_TOOLS_SHA="$(sha256_file "$SNAP_DIR/$TOOLS_SURFACE")"

FINGERPRINT="$(node - <<NODE
const files = {
  openclawHelp: "${OPENCLAW_HELP_SHA}",
  openclawCronHelp: "${OPENCLAW_CRON_HELP_SHA}",
  openclawStatusJson: "${OPENCLAW_STATUS_SHA}",
  openclawStatusHeader: "${OPENCLAW_STATUS_HEADER_SHA}",
  openclawVersionTxt: "${OPENCLAW_VERSION_SHA}",
  runtimeToolSurfaceMd: "${RUNTIME_TOOLS_SHA}",
};
const keys = Object.keys(files).sort();
const crypto = require('crypto');
const h = crypto.createHash('sha256');
for (const k of keys) {
  h.update(k);
  h.update(' ');
  h.update(files[k]);
  h.update("\n");
}
process.stdout.write(h.digest('hex'));
NODE
)"

cat >"$SNAP_DIR/manifest.json" <<EOF
{
  "capturedAt": "${CAPTURED_AT}",
  "openclawVersion": "${OPENCLAW_VERSION}",
  "fingerprint": "${FINGERPRINT}",
  "snapshotHashes": {
    "openclawHelp": "${OPENCLAW_HELP_SHA}",
    "openclawCronHelp": "${OPENCLAW_CRON_HELP_SHA}",
    "openclawStatusJson": "${OPENCLAW_STATUS_SHA}",
    "openclawStatusHeader": "${OPENCLAW_STATUS_HEADER_SHA}",
    "openclawVersionTxt": "${OPENCLAW_VERSION_SHA}",
    "runtimeToolSurfaceMd": "${RUNTIME_TOOLS_SHA}"
  },
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

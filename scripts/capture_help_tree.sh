#!/usr/bin/env bash
# capture_help_tree.sh — Capture openclaw --help for EVERY command+subcommand
# Output: magicmerlin/compat/snapshots/YYYY-MM-DD_openclaw_help_tree.json
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SNAP_DIR="$ROOT_DIR/compat/snapshots"
DATE="$(date +%F)"
OUT="$SNAP_DIR/${DATE}_openclaw_help_tree.json"

mkdir -p "$SNAP_DIR"

escape_json() {
  python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))'
}

# Extract command names from a help block.
# Format: "  cmd *  Description" or "  cmd  Description"
# Commands section starts with "Commands:" and ends at "Examples:" or blank line.
extract_commands() {
  local text="$1"
  echo "$text" | \
    awk '/^Commands:/{found=1; next} /^Examples:|^$/{if(found) exit} found{print}' | \
    grep -E '^\s+[a-z]' | \
    awk '{print $1}' | \
    sed 's/\*$//' | \
    grep -v '^Hint:' | \
    sort -u
}

# Check if command is starred (has subcommands)
is_starred() {
  local text="$1"
  local cmd="$2"
  echo "$text" | grep -qE "^\s+${cmd}\s+\*" 2>/dev/null
}

echo "Capturing OpenClaw help tree..." >&2

TOP_HELP="$(openclaw --help 2>/dev/null || true)"
OC_VERSION="$(openclaw --version 2>/dev/null | tr -d '\n')"
TOP_CMDS="$(extract_commands "$TOP_HELP")"

if [ -z "$TOP_CMDS" ]; then
  echo "ERROR: no commands parsed from openclaw --help" >&2
  exit 1
fi

CMD_COUNT="$(echo "$TOP_CMDS" | wc -l | tr -d ' ')"
echo "Found $CMD_COUNT top-level commands" >&2

# Build JSON with python for safety
python3 - "$OUT" "$OC_VERSION" "$DATE" <<'PYEOF'
import json, subprocess, sys, os

out_path = sys.argv[1]
oc_version = sys.argv[2]
date = sys.argv[3]

def normalize(text: str) -> str:
    # Strip OpenClaw's non-deterministic slogan after the em dash.
    # Example: "🦞 OpenClaw 2026.3.1 (2a8ac97) — <random quip>" -> "🦞 OpenClaw 2026.3.1 (2a8ac97)"
    out_lines = []
    for line in text.splitlines():
        if line.startswith("🦞 OpenClaw ") and " — " in line:
            out_lines.append(line.split(" — ", 1)[0])
        else:
            out_lines.append(line)
    return "\n".join(out_lines) + ("\n" if text.endswith("\n") else "")

def run_help(args):
    """Run openclaw <args> --help and return output."""
    try:
        r = subprocess.run(
            ["openclaw"] + args + ["--help"],
            capture_output=True, text=True, timeout=30
        )
        raw = r.stdout or r.stderr or "(no output)"
        return normalize(raw)
    except Exception as e:
        return f"(error: {e})"

def extract_cmds(help_text):
    """Extract command names from help text.

    IMPORTANT: OpenClaw help descriptions can wrap, and wrapped lines may start
    with lowercase words (e.g. "gateway, workspace..."). We only treat lines
    with exactly two leading spaces as command declarations.
    """
    import re

    cmds = []
    in_commands = False
    for line in help_text.splitlines():
        if line.strip().startswith("Commands:"):
            in_commands = True
            continue
        if in_commands:
            if line.strip() == "" or line.strip().startswith("Examples:"):
                break

            m = re.match(r"^  ([a-z][a-z0-9-]*)(\s+\*\s+|\s{2,})", line)
            if not m:
                continue

            name = m.group(1)
            has_subs = bool(re.match(r"^  [a-z][a-z0-9-]*\s+\*\s+", line))
            cmds.append((name, has_subs))

    return cmds

top_help = run_help([])
top_cmds = extract_cmds(top_help)

print(f"Parsing {len(top_cmds)} top-level commands...", file=sys.stderr)

tree = {
    "version": "1",
    "capturedAt": subprocess.run(["date", "-Iseconds"], capture_output=True, text=True).stdout.strip(),
    "openclawVersion": oc_version,
    "commandCount": 0,
    "commands": {}
}

total = 0

for cmd_name, has_subs in top_cmds:
    print(f"  {cmd_name}{'*' if has_subs else ''}", file=sys.stderr)
    cmd_help = run_help([cmd_name])
    entry = {
        "help": cmd_help,
        "hasSubcommands": has_subs
    }

    if has_subs:
        sub_cmds = extract_cmds(cmd_help)
        subs = {}
        for sub_name, sub_has_subs in sub_cmds:
            sub_help = run_help([cmd_name, sub_name])
            subs[sub_name] = {
                "help": sub_help,
                "hasSubcommands": sub_has_subs
            }
            # Go one more level deep if needed
            if sub_has_subs:
                subsub_cmds = extract_cmds(sub_help)
                subsubs = {}
                for ss_name, _ in subsub_cmds:
                    ss_help = run_help([cmd_name, sub_name, ss_name])
                    subsubs[ss_name] = ss_help
                    total += 1
                subs[sub_name]["subcommands"] = subsubs
            total += 1
        entry["subcommands"] = subs

    tree["commands"][cmd_name] = entry
    total += 1

tree["commandCount"] = total

with open(out_path, "w") as f:
    json.dump(tree, f, indent=2, ensure_ascii=False)

print(f"Done. {total} commands captured.", file=sys.stderr)
PYEOF

# Validate
python3 -c "import json; d=json.load(open('$OUT')); print(f'OK: {d[\"commandCount\"]} commands, {len(d[\"commands\"])} top-level')"
du -h "$OUT"
echo "Wrote: $OUT"

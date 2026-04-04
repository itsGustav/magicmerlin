# Sprint 12 — Session compaction, control UI polish, packaging prep

## Goal
1. Real JSONL-based session compaction with memory extraction
2. Control UI improvements (7-tab SPA, live data)
3. Packaging: version bump to 0.9.0, Dockerfile, cargo-dist setup

## Working directory
`~/Projects/magicmerlin`

## PART A — Real session compaction

Read `agent/src/engine.rs` — find `compact_if_needed` and `estimate_context_percent`.
Read `sessions/src/lib.rs` — find `compact_session`.

### Current state
Session compaction currently truncates messages but doesn't extract memories first.

### What to implement

In `agent/src/session.rs` (or wherever `SessionManager` lives), improve `compact_if_needed`:

```rust
pub fn compact_if_needed(
    &self,
    session: &mut SessionRecord,
    context_window: u64,
    threshold_pct: f32,
) -> Result<Option<CompactionResult>> {
    let pct = self.estimate_context_percent(session, context_window);
    if pct < threshold_pct as f64 {
        return Ok(None);
    }
    
    // 1. Pre-compaction: extract memory candidates from transcript
    //    Look for assistant messages containing "I'll remember", "Important:", 
    //    decisions, facts, names, dates
    let memory_candidates = extract_memory_candidates(&session.transcript);
    
    // 2. Write extracted memories to workspace_dir/memory/YYYY-MM-DD.md
    if !memory_candidates.is_empty() {
        let date = chrono::Local::now().format("%Y-%m-%d");
        let mem_path = self.workspace_dir.join("memory").join(format!("{date}.md"));
        std::fs::create_dir_all(mem_path.parent().unwrap())?;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&mem_path)?;
        use std::io::Write;
        writeln!(f, "\n## Auto-extracted from session compaction ({})\n", chrono::Local::now())?;
        for candidate in &memory_candidates {
            writeln!(f, "- {candidate}")?;
        }
    }
    
    // 3. Compact: keep system prompt + last N messages (default: last 20)
    let msgs = session.transcript.read(0, None)?;
    let keep = 20.min(msgs.len());
    let before = msgs.len();
    let before_tokens = self.estimate_context_percent(session, context_window);
    
    // Keep last `keep` messages
    let kept: Vec<Value> = msgs.into_iter().rev().take(keep).collect::<Vec<_>>()
        .into_iter().rev().collect();
    
    // Rewrite transcript
    session.transcript.reset(kept)?;
    
    let after = keep;
    let after_tokens = self.estimate_context_percent(session, context_window);
    
    Ok(Some(CompactionResult {
        messages_before: before,
        messages_after: after,
        tokens_before: (before_tokens * context_window as f64) as u64,
        tokens_after: (after_tokens * context_window as f64) as u64,
        memory_candidates_extracted: memory_candidates.len(),
    }))
}

fn extract_memory_candidates(transcript: &Transcript) -> Vec<String> {
    let msgs = transcript.read(0, None).unwrap_or_default();
    let mut candidates = Vec::new();
    for msg in &msgs {
        let role = msg.get("role").and_then(Value::as_str).unwrap_or("");
        let content = msg.get("content").and_then(Value::as_str).unwrap_or("");
        if role == "assistant" {
            // Simple heuristic: lines starting with "Remember:", "Note:", facts with numbers/dates
            for line in content.lines() {
                let l = line.trim();
                if l.starts_with("Remember:") || l.starts_with("Note:") || 
                   l.starts_with("Important:") || l.starts_with("Key insight:") ||
                   (l.len() > 20 && l.len() < 200 && l.contains("0x") && l.contains("contract")) {
                    candidates.push(l.to_string());
                }
            }
        }
    }
    candidates
}
```

### Wire to `/compact` slash command

In `gateway/src/main.rs` `run_agent_turn`, when `SlashCommand::Compact`:
1. Load session
2. Call `session_manager.compact_if_needed(session, 120_000, 0.0)` (force compact)
3. Return stats: "Compacted: {before} → {after} messages, {n} memories extracted"

---

## PART B — Control UI improvements

Read the current embedded HTML in `gateway/src/main.rs` (look for `const CONTROL_UI_HTML` or similar).
If it's in a separate file, find it with `find . -name "*.html" -o -name "*ui*"`.

### Improvements needed:

**1. Live agent status on Overview tab**
```javascript
// Poll /call with {"method":"agents.list"} every 5s
// Show each agent: name, model, status, last message time
```

**2. Sessions tab — show real session count and last message**
```javascript
// Poll /call with {"method":"sessions.list"} 
// Show: session_id, last_input (truncated), created_at, message_count
```

**3. Config tab — live read/write**
```javascript
// GET: POST /call {"method":"config.list"} → show all keys
// EDIT: input field + POST /call {"method":"config.set","params":{"key":k,"value":v}}
```

**4. Add Memory tab** (new 8th tab)
```javascript
// Search box → POST /call {"method":"agent.run","params":{"session_id":"ui","message":"/memory <query>"}}
// Show results
```

**5. Better error display**
- Show non-200 gateway responses as red banners
- Auto-reconnect on WebSocket disconnect

---

## PART C — Version + packaging

### 1. Bump version to 0.9.0

In `Cargo.toml` (workspace root):
```toml
[workspace.package]
version = "0.9.0"
```

Update version string in gateway responses:
```rust
// In the CompatInfo/status response
"version": "0.9.0",
"compatTarget": "OpenClaw 2026.4.2",
```

### 2. Dockerfile

Create `Dockerfile` at repo root:
```dockerfile
FROM rust:1.76-slim as build
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p magicmerlin -p magicmerlin-gateway --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libsqlite3-0 && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 merlin
USER merlin
WORKDIR /home/merlin
COPY --from=build /app/target/release/magicmerlin /usr/local/bin/magicmerlin
COPY --from=build /app/target/release/magicmerlin-gateway /usr/local/bin/magicmerlin-gateway
ENV MAGICMERLIN_GATEWAY_PORT=18789
EXPOSE 18789
ENTRYPOINT ["magicmerlin-gateway"]
CMD ["--serve", "18789", "--bind", "0.0.0.0"]
```

### 3. GitHub Actions CI

Update `.github/workflows/` (or create `ci.yml`):
```yaml
name: CI
on: [push, pull_request]
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: cargo test
      - run: cargo clippy -- -D warnings
```

### 4. Release script

Create `scripts/release.sh`:
```bash
#!/bin/bash
set -e
VERSION=${1:-0.9.0}
echo "Releasing MagicMerlin $VERSION"
# Update version
sed -i '' "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
cargo build --release
git tag -a "v$VERSION" -m "Release $VERSION"
git push && git push --tags
echo "Done! Create GitHub release from tag v$VERSION"
```

---

## Build + commit

```bash
cargo build --release 2>&1 | tail -10
git add -A
git commit -m "feat: Sprint 12 — session compaction with memory extraction, control UI improvements, packaging v0.9.0"
```

```bash
openclaw system event --text "Sprint 12 done: session compaction + memory extraction, control UI, Dockerfile, v0.9.0" --mode now
```

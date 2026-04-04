# Sprint 10 — Memory embeddings + CLI wiring + multi-agent config

## Goal
Three parallel improvements that unblock the most important remaining gaps:
1. Upgrade `memory_search` from TF-IDF to real semantic embeddings via `fastembed`
2. Wire the top 20 most important CLI commands so they actually work
3. Add multi-agent config loading (named agents: merlin, henry, paylobster, lobsterprime)

## Working directory
`~/Projects/magicmerlin`

## PART A — Semantic memory search with fastembed

### Step A1 — Add fastembed to Cargo.toml

In `agent-tools/Cargo.toml`, add:
```toml
fastembed = "3"
```

Check if it's already there first.

### Step A2 — Replace TF-IDF in MemorySearchTool

In `agent-tools/src/tools.rs`, find `MemorySearchTool::execute` (currently uses BM25 chunking).

Replace the scoring logic with fastembed embeddings:

```rust
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
    let query = required_string(&params, "query", self.name())?;
    let max_results = params.get("maxResults").or_else(|| params.get("limit"))
        .and_then(Value::as_u64).unwrap_or(10) as usize;
    let min_score = params.get("minScore").or_else(|| params.get("min_score"))
        .and_then(Value::as_f64).unwrap_or(0.01);

    // Collect memory files
    let mut files = collect_memory_files(&ctx.state_paths.state_dir)?;
    let ws_files = collect_memory_files(&ctx.workspace_dir)?;
    for f in ws_files { if !files.contains(&f) { files.push(f); } }

    // Chunk all files
    let mut chunks: Vec<(PathBuf, usize, usize, String)> = Vec::new(); // (path, start, end, text)
    for path in &files {
        let body = std::fs::read_to_string(path).map_err(|e| ToolError::Io { path: path.clone(), source: e })?;
        for chunk in chunk_text(&body, 200) {
            chunks.push((path.clone(), chunk.start_line, chunk.end_line, chunk.text));
        }
    }

    if chunks.is_empty() {
        return Ok(ToolResult::success(json!({"results": []})));
    }

    // Initialize fastembed (model downloads on first use, cached)
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(false)
    ).map_err(|e| ToolError::Execution(format!("fastembed init: {e}")))?;

    // Embed query + all chunks
    let chunk_texts: Vec<&str> = chunks.iter().map(|(_, _, _, t)| t.as_str()).collect();
    let mut all_texts = vec![query.as_str()];
    all_texts.extend_from_slice(&chunk_texts);

    let embeddings = model.embed(all_texts, None)
        .map_err(|e| ToolError::Execution(format!("embed: {e}")))?;

    let query_vec = &embeddings[0];
    let chunk_vecs = &embeddings[1..];

    // Cosine similarity
    let mut scored: Vec<(f64, usize)> = chunk_vecs.iter().enumerate()
        .map(|(i, vec)| (cosine_similarity(query_vec, vec), i))
        .filter(|(score, _)| *score >= min_score)
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let results: Vec<Value> = scored.into_iter().take(max_results).map(|(score, i)| {
        let (path, start, end, text) = &chunks[i];
        let rel_path = path.strip_prefix(&ctx.state_paths.state_dir)
            .or_else(|_| path.strip_prefix(&ctx.workspace_dir))
            .unwrap_or(path);
        json!({
            "path": rel_path,
            "startLine": start,
            "endLine": end,
            "score": (score * 1000.0).round() / 1000.0,
            "snippet": truncate_chars(text, 500),
            "source": "memory",
            "citation": format!("{}#L{}-L{}", rel_path.display(), start, end),
        })
    }).collect();

    Ok(ToolResult::success(json!({"results": results, "provider": "fastembed", "model": "AllMiniLML6V2"})))
}
```

Add a `cosine_similarity` helper:
```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 { return 0.0; }
    (dot / (mag_a * mag_b)) as f64
}
```

**Important**: fastembed downloads models on first run (~90MB for MiniLM). This is expected.
The model is cached in `~/.cache/huggingface/` and subsequent calls are fast.

### Step A3 — Test it compiles
```bash
cargo check -p magicmerlin-agent-tools 2>&1 | head -30
```

---

## PART B — Wire top 20 CLI commands

In `cli/src/main.rs`, find each command arm that calls `unimplemented!()` or `todo!()` or prints 
"not yet implemented" and replace with real implementations.

### Commands to wire (in priority order):

**1. `magicmerlin status`**
```bash
# Should call gateway /call status, format like OpenClaw
# If gateway unreachable: show offline status
```
Implement: HTTP POST to `http://127.0.0.1:{port}/call` with `{"method":"status","params":{}}`, 
pretty-print the JSON response as a status card.

**2. `magicmerlin gateway start`**
Implement: Check if `magicmerlin-gateway` binary exists, spawn it as background process,
write PID to `~/.magicmerlin/gateway.pid`.

**3. `magicmerlin gateway stop`**
Implement: Read PID from `~/.magicmerlin/gateway.pid`, send SIGTERM.

**4. `magicmerlin gateway restart`**
Implement: stop then start.

**5. `magicmerlin gateway status`**
Implement: Check if gateway is running (HTTP health check), print status.

**6. `magicmerlin cron list`**
Implement: Call `cron.list` via HTTP, print jobs table.

**7. `magicmerlin cron add <name> <schedule> <kind> <payload>`**
Implement: Call `cron.add` via HTTP.

**8. `magicmerlin sessions list`**
Implement: Call `sessions.list` via HTTP, print table.

**9. `magicmerlin sessions compact <id>`**
Implement: Call `sessions.compact` via HTTP.

**10. `magicmerlin sessions delete <id>`**
Implement: Call `sessions.delete` via HTTP.

**11. `magicmerlin config get <key>`**
Implement: Call `config.get` via HTTP.

**12. `magicmerlin config set <key> <value>`**
Implement: Call `config.set` via HTTP.

**13. `magicmerlin memory search <query>`**
Implement: Call `memory_search` tool via `agent.run` or directly search local files.

**14. `magicmerlin logs [--tail N]`**
Implement: Call `logs.list` via HTTP or read local log file, print last N lines.

**15. `magicmerlin health`**
Implement: Quick health check — is gateway running? What version? Print summary.

**16. `magicmerlin ping`**
Implement: Call gateway `/call` with `{"method":"health"}`, print RTT.

**17. `magicmerlin agents list`**
Implement: Call `agents.list` via HTTP, print table.

**18. `magicmerlin approvals list`**
Implement: Call `approvals.list` via HTTP, print table.

**19. `magicmerlin version`**
Implement: Print `magicmerlin 0.0.0 (OpenClaw compat v0.5)` or read from build-time env.

**20. `magicmerlin security audit`**
Implement: Call `config.get` + run `run_security_audit()`, print findings.

### Pattern for HTTP calls in CLI:
```rust
fn gateway_http_call(method: &str, params: Value) -> anyhow::Result<Value> {
    let port = std::env::var("MAGICMERLIN_GATEWAY_PORT")
        .unwrap_or_else(|_| "18789".to_string())
        .parse::<u16>().unwrap_or(18789);
    let url = format!("http://127.0.0.1:{port}/call");
    let client = reqwest::blocking::Client::new();
    let body = json!({"method": method, "params": params});
    let resp = client.post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()?
        .json::<Value>()?;
    Ok(resp)
}
```

---

## PART C — Multi-agent config loading

In `gateway/src/main.rs` (or a new `gateway/src/agents.rs`):

1. Read `config.agents` section — look for named agents (merlin, henry, paylobster, lobsterprime)
2. For each named agent, build an `AgentEngineConfig` with its model + workspace + agent_dir
3. Store in `AppState` as `HashMap<String, Arc<AgentEngine>>`
4. In `run_agent_turn`, allow `agent` param to select which engine to use (default: "merlin")
5. Add `"agents.list"` method to `dispatch_ws_method` that returns the list of configured agents

This enables running `agent.run` with `{"session_id": "...", "message": "...", "agent": "henry"}`.

---

## Final steps

**Compile:**
```bash
cargo build --release 2>&1 | tail -40
```

Must be clean. Fix all errors.

**Test memory search:**
```bash
echo "test memory search" | ./target/release/magicmerlin memory search "PayLobster contracts"
```
Or via gateway:
```bash
./target/release/magicmerlin-gateway --serve 19003 &
sleep 2
curl -s -X POST http://127.0.0.1:19003/call \
  -H "Content-Type: application/json" \
  -d '{"method":"agent.run","params":{"session_id":"test","message":"What are the PayLobster contract addresses?","timeout_seconds":30}}'
pkill -f "magicmerlin-gateway.*19003"
```

**Commit:**
```bash
git add -A
git commit -m "feat: semantic memory search via fastembed; wire 20 CLI commands; multi-agent config loading"
```

**Notify:**
```bash
openclaw system event --text "Sprint 10 done: fastembed memory search + 20 CLI commands wired + multi-agent config" --mode now
```

## Notes
- fastembed compile can take ~2-3 minutes on first build (downloads model weights). Normal.
- reqwest blocking client for CLI (sync context), async for gateway (tokio context)
- For CLI reqwest blocking: add `reqwest = { version = "0.12", features = ["blocking", "json"] }` to cli/Cargo.toml
- Keep existing BM25 as fallback if fastembed init fails (catch the error, fall back gracefully)

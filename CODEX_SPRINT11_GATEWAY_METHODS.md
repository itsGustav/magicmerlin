# Sprint 11 — Gateway: wire all remaining stub methods to real data

## Goal
Every method in `dispatch_ws_method` and `http_call` in `gateway/src/main.rs` must return real data.
Audit all methods, find stubs returning hardcoded/empty responses, replace with real implementations.

## Working directory
`~/Projects/magicmerlin`

## Step 1 — Audit all methods

Search for stub patterns:
```bash
grep -n "todo!()\|unimplemented!()\|\"not implemented\"\|json!({\"ok\": true})\|json!({\"stub" gateway/src/main.rs | head -60
```

Also look for methods returning empty arrays or fake data:
```bash
grep -n '"agents": \[\]\|"sessions": \[\]\|"channels": \[\]\|"nodes": \[\]' gateway/src/main.rs | head -20
```

## Step 2 — Methods to implement (priority order)

### High value (used by tools):

**`channels.send`** — must actually dispatch through channel registry
- Currently: probably returns `{"ok": true}` stub
- Fix: `state.channel_registry.send(channel, target, message).await`
- If no channel registry in AppState yet, add `Arc<ChannelRegistry>` to AppState

**`channels.list`** — return configured channels from config
- Read `config.channels` section, return list with status for each

**`channels.status`** — return per-channel health

**`channels.react`** — dispatch reaction to channel registry

**`channels.delete`** — dispatch delete to channel registry

**`memory.search`** — if this gateway method exists, call through to memory_search tool

**`sessions.list`** — must return actual sessions from SQLite db (check current impl)

**`sessions.get`** — return specific session by id

**`config.get`** — read from ConfigManager (check if currently wired)

**`config.set`** — write to ConfigManager

**`config.list`** — return all config keys

**`agents.list`** — return all configured agents with status

**`agents.get`** — return specific agent config

**`nodes.list`** — return connected nodes (empty list if none connected is OK, but not a stub error)

**`browser.status`** — return browser process status

**`logs.list`** — read recent log entries from log file

**`system.info`** — return OS, version, uptime, memory usage

### Medium value:

**`approvals.pending`** — query pending approvals from SQLite

**`approvals.list`** — query all approvals

**`approvals.approve`** / **`approvals.deny`** — update approval status

**`plugins.list`** — return loaded plugins

**`acp.spawn`** — wire to AcpRuntime::spawn

**`acp.cleanup`** — wire to AcpRuntime cleanup

**`security.audit`** — call `run_security_audit()` with current config context

**`gateway.restart`** — send SIGHUP to self (or exec restart)

**`update.run`** — `cargo install --git ... magicmerlin` (self-update)

## Step 3 — Add ChannelRegistry to AppState (if missing)

If `channels.send` / `channels.react` / `channels.delete` can't dispatch because there's no
channel registry in AppState, add one:

```rust
// In AppState add:
channel_registry: Arc<ChannelRegistry>,

// In initialization:
let mut registry = ChannelRegistry::new();
if let Some(tg_config) = &config.channels.telegram {
    let tg = TelegramChannel::new(tg_config.clone());
    registry.register("telegram", Box::new(tg));
}
// etc for other channels
let channel_registry = Arc::new(registry);
```

Check what `ChannelRegistry` looks like in `channels/src/` — it may already exist.

## Step 4 — Real `agents.list` with multi-agent support

```rust
"agents.list" => {
    let cfg = state.config.lock().await;
    let agents: Vec<Value> = cfg.agents.iter().map(|(name, agent_cfg)| {
        json!({
            "name": name,
            "model": agent_cfg.model,
            "status": "active",
            "heartbeat": agent_cfg.heartbeat.as_ref().map(|h| h.enabled),
        })
    }).collect();
    Ok(json!({"agents": agents}))
}
```

## Step 5 — Real `system.info`

```rust
"system.info" | "status" => {
    let uptime_secs = state.started_at.elapsed().as_secs();
    Ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "compatVersion": state.info.compat_version,
        "fingerprint": state.info.fingerprint,
        "uptime": uptime_secs,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "sessions": sessions::count_sessions(&state.db_path).await.unwrap_or(0),
        "scheduler": state.scheduler.state().await.ok(),
        "port": state.port,
    }))
}
```

## Step 6 — Real `logs.list`

Read from the log file (typically `/tmp/magicmerlin/magicmerlin-YYYY-MM-DD.log` or similar):
```rust
"logs.list" => {
    let tail = params.get("tail").and_then(Value::as_u64).unwrap_or(100) as usize;
    let log_path = state.workspace_dir.parent().unwrap_or(&state.workspace_dir)
        .join("logs")
        .join(format!("magicmerlin-{}.log", chrono::Local::now().format("%Y-%m-%d")));
    
    let lines = if log_path.exists() {
        let content = tokio::fs::read_to_string(&log_path).await.unwrap_or_default();
        content.lines().rev().take(tail).collect::<Vec<_>>().into_iter().rev()
            .map(|l| json!(l)).collect::<Vec<_>>()
    } else {
        vec![]
    };
    
    Ok(json!({"lines": lines, "path": log_path}))
}
```

## Step 7 — Build clean

```bash
cargo build --release 2>&1 | tail -20
```

Fix all errors. No new stubs.

## Step 8 — Test key methods

```bash
./target/release/magicmerlin-gateway --serve 19010 &
sleep 2

# Test status
curl -s -X POST http://127.0.0.1:19010/call \
  -H "Content-Type: application/json" \
  -d '{"method":"status","params":{}}' | python3 -m json.tool

# Test agents.list
curl -s -X POST http://127.0.0.1:19010/call \
  -H "Content-Type: application/json" \
  -d '{"method":"agents.list","params":{}}' | python3 -m json.tool

# Test sessions.list
curl -s -X POST http://127.0.0.1:19010/call \
  -H "Content-Type: application/json" \
  -d '{"method":"sessions.list","params":{}}' | python3 -m json.tool

pkill -f "magicmerlin-gateway.*19010"
```

All three must return real structured data (not stubs, not errors).

## Step 9 — Commit

```bash
git add -A
git commit -m "feat(gateway): wire all stub methods to real data — channels, agents, sessions, logs, system, approvals"
```

## When done

```bash
openclaw system event --text "Sprint 11 done: all gateway methods return real data" --mode now
```

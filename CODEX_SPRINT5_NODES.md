# Sprint 5 — Agent B: Nodes + Orchestration Tools

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Several tools in `agent-tools/src/tools.rs` are stubs — this sprint wires them all.

---

## Tool 1: `nodes` — Remote device control

The `NodesTool` stub needs a real HTTP client talking to node-host.

Node hosts are registered in config: `config.nodes[]` — each has `{ id, url, token }`.

```rust
// NodesTool::execute:
// GET/POST to {node.url}/api/{action} with Bearer {node.token}

pub struct NodeApiClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl NodeApiClient {
    // actions map to HTTP endpoints:
    // status → GET /api/status
    // describe → GET /api/describe
    // pending → GET /api/pairing/pending
    // approve → POST /api/pairing/approve { requestId }
    // reject → POST /api/pairing/reject { requestId }
    // notify → POST /api/notify { title, body, priority, sound, delivery }
    // camera_snap → POST /api/camera/snap { facing, maxWidth, quality }  → returns base64 image
    // camera_list → GET /api/camera/list
    // camera_clip → POST /api/camera/clip { facing, durationMs, fps, includeAudio }
    // photos_latest → GET /api/photos/latest?limit=N
    // screen_record → POST /api/screen/record { durationMs, screenIndex }
    // location_get → GET /api/location?accuracy=balanced&timeoutMs=5000
    // notifications_list → GET /api/notifications?limit=N
    // notifications_action → POST /api/notifications/action { notificationKey, action, replyText }
    // device_status → GET /api/device/status
    // device_info → GET /api/device/info
    // run → POST /api/run { command: [String], cwd, env, timeoutMs }
    // invoke → POST /api/invoke { command, params, timeoutMs }
    
    async fn get(&self, path: &str) -> Result<Value>;
    async fn post(&self, path: &str, body: Value) -> Result<Value>;
}

// In NodesTool::execute:
// 1. Get node from config by `node` param or first available
// 2. Build NodeApiClient
// 3. Dispatch to correct endpoint
// 4. Return response JSON
```

Add `node_configs: Vec<NodeConfig>` to `ToolContext`.

---

## Tool 2: `sessions_spawn` — ACP sub-agent spawn

The `SessionsSpawnTool` stub needs to wire to the ACP runtime.

```rust
// SessionsSpawnTool::execute:
// Params: { task: String, agentId?: String, mode?: "run"|"session", 
//           model?: String, runtime?: "subagent"|"acp",
//           timeoutSeconds?: u64, label?: String, thread?: bool }

// Strategy: POST to gateway sessions.spawn method
// Gateway handles: ACP harness spawn, sub-agent isolation, session tracking

// Return: { sessionKey: String, sessionId: String }

async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
    let resp = gateway_call(&ctx.gateway_url, "sessions.spawn", params).await?;
    Ok(ToolResult::json(resp))
}
```

---

## Tool 3: `sessions_list` — List sessions

```rust
// POST gateway sessions.list
// Params: { activeMinutes?: u32, kinds?: [String], limit?: u32, messageLimit?: u32 }
// Return: array of session summaries
```

---

## Tool 4: `sessions_history` — Session transcript

```rust
// POST gateway sessions.history
// Params: { sessionKey: String, limit?: u32, includeTools?: bool }
// Return: { messages: [...], total: N }
```

---

## Tool 5: `sessions_send` — Send to another session

```rust
// POST gateway sessions.send
// Params: { sessionKey: String, message: String, timeoutSeconds?: u64 }
// Return: { ok: bool, reply?: String }
```

---

## Tool 6: `sessions_yield` — End current turn

```rust
// SessionsYieldTool: POST gateway sessions.yield
// Params: { message?: String }
// Signals the gateway to pause this session until a sub-agent reports back
```

---

## Tool 7: `subagents` — Manage sub-agents

```rust
// SubagentsTool:
// action=list → POST gateway subagents.list
// action=steer → POST gateway subagents.steer { target, message }
// action=kill → POST gateway subagents.kill { target }
```

---

## Tool 8: `agents_list` — Available agent IDs

```rust
// AgentsListTool:
// POST gateway agents.list → return array of agent IDs
// (used by agent to know what harnesses are available for sessions_spawn)
```

---

## Tool 9: `gateway` tool (cron-level gateway control)

Some agents call a `gateway` tool for config changes and restarts:

```rust
// GatewayTool actions:
// restart → POST gateway gateway.restart { reason, delayMs }
// config.get → POST gateway gateway.config.get { path? }
// config.patch → POST gateway gateway.config.patch { raw, note }
// config.apply → POST gateway gateway.config.apply { raw, note }
// update.run → POST gateway gateway.update.run
```

---

## Tool 10: `cron` tool completeness audit

The existing `CronTool` dispatches to gateway. Verify all actions work:
- `status`, `list`, `add`, `update`, `remove`, `run`, `runs`, `wake`

Also implement the `job` parameter building correctly for `add`:
```rust
// CronTool add params:
// { action: "add", job: { name?, schedule: {kind,expr/everyMs/at}, 
//   payload: {kind,text/message}, delivery?, sessionTarget, enabled? } }
// Pass job object directly to gateway cron.add
```

---

## Gateway Call Helper

Create a shared helper in `agent-tools/src/gateway.rs`:

```rust
pub async fn gateway_call(gateway_url: &str, method: &str, params: Value) -> Result<Value> {
    let url = format!("{}/call", gateway_url);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&json!({ "method": method, "params": params }))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| ToolError::Io(format!("Gateway unreachable: {e}")))?;
    
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ToolError::Gateway(format!("HTTP {status}: {body}")).into());
    }
    
    Ok(resp.json::<Value>().await?)
}
```

Use this helper in all gateway-dispatch tools (replacing duplicated inline reqwest calls).

---

## Rules
- `cargo build --workspace` clean
- Unit tests: NodeApiClient URL building, sessions_spawn param forwarding
- All gateway-dispatch tools use the shared `gateway_call` helper

## Completion
```bash
openclaw system event --text "Sprint 5B done: nodes tool HTTP client, sessions_spawn/list/history/send/yield wired to gateway, subagents/agents_list/gateway tool, shared gateway_call helper" --mode now
```

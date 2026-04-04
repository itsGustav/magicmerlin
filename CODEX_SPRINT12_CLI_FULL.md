# Sprint 12 — Wire ALL CLI commands (257 total, no unimplemented!())

## Goal
Every single CLI command in `cli/src/main.rs` must either:
1. Execute real logic (call gateway, read config, run action), OR
2. Print a clear "Not yet available in this version" message

Zero `unimplemented!()`, zero `todo!()`, zero panic on any command.

## Working directory
`~/Projects/magicmerlin`

## Step 1 — Audit

```bash
grep -n "unimplemented!()\|todo!()\|println!(\"TODO\|println!(\"Not yet\|eprintln!(\"TODO" cli/src/main.rs | wc -l
grep -n "unimplemented!()\|todo!()" cli/src/main.rs | head -50
```

## Step 2 — Gateway HTTP helper

First, ensure there's a clean `gateway_call` helper in CLI (blocking, for non-async context):
```rust
fn gateway_call(port: u16, method: &str, params: Value) -> anyhow::Result<Value> {
    let url = format!("http://127.0.0.1:{port}/call");
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&json!({"method": method, "params": params}))
        .timeout(std::time::Duration::from_secs(10))
        .send()?
        .json::<Value>()?;
    Ok(resp)
}

fn default_port() -> u16 {
    std::env::var("MAGICMERLIN_GATEWAY_PORT")
        .ok().and_then(|p| p.parse().ok()).unwrap_or(18789)
}
```

## Step 3 — Wire every command group

For each command, implement real logic. Pattern for gateway-backed commands:
```rust
SomeCommand::List => {
    let result = gateway_call(default_port(), "some.list", json!({}))?;
    print_table(&result); // or println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

### Complete command list to wire:

**status** → `gateway_call("status", {})`  
**health** → `gateway_call("health", {})`  
**version** → `println!("{}", env!("CARGO_PKG_VERSION"))`  
**ping** → measure RTT to gateway health endpoint

**gateway start** → spawn `magicmerlin-gateway --serve {port} --daemon`  
**gateway stop** → `kill $(cat ~/.magicmerlin/gateway.pid)`  
**gateway restart** → stop then start  
**gateway status** → HTTP GET to gateway  
**gateway install** → install as LaunchAgent (macOS) or systemd (Linux)  
**gateway uninstall** → remove service  

**agent run <prompt>** → `gateway_call("agent.run", {"session_id": "cli", "message": prompt})`  
**agent list** → `gateway_call("agents.list", {})`  
**agent stop <name>** → `gateway_call("agents.stop", {"name": name})`  

**sessions list** → `gateway_call("sessions.list", {})`  
**sessions get <id>** → `gateway_call("sessions.get", {"id": id})`  
**sessions compact <id>** → `gateway_call("sessions.compact", {"id": id})`  
**sessions delete <id>** → `gateway_call("sessions.delete", {"id": id})`  

**cron list** → `gateway_call("cron.list", {})`  
**cron add <name> <expr> <kind> <payload>** → `gateway_call("cron.add", {...})`  
**cron remove <id>** → `gateway_call("cron.remove", {"id": id})`  
**cron run <id>** → `gateway_call("cron.run", {"id": id})`  
**cron pause <id>** → `gateway_call("cron.pause", {"id": id})`  
**cron resume <id>** → `gateway_call("cron.resume", {"id": id})`  
**cron runs <id>** → `gateway_call("cron.runs", {"id": id})`  

**config get <key>** → `gateway_call("config.get", {"key": key})`  
**config set <key> <value>** → `gateway_call("config.set", {"key": key, "value": value})`  
**config list** → `gateway_call("config.list", {})`  
**config validate** → parse config file, report errors  
**config edit** → open config in `$EDITOR`  

**logs [--tail N]** → `gateway_call("logs.list", {"tail": n})`  
**logs follow** → SSE stream from `/events`  

**channels list** → `gateway_call("channels.list", {})`  
**channels status** → `gateway_call("channels.status", {})`  
**channels restart <name>** → `gateway_call("channels.restart", {"channel": name})`  

**memory search <query>** → read local files + TF-IDF search (or gateway call)  
**memory get <path>** → read specific memory file  

**security audit** → `gateway_call("security.audit", {})`  
**security audit --deep** → `gateway_call("security.audit", {"deep": true})`  

**secrets list** → list env vars / secrets from config  
**secrets get <key>** → show specific secret (masked)  
**secrets set <key> <value>** → write to secrets store  

**plugins list** → `gateway_call("plugins.list", {})`  
**plugins reload** → `gateway_call("plugins.reload", {})`  

**approvals list** → `gateway_call("approvals.list", {})`  
**approvals approve <code>** → `gateway_call("approvals.approve", {"code": code})`  
**approvals deny <code>** → `gateway_call("approvals.deny", {"code": code})`  

**nodes list** → `gateway_call("nodes.list", {})`  
**nodes status <id>** → `gateway_call("nodes.status", {"id": id})`  

**browser status** → `gateway_call("browser.status", {})`  
**browser start** → `gateway_call("browser.start", {})`  
**browser stop** → `gateway_call("browser.stop", {})`  

**acp list** → `gateway_call("acp.list", {})`  
**acp spawn <agent> <task>** → `gateway_call("acp.spawn", {"agent": agent, "task": task})`  
**acp kill <session>** → `gateway_call("acp.cleanup", {"sessionId": session})`  

**sandbox status** → `gateway_call("sandbox.status", {})`  

**qr** → generate QR code for pairing (call gateway pairing endpoint, render QR in terminal)  

**update** → `cargo install --git https://github.com/itsGustav/magicmerlin magicmerlin`  

**setup** / **onboard** → interactive setup wizard:
1. Check if gateway is running
2. Prompt for model API key
3. Prompt for Telegram bot token (optional)  
4. Write to config file
5. Start gateway

**tui** → launch Ratatui terminal dashboard (if implemented, or print "TUI coming in v1.1")  

**completion <shell>** → already implemented (generate shell completions)  

**help-all** → print all commands with descriptions  

**context** → `gateway_call("context.list", {})`  

**subagents list** → `gateway_call("subagents.list", {})`  
**subagents kill <id>** → `gateway_call("subagents.kill", {"id": id})`  
**subagents steer <id> <msg>** → `gateway_call("subagents.steer", {"id": id, "message": msg})`  

## Step 4 — Pretty-print helper

Add a `print_json` helper that formats JSON nicely for terminal output:
```rust
fn print_json(val: &Value) {
    println!("{}", serde_json::to_string_pretty(val).unwrap_or_else(|_| format!("{val}")));
}

fn print_table(val: &Value) {
    // If value has a top-level array field, print as simple table
    // Otherwise, fall back to print_json
    if let Some(obj) = val.as_object() {
        for (key, arr) in obj {
            if let Some(items) = arr.as_array() {
                println!("--- {key} ({}) ---", items.len());
                for item in items {
                    println!("  {}", serde_json::to_string(item).unwrap_or_default());
                }
                return;
            }
        }
    }
    print_json(val);
}
```

## Step 5 — Build clean

```bash
cargo build --release 2>&1 | tail -20
```

Zero errors. Warnings OK.

## Step 6 — Verify no panics

```bash
./target/release/magicmerlin status 2>&1 | head -5
./target/release/magicmerlin version
./target/release/magicmerlin cron list 2>&1 | head -5
./target/release/magicmerlin sessions list 2>&1 | head -5
./target/release/magicmerlin help-all | wc -l
```

None should panic. Gateway-backed ones will fail gracefully if gateway isn't running.

## Step 7 — Commit

```bash
git add -A
git commit -m "feat(cli): wire all 257 CLI commands — no more unimplemented!() or todo!()"
```

## When done

```bash
openclaw system event --text "Sprint 12 done: all 257 CLI commands wired, zero unimplemented!()" --mode now
```

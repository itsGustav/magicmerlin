# Sprint 4 — Agent A: CLI Wiring + Gateway Service

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
CLI is at `cli/src/main.rs` — 1,486 lines, 193 commands scaffolded but most call `unimplemented!()`.
Goal: wire every command to a real implementation via gateway HTTP calls.

## Architecture
All CLI commands that need runtime data should call the gateway:
```rust
// Helper: POST {gateway_url}/call
async fn gateway_call(method: &str, params: Value) -> Result<Value> {
    let url = std::env::var("MAGICMERLIN_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:18789".into());
    let resp = reqwest::Client::new()
        .post(format!("{url}/call"))
        .json(&json!({"method": method, "params": params}))
        .timeout(Duration::from_secs(10))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}
```

## Commands to Wire

### 1. `magicmerlin status`
```rust
// Call gateway.status → format health card
// Show: channels (name, status, last_message), model, session count, uptime
// Fallback if gateway down: "Gateway offline — run 'magicmerlin gateway start'"
```

### 2. `magicmerlin gateway start/stop/restart/status`
```rust
// start: launch magicmerlin-gateway as background daemon
//   - find binary: which magicmerlin-gateway or same dir as magicmerlin
//   - write PID file to ~/.magicmerlin/gateway.pid
//   - on macOS: optionally create/load LaunchAgent plist
// stop: read PID file, send SIGTERM, remove PID file
// restart: stop + start
// status: check if PID is alive, show uptime + port
```

### 3. `magicmerlin sessions list`
```rust
// Call gateway: sessions.list → tabulate: key | model | messages | last_active
```

### 4. `magicmerlin sessions show <key>`
```rust
// Call sessions.show → print recent messages from transcript
```

### 5. `magicmerlin sessions delete <key>`
```rust
// Call sessions.delete → confirm prompt → delete
```

### 6. `magicmerlin sessions compact <key>`  
```rust
// Call sessions.compact → show before/after token counts
```

### 7. `magicmerlin config get [path]`
```rust
// Call gateway: config.get → pretty-print JSON
// If path given: extract that path from config (e.g. "model", "channels.telegram")
```

### 8. `magicmerlin config set <path> <value>`
```rust
// Call gateway: config.patch → confirm → show diff
```

### 9. `magicmerlin config validate`
```rust
// Load config from disk, run validation, report errors
```

### 10. `magicmerlin cron list`
```rust
// Call cron.list → tabulate: id | name | schedule | last_run | next_run | status
```

### 11. `magicmerlin cron add --name <n> --schedule <expr> --message <m>`
```rust
// Build job JSON → call cron.add → show job ID
```

### 12. `magicmerlin cron remove <id>`
```rust
// Call cron.remove → confirm → done
```

### 13. `magicmerlin cron run <id>`
```rust
// Call cron.run → show output
```

### 14. `magicmerlin cron runs <id>`
```rust
// Call cron.runs → tabulate run history
```

### 15. `magicmerlin memory search <query>`
```rust
// Call memory.search → display results with citations
```

### 16. `magicmerlin memory get <path> [--from N] [--lines N]`
```rust
// Call memory.get → print content
```

### 17. `magicmerlin channels status`
```rust
// Call channels.list → show each channel's connection status
```

### 18. `magicmerlin message send --to <target> --message <text> [--channel telegram]`
```rust
// Call channels.send → confirm sent + message ID
```

### 19. `magicmerlin agents list`
```rust
// Call agents.list → tabulate: id | model | channels | last_active
```

### 20. `magicmerlin models list`
```rust
// Call providers.list → show available models per provider
```

### 21. `magicmerlin logs [--tail N] [--follow]`
```rust
// Read log file from config.log_path
// --follow: tail -f equivalent using tokio::fs::File + sleep
// Default: last 100 lines
```

### 22. `magicmerlin security audit`
```rust
// Call gateway: security.audit or run locally
// Check: config permissions, exposed ports, API key exposure, etc.
// Print report with PASS/WARN/FAIL per check
```

### 23. `magicmerlin approvals list`
```rust
// Call approvals.list → show pending approval requests
```

### 24. `magicmerlin plugins list`
```rust
// Call plugins.list → tabulate loaded plugins
```

### 25. `magicmerlin skills list`
```rust
// Scan skills directories locally → tabulate name | description | location
```

### 26. `magicmerlin version`
```rust
// Print: magicmerlin 0.1.0 (OpenClaw compat v0.5)
// Also: cargo version env var CARGO_PKG_VERSION
```

### 27. `magicmerlin ping`
```rust
// GET {gateway_url}/health → print latency + status
```

## Output Formatting

Use a consistent table formatter:
```rust
fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    // Calculate column widths
    // Print with │ separators (or plain spaces for --no-color)
}
```

Color output: use `\x1b[32m` green for OK, `\x1b[31m` red for error, `\x1b[33m` yellow for warn.
Respect `NO_COLOR` env var and `--no-color` flag.

## Gateway Start Implementation (Critical)

```rust
fn gateway_start(port: Option<u16>, bind: Option<String>, daemon: bool) -> Result<()> {
    let port = port.unwrap_or(18789);
    let bind = bind.unwrap_or_else(|| "127.0.0.1".into());
    
    // Find binary
    let binary = find_gateway_binary()?;
    
    // Build args
    let mut cmd = std::process::Command::new(&binary);
    cmd.args(["--serve", &port.to_string(), "--bind", &bind, "--daemon"]);
    
    if daemon {
        // Spawn detached
        cmd.stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::null())
           .spawn()?;
        // Save PID
        let pid_path = home_dir()?.join(".magicmerlin/gateway.pid");
        std::fs::write(pid_path, cmd_output.id().to_string())?;
        println!("Gateway started on :{port}");
    } else {
        // Foreground
        cmd.status()?;
    }
    Ok(())
}
```

## Rules
- Wire as many commands as possible (target: all 27 above at minimum)
- Every command should have a real implementation, not `unimplemented!()`
- `cargo build --workspace` must pass clean
- Consistent error messages: if gateway unreachable, suggest `magicmerlin gateway start`

## Completion
```bash
openclaw system event --text "Sprint 4A done: CLI fully wired — status, gateway start/stop, sessions, config, cron, memory, channels, message, agents, models, logs, security audit, plugins, skills, version, ping" --mode now
```

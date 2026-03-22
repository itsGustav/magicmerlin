# Sprint 7 — Control UI + CI + Docs Auto-Generator

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
This sprint ships: (1) a functional Control UI served from the gateway, (2) GitHub Actions CI, (3) auto-generated docs.

---

## Part 1: Control UI

### Strategy: Minimal HTML + JS served from gateway

Create `gateway/static/index.html` — a single-file SPA using vanilla JS + CSS. No build step required.

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Magic Merlin</title>
  <style>
    /* Dark theme, monospace, minimal */
    :root { --bg: #0d1117; --fg: #c9d1d9; --accent: #58a6ff; --border: #30363d; }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { background: var(--bg); color: var(--fg); font-family: 'SF Mono', monospace; font-size: 13px; }
    nav { display: flex; gap: 1px; background: var(--border); border-bottom: 1px solid var(--border); }
    nav button { background: var(--bg); color: var(--fg); border: none; padding: 10px 16px; cursor: pointer; }
    nav button.active { color: var(--accent); border-bottom: 2px solid var(--accent); }
    .panel { display: none; padding: 16px; }
    .panel.active { display: block; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 8px 12px; text-align: left; border-bottom: 1px solid var(--border); }
    th { color: var(--accent); font-weight: normal; }
    .badge { padding: 2px 6px; border-radius: 4px; font-size: 11px; }
    .ok { background: #1a3a1a; color: #56d364; }
    .err { background: #3a1a1a; color: #f85149; }
    .warn { background: #3a2a1a; color: #e3b341; }
    pre { background: #161b22; padding: 12px; border-radius: 6px; overflow: auto; max-height: 400px; }
    input, select { background: #161b22; border: 1px solid var(--border); color: var(--fg); padding: 6px 10px; border-radius: 4px; }
    button.action { background: var(--accent); color: #0d1117; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; }
  </style>
</head>
<body>
  <nav>
    <button onclick="tab('overview')" class="active" id="tab-overview">Overview</button>
    <button onclick="tab('sessions')" id="tab-sessions">Sessions</button>
    <button onclick="tab('cron')" id="tab-cron">Cron</button>
    <button onclick="tab('channels')" id="tab-channels">Channels</button>
    <button onclick="tab('config')" id="tab-config">Config</button>
    <button onclick="tab('logs')" id="tab-logs">Logs</button>
  </nav>

  <div id="overview" class="panel active">
    <h2 style="margin-bottom:12px;color:var(--accent)">🧙 Magic Merlin</h2>
    <div id="status-card">Loading...</div>
    <table style="margin-top:16px" id="agents-table"></table>
  </div>

  <div id="sessions" class="panel">
    <button class="action" onclick="loadSessions()">Refresh</button>
    <table style="margin-top:12px" id="sessions-table"></table>
  </div>

  <div id="cron" class="panel">
    <button class="action" onclick="loadCron()">Refresh</button>
    <table style="margin-top:12px" id="cron-table"></table>
  </div>

  <div id="channels" class="panel">
    <table id="channels-table"></table>
  </div>

  <div id="config" class="panel">
    <pre id="config-pre">Loading...</pre>
  </div>

  <div id="logs" class="panel">
    <div style="margin-bottom:8px">
      <button class="action" onclick="loadLogs()">Refresh</button>
      <button class="action" style="margin-left:8px" onclick="toggleFollow()">Follow</button>
    </div>
    <pre id="logs-pre" style="max-height:600px"></pre>
  </div>

  <script>
    const GW = window.location.origin; // same origin as gateway
    let followInterval = null;

    async function call(method, params = {}) {
      const r = await fetch(`${GW}/call`, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({method, params})
      });
      return r.json();
    }

    function tab(name) {
      document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
      document.querySelectorAll('nav button').forEach(b => b.classList.remove('active'));
      document.getElementById(name).classList.add('active');
      document.getElementById(`tab-${name}`).classList.add('active');
      if (name === 'sessions') loadSessions();
      if (name === 'cron') loadCron();
      if (name === 'channels') loadChannels();
      if (name === 'config') loadConfig();
      if (name === 'logs') loadLogs();
    }

    async function loadOverview() {
      const status = await call('gateway.status');
      document.getElementById('status-card').innerHTML = `
        <span class="badge ok">● online</span>
        <span style="margin-left:8px">port ${status.port || '?'}</span>
        <span style="margin-left:8px">uptime ${status.uptime || '?'}</span>
        <span style="margin-left:8px">sessions: ${status.sessions || 0}</span>
      `;
      const agents = await call('agent.status');
      const rows = (agents.agents || []).map(a =>
        `<tr><td>●</td><td>${a.id}</td><td>${a.model}</td><td>${a.last_active || '-'}</td></tr>`
      ).join('');
      document.getElementById('agents-table').innerHTML =
        `<tr><th></th><th>Agent</th><th>Model</th><th>Last Active</th></tr>${rows}`;
    }

    async function loadSessions() {
      const data = await call('sessions.list');
      const rows = (data.sessions || data || []).map(s =>
        `<tr><td>${s.key}</td><td>${s.messageCount || 0}</td><td>${s.contextPercent ? Math.round(s.contextPercent*100)+'%' : '-'}</td><td>${s.lastActive || '-'}</td><td><button onclick="deleteSession('${s.key}')">Delete</button></td></tr>`
      ).join('');
      document.getElementById('sessions-table').innerHTML =
        `<tr><th>Key</th><th>Messages</th><th>Context</th><th>Last Active</th><th></th></tr>${rows}`;
    }

    async function deleteSession(key) {
      if (!confirm(`Delete session ${key}?`)) return;
      await call('sessions.delete', {sessionKey: key});
      loadSessions();
    }

    async function loadCron() {
      const data = await call('cron.list');
      const rows = (data.jobs || data || []).map(j =>
        `<tr><td>${j.id?.substring(0,8)}</td><td>${j.name||'-'}</td><td><code>${j.schedule?.expr||j.schedule?.kind||'-'}</code></td><td>${j.lastRun||'-'}</td><td>${j.nextRun||'-'}</td><td><span class="badge ${j.enabled?'ok':'warn'}">${j.enabled?'on':'off'}</span></td><td><button onclick="runCron('${j.id}')">▶</button></td></tr>`
      ).join('');
      document.getElementById('cron-table').innerHTML =
        `<tr><th>ID</th><th>Name</th><th>Schedule</th><th>Last Run</th><th>Next Run</th><th>Status</th><th></th></tr>${rows}`;
    }

    async function runCron(id) {
      await call('cron.run', {jobId: id});
      setTimeout(loadCron, 1000);
    }

    async function loadChannels() {
      const data = await call('channels.list');
      const rows = (data.channels || data || []).map(c =>
        `<tr><td>${c.name}</td><td>${c.platform}</td><td><span class="badge ${c.status==='connected'?'ok':'err'}">${c.status}</span></td><td>${c.lastMessage||'-'}</td></tr>`
      ).join('');
      document.getElementById('channels-table').innerHTML =
        `<tr><th>Name</th><th>Platform</th><th>Status</th><th>Last Message</th></tr>${rows}`;
    }

    async function loadConfig() {
      const data = await call('config.get');
      document.getElementById('config-pre').textContent = JSON.stringify(data, null, 2);
    }

    async function loadLogs() {
      const data = await call('gateway.logs', {tail: 200});
      document.getElementById('logs-pre').textContent = (data.lines || data || []).join('\n');
      document.getElementById('logs-pre').scrollTop = 99999;
    }

    function toggleFollow() {
      if (followInterval) { clearInterval(followInterval); followInterval = null; }
      else { followInterval = setInterval(loadLogs, 2000); }
    }

    loadOverview();
    setInterval(loadOverview, 5000);
  </script>
</body>
</html>
```

### Serve from gateway

In `gateway/src/main.rs`, add route:
```rust
.route("/", get(serve_ui))
.route("/ui", get(serve_ui))

async fn serve_ui() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    axum::response::Html(html)
}
```

Create `gateway/static/` directory and save the HTML there.
Use `include_str!` for zero-runtime-dependency serving.

---

## Part 2: GitHub Actions CI

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --workspace --all-features
      - name: Test
        run: cargo test --workspace
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

  cross-compile:
    name: Cross-compile (${{ matrix.target }})
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - x86_64-unknown-linux-musl
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Install cross
        run: cargo install cross
      - name: Cross build
        run: cross build --workspace --target ${{ matrix.target }} --release

  parity:
    name: Parity Sentinel
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run parity checks
        run: cargo test --package magicmerlin-sentinel
```

---

## Part 3: Docs Auto-Generator

Create `tools/src/docgen.rs` — a binary that generates Markdown docs:

```rust
// Reads: parity/openclaw_docs_index.json
// For each page, generates a Markdown file in docs/
// Pages that have known equivalents get real content
// Unknown pages get a template with TODOs

fn main() {
    let index: Vec<DocPage> = serde_json::from_str(
        include_str!("../../parity/openclaw_docs_index.json")
    ).unwrap();
    
    for page in &index {
        let path = format!("docs/{}.md", page.slug);
        if !std::path::Path::new(&path).exists() {
            let content = generate_page(page);
            std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).ok();
            std::fs::write(&path, content).ok();
            println!("Generated: {path}");
        }
    }
}

fn generate_page(page: &DocPage) -> String {
    format!(
        "# {}\n\n{}\n\n<!-- TODO: expand this page -->\n\n## See Also\n- [OpenClaw docs](https://docs.openclaw.ai/{})\n",
        page.title,
        page.description.as_deref().unwrap_or(""),
        page.slug,
    )
}
```

Add to `tools/Cargo.toml`:
```toml
[[bin]]
name = "docgen"
path = "src/docgen.rs"
```

Also create `docs/README.md` as the docs landing page.

Run it: `cargo run -p magicmerlin-tools --bin docgen` → generates all 332 docs pages.

---

## Rules
- CI must pass on ubuntu-latest and macos-latest
- Control UI must serve from `/` without any external deps at runtime
- `cargo build --workspace` must pass clean
- Docs generator must create at least the directory structure

## Completion
```bash
openclaw system event --text "Sprint 7 done: Control UI (single-file SPA, dark theme, overview+sessions+cron+channels+config+logs), GitHub Actions CI (test+clippy+cross-compile+parity), docs auto-generator (332 pages)" --mode now
```

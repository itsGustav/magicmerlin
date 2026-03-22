# Sprint 7 — Control UI (Static Serve + Minimal SPA)

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Gateway runs at `gateway/src/main.rs` on axum. 
Control UI is completely absent — must ship a working web dashboard.

## Strategy
Build a self-contained HTML/JS/CSS dashboard served statically by the gateway.
No external build step — everything is embedded as Rust `include_str!` or inline constants.
Target: functional single-page app covering the 5 core views.

---

## Step 1: Create `gateway/src/ui/` module

```
gateway/src/ui/
  mod.rs         — axum router for UI routes
  index.html     — the SPA shell
  dashboard.js   — vanilla JS, no bundler needed
  styles.css     — minimal dark theme CSS
```

### `mod.rs` — serve the UI
```rust
use axum::{response::Html, routing::get, Router};

pub fn ui_router() -> Router {
    Router::new()
        .route("/ui", get(serve_index))
        .route("/ui/", get(serve_index))
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}
```

Mount in gateway main:
```rust
let app = app.merge(ui::ui_router());
```

---

## Step 2: Build `index.html` — The SPA

A single HTML file with all JS and CSS inline (no CDN deps, works offline).

### Layout
```html
<!DOCTYPE html>
<html data-theme="dark">
<head>
  <title>Magic Merlin</title>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>/* inline CSS */</style>
</head>
<body>
  <nav>
    <div class="logo">🧙 Magic Merlin</div>
    <div class="tabs">
      <button onclick="showTab('overview')" class="active">Overview</button>
      <button onclick="showTab('sessions')">Sessions</button>
      <button onclick="showTab('cron')">Cron</button>
      <button onclick="showTab('agents')">Agents</button>
      <button onclick="showTab('logs')">Logs</button>
    </div>
    <div class="status-badge" id="gateway-status">●</div>
  </nav>

  <main id="content">
    <!-- Tab content injected by JS -->
  </main>

  <script>/* inline JS */</script>
</body>
</html>
```

### CSS — dark theme (inline in `<style>`)
```css
:root {
  --bg: #0d1117;
  --surface: #161b22;
  --border: #30363d;
  --text: #c9d1d9;
  --accent: #58a6ff;
  --green: #3fb950;
  --red: #f85149;
  --yellow: #d29922;
  --radius: 8px;
}
* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: var(--bg); color: var(--text); font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; font-size: 14px; }
nav { display: flex; align-items: center; gap: 16px; padding: 12px 20px; background: var(--surface); border-bottom: 1px solid var(--border); }
.logo { font-weight: 700; font-size: 16px; color: var(--accent); }
.tabs button { background: none; border: none; color: var(--text); padding: 6px 12px; cursor: pointer; border-radius: 6px; font-size: 13px; }
.tabs button:hover, .tabs button.active { background: var(--accent); color: #fff; }
main { padding: 20px; }
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
.card { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 16px; }
.card h3 { font-size: 12px; text-transform: uppercase; color: #6e7681; margin-bottom: 12px; letter-spacing: 0.5px; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 8px 12px; text-align: left; border-bottom: 1px solid var(--border); font-size: 13px; }
th { color: #6e7681; font-weight: 500; }
.dot-green::before { content: "●"; color: var(--green); margin-right: 6px; }
.dot-red::before { content: "●"; color: var(--red); margin-right: 6px; }
.dot-yellow::before { content: "●"; color: var(--yellow); margin-right: 6px; }
.badge { display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 11px; }
.badge-green { background: rgba(63,185,80,0.15); color: var(--green); }
.badge-red { background: rgba(248,81,73,0.15); color: var(--red); }
pre { background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 12px; overflow-x: auto; font-size: 12px; line-height: 1.5; }
.log-error { color: var(--red); }
.log-warn { color: var(--yellow); }
.log-info { color: var(--text); }
.log-debug { color: #6e7681; }
#gateway-status { font-size: 20px; color: var(--green); margin-left: auto; }
```

### JS — SPA logic (inline in `<script>`)

```javascript
const API = '';  // same-origin

async function call(method, params = {}) {
  try {
    const r = await fetch('/call', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({method, params})
    });
    return await r.json();
  } catch(e) {
    return {error: e.message};
  }
}

let currentTab = 'overview';
let refreshTimer = null;

function showTab(name) {
  currentTab = name;
  document.querySelectorAll('.tabs button').forEach(b => b.classList.remove('active'));
  document.querySelector(`.tabs button[onclick="showTab('${name}')"]`).classList.add('active');
  renderTab();
}

async function renderTab() {
  const el = document.getElementById('content');
  el.innerHTML = '<p style="color:#6e7681;padding:20px">Loading…</p>';
  switch(currentTab) {
    case 'overview': el.innerHTML = await renderOverview(); break;
    case 'sessions': el.innerHTML = await renderSessions(); break;
    case 'cron':     el.innerHTML = await renderCron();     break;
    case 'agents':   el.innerHTML = await renderAgents();   break;
    case 'logs':     el.innerHTML = await renderLogs();     break;
  }
}

async function renderOverview() {
  const [status, channels] = await Promise.all([
    call('gateway.status'),
    call('channels.list')
  ]);
  const s = status.result || status;
  const ch = channels.result || [];
  return `
    <div class="grid">
      <div class="card">
        <h3>Gateway</h3>
        <table>
          <tr><td>Status</td><td><span class="dot-green"></span>Online</td></tr>
          <tr><td>Uptime</td><td>${s.uptime || '—'}</td></tr>
          <tr><td>Version</td><td>${s.version || '0.1.0'}</td></tr>
          <tr><td>Model</td><td>${s.model || '—'}</td></tr>
          <tr><td>PID</td><td>${s.pid || '—'}</td></tr>
        </table>
      </div>
      <div class="card">
        <h3>Channels (${ch.length})</h3>
        <table>
          ${ch.map(c => `<tr>
            <td class="${c.status==='connected'?'dot-green':'dot-red'}">${c.name}</td>
            <td>${c.platform || ''}</td>
            <td><span class="badge ${c.status==='connected'?'badge-green':'badge-red'}">${c.status}</span></td>
          </tr>`).join('')}
        </table>
      </div>
    </div>`;
}

async function renderSessions() {
  const r = await call('sessions.list', {limit: 50});
  const sessions = r.result || r || [];
  return `
    <div class="card">
      <h3>Sessions (${sessions.length})</h3>
      <table>
        <tr><th>Key</th><th>Model</th><th>Messages</th><th>Last Active</th><th>Tokens</th></tr>
        ${sessions.map(s => `<tr>
          <td>${s.key}</td><td>${s.model||'—'}</td>
          <td>${s.messageCount||0}</td><td>${s.lastActive||'—'}</td>
          <td>${s.tokensUsed||'—'}</td>
        </tr>`).join('')}
      </table>
    </div>`;
}

async function renderCron() {
  const r = await call('cron.list');
  const jobs = r.result || r || [];
  return `
    <div class="card">
      <h3>Cron Jobs (${jobs.length})</h3>
      <table>
        <tr><th>ID</th><th>Name</th><th>Schedule</th><th>Last Run</th><th>Status</th></tr>
        ${jobs.map(j => `<tr>
          <td style="font-family:monospace;font-size:11px">${(j.id||'').slice(0,8)}</td>
          <td>${j.name||'—'}</td><td>${j.schedule?.expr||j.schedule?.everyMs||'—'}</td>
          <td>${j.lastRun||'—'}</td>
          <td><span class="badge ${j.enabled?'badge-green':'badge-red'}">${j.enabled?'enabled':'disabled'}</span></td>
        </tr>`).join('')}
      </table>
    </div>`;
}

async function renderAgents() {
  const r = await call('agent.list');
  const agents = r.result || r || [];
  return `
    <div class="card">
      <h3>Agents (${agents.length})</h3>
      <table>
        <tr><th>ID</th><th>Model</th><th>Channels</th><th>Last Active</th></tr>
        ${agents.map(a => `<tr>
          <td>${a.id}</td><td>${a.model||'—'}</td>
          <td>${(a.channels||[]).join(', ')||'—'}</td>
          <td>${a.lastActive||'—'}</td>
        </tr>`).join('')}
      </table>
    </div>`;
}

async function renderLogs() {
  const r = await call('gateway.logs', {tail: 200});
  const lines = r.result || r || [];
  const html = lines.map(l => {
    const level = l.includes('[ERROR]')||l.includes('ERROR') ? 'error'
                : l.includes('[WARN]')||l.includes('WARN') ? 'warn'
                : l.includes('[DEBUG]') ? 'debug' : 'info';
    return `<span class="log-${level}">${escHtml(l)}</span>`;
  }).join('\n');
  return `<div class="card"><h3>Recent Logs</h3><pre>${html}</pre></div>`;
}

function escHtml(s) {
  return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

// Auto-refresh every 10s
async function refresh() {
  await renderTab();
  // Check gateway status
  try {
    await fetch('/health');
    document.getElementById('gateway-status').style.color = 'var(--green)';
  } catch {
    document.getElementById('gateway-status').style.color = 'var(--red)';
  }
}

setInterval(refresh, 10000);
renderTab();  // initial render
```

---

## Step 3: Add `gateway.logs` method

In `gateway/src/methods/mod.rs` or main.rs, add:
```rust
"gateway.logs" => {
    let tail = params.get("tail").and_then(Value::as_u64).unwrap_or(100) as usize;
    // Read log file from config or default path
    let log_path = state.config.log_path.clone()
        .unwrap_or_else(|| home_dir().join(".magicmerlin/gateway.log"));
    let lines = read_last_n_lines(&log_path, tail)?;
    json!({"result": lines})
}
```

---

## Step 4: Wire `/health` endpoint

Ensure gateway has:
```rust
.route("/health", get(|| async { Json(json!({"ok": true, "service": "magicmerlin"})) }))
```

---

## Rules
- No npm, no webpack, no external CDN — fully self-contained HTML
- Must serve at `GET /ui` from the running gateway
- Auto-refreshes every 10s
- Works offline (no external assets)
- `cargo build --workspace` clean

## Completion
```bash
openclaw system event --text "Sprint 7 done: Control UI live at /ui — dark SPA with Overview/Sessions/Cron/Agents/Logs tabs, auto-refresh, served statically from gateway" --mode now
```

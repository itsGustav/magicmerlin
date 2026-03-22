# Sprint 7 — Control UI

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
There is no Control UI yet. The gateway serves a static HTML page at GET /.
Goal: Build a full single-page dashboard served by the gateway, no external build tools needed.

## Strategy: Embedded HTML/JS (no npm, no webpack)
Build a self-contained HTML+JS+CSS Control UI that the gateway serves as a static file.
Use vanilla JS with fetch() calls to the gateway /call endpoint.
Embed the HTML as a Rust `include_str!()` literal — no separate build step.

---

## File Layout
```
gateway/
  static/
    index.html     ← main SPA (embed as include_str! in gateway)
    style.css      ← embedded in index.html via <style> tag
```

---

## index.html — Full SPA

Build a single HTML file (target: ~1500 lines) with:

### Header
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Magic Merlin</title>
  <style>
    /* Dark theme, monospace, compact */
    :root {
      --bg: #0d1117;
      --bg2: #161b22;
      --bg3: #21262d;
      --border: #30363d;
      --text: #c9d1d9;
      --text-dim: #8b949e;
      --accent: #58a6ff;
      --green: #3fb950;
      --red: #f85149;
      --yellow: #d29922;
      --purple: #bc8cff;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { background: var(--bg); color: var(--text); font-family: 'SF Mono', 'Consolas', monospace; font-size: 13px; }
    /* Nav tabs */
    .nav { display: flex; background: var(--bg2); border-bottom: 1px solid var(--border); padding: 0 16px; }
    .nav-tab { padding: 12px 16px; cursor: pointer; border-bottom: 2px solid transparent; color: var(--text-dim); }
    .nav-tab.active { color: var(--accent); border-bottom-color: var(--accent); }
    /* Cards */
    .card { background: var(--bg2); border: 1px solid var(--border); border-radius: 6px; padding: 16px; margin: 8px 0; }
    /* Tables */
    table { width: 100%; border-collapse: collapse; }
    th { color: var(--text-dim); font-weight: normal; text-align: left; padding: 8px; border-bottom: 1px solid var(--border); }
    td { padding: 8px; border-bottom: 1px solid var(--border); }
    /* Status dots */
    .dot-green::before { content: '●'; color: var(--green); margin-right: 6px; }
    .dot-red::before { content: '●'; color: var(--red); margin-right: 6px; }
    .dot-yellow::before { content: '●'; color: var(--yellow); margin-right: 6px; }
    /* Buttons */
    button { background: var(--bg3); color: var(--text); border: 1px solid var(--border); padding: 6px 12px; border-radius: 4px; cursor: pointer; }
    button:hover { background: var(--accent); color: #000; }
    /* Input */
    input, textarea, select { background: var(--bg3); color: var(--text); border: 1px solid var(--border); padding: 6px 10px; border-radius: 4px; width: 100%; }
    /* Layout */
    .container { max-width: 1200px; margin: 0 auto; padding: 16px; }
    .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
    .header { display: flex; justify-content: space-between; align-items: center; padding: 16px; background: var(--bg2); border-bottom: 1px solid var(--border); }
    .logo { font-size: 18px; font-weight: bold; color: var(--accent); }
    .badge { background: var(--bg3); color: var(--text-dim); padding: 2px 8px; border-radius: 10px; font-size: 11px; }
  </style>
</head>
```

### Navigation Tabs
```
🧙 Magic Merlin   [Overview] [Agents] [Sessions] [Cron] [Channels] [Logs] [Config]
```

### Tab 1: Overview
- Gateway status card: version, uptime, PID, model, port
- Active agents count, session count, cron job count
- Recent activity feed (last 10 log lines)
- Quick actions: Restart Gateway, Compact All Sessions

### Tab 2: Agents
- Table: Agent Name | Model | Status | Channels | Last Active
- Click agent → modal with config, session count, run history
- Button: "Run turn" — text input → POST agent.run → show reply

### Tab 3: Sessions
- Table: Session Key | Messages | Tokens | Last Active | Model | Actions
- Actions: View transcript, Delete, Compact
- Click row → transcript viewer (chat bubble style)

### Tab 4: Cron
- Table: ID | Name | Schedule | Last Run | Next Run | Status | Actions
- Actions: Run now, Enable/Disable, Delete
- "Add Job" form: name, schedule expression, message, sessionTarget

### Tab 5: Channels
- Table: Platform | Account | Status | Last Message | Error
- Status dots: green=connected, red=error, yellow=degraded
- Per-channel: last 3 messages preview

### Tab 6: Logs
- Live log stream (poll /call with method logs.tail every 2s)
- Color-coded: ERROR=red, WARN=yellow, INFO=white, DEBUG=dim
- Filter input, auto-scroll toggle

### Tab 7: Config
- JSON config viewer/editor (pretty-printed, editable textarea)
- Save button → POST gateway.config.patch
- Reset button

### JavaScript Architecture
```javascript
const API = {
  async call(method, params = {}) {
    const r = await fetch('/call', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({method, params})
    });
    return r.json();
  }
};

// Auto-refresh every 3s
const REFRESH_INTERVAL = 3000;

// Tab routing
const tabs = ['overview', 'agents', 'sessions', 'cron', 'channels', 'logs', 'config'];
let currentTab = 'overview';

function switchTab(tab) {
  currentTab = tab;
  document.querySelectorAll('.nav-tab').forEach(t => t.classList.toggle('active', t.dataset.tab === tab));
  document.querySelectorAll('.tab-content').forEach(c => c.style.display = c.id === tab ? '' : 'none');
  renderTab(tab);
}

async function renderTab(tab) {
  switch(tab) {
    case 'overview': await renderOverview(); break;
    case 'agents': await renderAgents(); break;
    case 'sessions': await renderSessions(); break;
    case 'cron': await renderCron(); break;
    case 'channels': await renderChannels(); break;
    case 'logs': await renderLogs(); break;
    case 'config': await renderConfig(); break;
  }
}

// Each render function: call API, build HTML, inject into tab div
async function renderOverview() {
  const status = await API.call('gateway.status');
  // Build and inject HTML...
}

// Auto-refresh loop
setInterval(() => renderTab(currentTab), REFRESH_INTERVAL);

// Init
switchTab('overview');
```

---

## Gateway Integration

In `gateway/src/main.rs`, serve the UI:

```rust
// Add to router:
.route("/", get(serve_index))
.route("/index.html", get(serve_index))

const UI_HTML: &str = include_str!("../static/index.html");

async fn serve_index() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        UI_HTML
    )
}
```

---

## Additional /call methods needed for UI

Ensure these gateway methods work (add stubs if missing):
- `logs.tail { limit?, since? }` → return last N log lines as `[{level, message, timestamp}]`
- `gateway.status` → `{version, uptime, pid, model, port, agent_count, session_count, cron_count}`

---

## Rules
- Single HTML file, no build step, no npm, no external CDN (fully offline capable)
- Dark theme, clean monospace aesthetic
- All data from gateway /call RPC
- `cargo build --workspace` clean

## Completion
```bash
openclaw system event --text "Sprint 7 done: Control UI SPA — 7 tabs (Overview/Agents/Sessions/Cron/Channels/Logs/Config), live refresh, transcript viewer, cron management, served by gateway at GET /" --mode now
```

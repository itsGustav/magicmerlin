# Sprint 4 — Agent B: CLI Advanced + TUI

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
This sprint handles the advanced CLI commands + a Ratatui terminal dashboard.

## Part 1: Advanced CLI Commands

### 1. `magicmerlin browser start/stop/status/tabs`
```rust
// start: call gateway browser.start → confirm
// stop: call gateway browser.stop
// status: call gateway browser.status → show profile, open tabs count
// tabs: call gateway browser.tabs → tabulate title | url | id
```

### 2. `magicmerlin nodes list`
```rust
// Call nodes.list → tabulate: id | name | platform | last_seen | paired
```

### 3. `magicmerlin nodes describe <id>`
```rust
// Call nodes.describe → show full node info
```

### 4. `magicmerlin nodes notify <id> --title <t> --body <b>`
```rust
// Call nodes.notify
```

### 5. `magicmerlin approvals set <code> <allow-once|allow-always|deny>`
```rust
// Call approvals.set → confirm
```

### 6. `magicmerlin approvals pending`
```rust
// Call approvals.pending → list pending requests
```

### 7. `magicmerlin context show [<session-key>]`
```rust
// Call sessions.show → print current context window usage
// Show: token count, % of limit, model, compaction history
```

### 8. `magicmerlin hooks list`
```rust
// Call hooks.list → tabulate registered webhooks
```

### 9. `magicmerlin hooks add --url <url> --event <event>`
```rust
// Call hooks.add
```

### 10. `magicmerlin system event --text <text> [--mode now|next-heartbeat]`
```rust
// POST to gateway: injects a system event into the main session
// Used by agents to wake the main session
```

### 11. `magicmerlin acp list`
```rust
// Call acp.list → show running ACP sessions
```

### 12. `magicmerlin acp spawn --agent <id> --task <task>`
```rust
// Call acp.spawn → show session key
```

### 13. `magicmerlin subagents list`
```rust
// Call subagents.list → tabulate running sub-agents
```

### 14. `magicmerlin subagents kill <session>`
```rust
// Call subagents.kill → confirm
```

### 15. `magicmerlin docs [<page>]`
```rust
// Without arg: open browser to local docs or https://docs.magicmerlin.dev
// With arg: search docs pages, print matching content to terminal
// Use: open or xdg-open to launch browser
```

### 16. `magicmerlin completions <shell>`
```rust
// Generate shell completions using clap_complete
// shells: bash, zsh, fish, elvish, powershell
// Print to stdout: eval "$(magicmerlin completions zsh)"
use clap_complete::{generate, Shell};
```

Add `clap_complete = "4"` to cli/Cargo.toml.

### 17. `magicmerlin qr [--url <url>]`
```rust
// Generate a QR code in the terminal (Unicode block chars)
// Without url: show pairing QR for this gateway
// With url: encode arbitrary URL
use qrcode::{QrCode, render::unicode};
```

Add `qrcode = "0.14"` to cli/Cargo.toml.

---

## Part 2: Ratatui TUI Dashboard

Add `ratatui = "0.27"` and `crossterm = "0.28"` to cli/Cargo.toml.

Implement `magicmerlin tui` command that opens a terminal dashboard:

### Layout
```
┌─ Magic Merlin ──────────────────────────────────────┐
│ [A]gents  [S]essions  [C]ron  [L]ogs  [?]Help  [Q]quit │
├────────────────────────────────────────────────────────┤
│ AGENTS                          │ ACTIVE SESSIONS      │
│ ● Gustav (main)   sonnet  ✓    │ telegram:8527778539  │
│ ● Lobster Prime   minimax ✓    │ agent:gustav:main    │
│ ○ PayLobster      minimax ✗    │                      │
│                                 │                      │
├────────────────────────────────────────────────────────┤
│ CRON JOBS                       │ RECENT LOGS          │
│ morning-brief  0 9 * * *  ✓    │ [14:23] agent turn   │
│ heartbeat     */30 * * * * ✓   │ [14:22] cron fired   │
│                                 │ [14:20] msg sent     │
├────────────────────────────────────────────────────────┤
│ Gateway: ● online :18789 | Model: sonnet | Uptime: 4h │
└────────────────────────────────────────────────────────┘
```

### Implementation

```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};

pub struct App {
    tab: usize,           // 0=Overview 1=Agents 2=Sessions 3=Cron 4=Logs
    agents: Vec<AgentStatus>,
    sessions: Vec<SessionStatus>,
    cron_jobs: Vec<CronJobStatus>,
    logs: Vec<String>,
    gateway_status: GatewayStatus,
    tick: u64,
}

impl App {
    // Refresh data from gateway every 2 seconds
    pub async fn refresh(&mut self, gateway_url: &str) -> Result<()>;
    
    // Handle key events
    pub fn on_key(&mut self, key: KeyCode) -> bool; // returns true to quit
}

pub fn draw(frame: &mut Frame, app: &App) {
    // Draw the full layout
}

pub async fn run_tui(gateway_url: &str) -> Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::default();
    
    // Event loop
    loop {
        terminal.draw(|f| draw(f, &app))?;
        
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if app.on_key(key.code) {
                    break;
                }
            }
        }
        
        // Refresh every 2s
        if app.tick % 20 == 0 {
            app.refresh(gateway_url).await.ok();
        }
        app.tick += 1;
    }
    
    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}
```

### Key bindings
- `q` / `Ctrl-C` → quit
- `Tab` / `1-5` → switch tab
- `r` → force refresh
- `↑↓` → scroll lists
- `Enter` → show detail for selected item

---

## Rules
- `cargo build --workspace` must pass clean
- TUI should gracefully handle gateway offline (show "Gateway offline" banner instead of crashing)
- Shell completions must actually work: `magicmerlin completions zsh | head -5` should output valid zsh completion code
- QR code should render in terminal without external deps

## Completion
```bash
openclaw system event --text "Sprint 4B done: browser/nodes/approvals/context/hooks/system/acp/subagents CLI commands wired, shell completions, QR code, Ratatui TUI dashboard with tabs/live refresh" --mode now
```

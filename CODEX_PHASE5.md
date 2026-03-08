# Phase 5: CLI — Build Instructions

Phases 0-4 complete (16 crates, 19.7K lines). Now build the full CLI with all 40+ commands matching OpenClaw.

## What To Build — extend `cli` crate

The existing cli/src/main.rs has a basic scaffold. Replace it with a full CLI using clap that mirrors every OpenClaw command.

### Command Groups (all as clap subcommands)
1. **status** — Channel health, session info, model, config summary. Connect to gateway WebSocket, call `health` + `status` methods, format output.
2. **setup** / **configure** — Interactive wizard (credentials, channels, gateway, agent defaults). Use dialoguer for prompts.
3. **onboard** — First-run setup flow
4. **health** / **doctor** — System diagnostics (check gateway running, channels connected, models accessible, disk space, config valid)
5. **dashboard** — Open Control UI URL in browser (`open http://localhost:18789/ui`)
6. **tui** — Terminal UI using ratatui (agents list, sessions, cron jobs, logs viewer, real-time updates)
7. **help** — Full help tree
8. **completion** — Shell completions for bash, zsh, fish (clap_complete)
9. **version** — Version output
10. **update** — Self-update (check GitHub releases, download, replace binary)
11. **reset** / **uninstall** — Cleanup local state

### Agent Commands
12. **agent** — Run one agent turn via gateway (`agent.run` method)
13. **agents list/add/remove/config** — Agent CRUD (list all agents, add new, remove, show/edit config)
14. **models list/status/auth** — Model listing, status check, auth management

### Gateway Commands
15. **gateway start/stop/restart/status** — Service lifecycle. Start spawns daemon, writes PID. Stop sends SIGTERM. Status checks PID + port.
16. **gateway call <method>** — Direct JSON-RPC method invocation with --params JSON
17. **daemon** — Legacy alias for gateway

### Channel Commands
18. **channels login/logout/status** — Per-channel authentication and health
19. **message send** — Direct message send (--target, --message, --channel)
20. **directory** — Contact/group ID lookup
21. **pairing list/approve/deny** — DM pairing management

### Data Commands
22. **sessions list/show/delete/compact** — Session CRUD
23. **memory search/get** — Memory file operations
24. **cron list/add/edit/rm/run/enable/disable/runs** — Full cron CRUD
25. **logs** — Log viewer (tail -f style with --follow)
26. **hooks** / **webhooks** — Webhook management

### Security/Admin Commands
27. **config get/set/unset/file/validate** — Config operations
28. **security audit** — Security scanner (check open ports, permissions, config issues)
29. **secrets reload** — Runtime secrets reload
30. **sandbox** — Container management
31. **approvals list/approve/deny** — Exec approval management
32. **plugins list/enable/disable/install** — Plugin management
33. **skills list/inspect** — Skill listing and SKILL.md display
34. **dns** — DNS/Tailscale helpers
35. **devices** — Device pairing
36. **nodes list/describe/run** — Node management
37. **qr** — QR code generation for pairing
38. **browser start/stop/status** — Browser lifecycle
39. **acp** — Agent Control Protocol tools
40. **docs** — Open documentation URL
41. **system event/heartbeat/presence** — System events

### Global Flags (on root command)
- `--dev` — Use dev profile
- `--profile <name>` — Named profile
- `--log-level <level>` — Override log level
- `--no-color` — Disable ANSI
- `-V` / `--version` — Version
- `-h` / `--help` — Help

### Implementation Notes
- Use `clap` derive macros for all commands
- Gateway communication: connect to `ws://127.0.0.1:18789` (or configured URL), send JSON-RPC
- For commands that need gateway: check if running first, suggest `magicmerlin gateway start` if not
- Output: default human-readable, `--json` flag for JSON output on all commands
- Use `dialoguer` for interactive prompts (configure, auth add)
- Use `clap_complete` for shell completions
- Use `ratatui` for TUI (basic layout: sidebar with agents, main area with sessions/logs)
- `open` crate or `std::process::Command` for opening URLs/dashboards

### Dependencies to add to cli/Cargo.toml
- clap (already there) + clap_complete
- dialoguer = "0.11"
- ratatui = "0.29" + crossterm = "0.28"
- tokio-tungstenite (for gateway WS client)
- tabled = "0.17" (for table output)
- colored = "2" (for colored output)
- open = "5" (for opening URLs)

Ensure `cargo check` + `cargo test` pass. Commit when done.

When completely finished, run:
openclaw system event --text "Phase 5 complete: CLI (40+ commands) built and tested" --mode now

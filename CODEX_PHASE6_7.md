# Phase 6+7: Plugins, Skills, ACP + Control UI, Testing — Build Instructions

Final phases. 16 crates exist, 20K lines. Build the extension system and polish.

## Phase 6: Plugins, Skills & ACP

### 6.1 Plugin System (new crate: `plugins`)
1. **Plugin trait**: name, version, description, init/start/stop lifecycle
2. **Plugin discovery**: scan plugin directories, load manifest files
3. **Plugin registry**: register, enable, disable, list
4. **Bundled plugins** (implement as built-in):
   - `session-memory`: Auto-save session context to memory files
   - `command-logger`: Log all commands to file
   - `boot-md`: Load AGENTS.md/SOUL.md/etc at session start
   - `bootstrap-extra-files`: Load additional workspace files
5. **Plugin runtime isolation**: Each plugin gets its own config namespace

### 6.2 Skills System (in `plugins` crate or new `skills` crate)
1. **Skill discovery**: Scan `~/.openclaw/workspace-*/skills/`, bundled skills, clawhub installed
2. **SKILL.md loader**: Parse skill metadata (name, description, requires)
3. **Skill prompt injection**: Generate `<available_skills>` XML block for system prompt
4. **Skill script execution**: Run skill scripts via exec tool
5. **Skill dependency checking**: Verify requiredEnv, primaryEnv, required binaries

### 6.3 ACP — Agent Control Protocol (new crate: `acp`)
1. **ACP runtime**: Spawn external coding agents (Claude Code, Codex, OpenCode, Gemini, Pi) as sub-processes
2. **ACP control plane**: Track spawned sessions, stream output, handle completion
3. **ACPX integration**: Extension protocol for coding agent dispatch
4. **Thread-bound sessions**: Persistent sessions tied to chat threads
5. **Agent harness configuration**: allowedAgents list, maxConcurrentSessions, TTL

## Phase 7: Control UI, Security, Testing

### 7.1 Control UI (embedded in `gateway`)
1. **Static file server**: Serve a simple HTML/JS dashboard from gateway
2. **Dashboard pages**: 
   - Overview: agents, active sessions, cron status, system health
   - Sessions: list, view transcript, compact, delete
   - Cron: list jobs, run history, enable/disable
   - Config: view/edit configuration
   - Logs: real-time log viewer via WebSocket
3. **Build as embedded HTML/JS** (use include_str! or serve from assets dir)
4. **Dark theme, mobile responsive**
5. **WebSocket for real-time updates** from gateway

### 7.2 Security Module (new module in `config` or standalone)
1. **Security audit**: Check for common issues:
   - Open DM policies on public bots
   - Missing sandbox configuration
   - Weak auth (no gateway token)
   - Exposed ports
   - Stale sessions with high token usage
2. **Workspace-only FS restrictions**: Validate file operations stay within workspace
3. **Tool deny lists**: Configurable blocked tools per agent
4. **Trusted proxy configuration**: For reverse proxy setups

### 7.3 Testing (across all crates)
Add comprehensive tests:
1. **Config**: Load/save roundtrip, env overlay, profile isolation, get/set/unset paths
2. **Providers**: Request formatting per provider, auth header generation, failover chain, rate limit handling
3. **Agent**: System prompt assembly, tool call loop (mock provider), session management
4. **Gateway**: WebSocket connection, JSON-RPC routing, cron scheduling
5. **Channels**: Message normalization, platform formatting, DM policy, mention gating, message splitting
6. **Media**: Link extraction, TTS request formatting
7. **CLI**: Command parsing, help output
8. **Sessions**: Transcript repair (broken tool_use/tool_result pairs), compaction
9. **Integration**: End-to-end agent turn with mock LLM

### 7.4 Documentation
1. Update README.md with:
   - Installation instructions (cargo install, Docker)
   - Quick start guide
   - Migration from OpenClaw
   - Architecture overview
   - All CLI commands listed
2. Add CHANGELOG.md
3. Add man page generation (clap_mangen)

## Quality Requirements
- Every public function has doc comments
- No unwrap() in library code
- Clippy clean
- Target: 80+ tests across all crates
- `cargo check` + `cargo test` + `cargo clippy` must pass

When completely finished, run:
openclaw system event --text "Phases 6+7 complete: Plugins, Skills, ACP, Control UI, Security, Testing — Magic Merlin v0.2 ready" --mode now

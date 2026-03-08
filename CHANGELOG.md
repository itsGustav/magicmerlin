# Changelog

## [0.2.0] - 2026-03-08

### Added
- New `magicmerlin-plugins` crate:
  - Plugin trait and lifecycle runtime (`init/start/stop`)
  - Bundled plugins (`session-memory`, `command-logger`, `boot-md`, `bootstrap-extra-files`)
  - Plugin discovery and manifest scanning
  - Plugin registry with enable/disable and isolated config namespaces
  - Skills subsystem: discovery, `SKILL.md` metadata parsing, dependency checks, XML prompt block generation, script execution
- New `magicmerlin-acp` crate:
  - ACP runtime for spawning external coding-agent subprocesses
  - Session control plane with event streaming and persistent thread-bound sessions
  - ACPX dispatch integration and harness policy config (`allowedAgents`, `maxConcurrentSessions`, `ttlSeconds`)
- Gateway integration:
  - ACP endpoints and JSON-RPC methods (`acp.spawn`, `acp.sessions.list`, `acp.cleanup`)
  - Embedded Control UI with overview/sessions/cron/config/logs pages
  - Live event log polling endpoint (`/events`)
  - Security audit endpoint (`/security/audit`) and RPC method (`security.audit`)
- Security module in `magicmerlin-config`:
  - Audit checks for DM policy, sandbox presence, weak auth, exposed bind, stale sessions, trusted proxy validation
  - Workspace path restriction validation helper
  - Tool deny-list helper per agent/global scope

### Changed
- `gateway` plugin access now delegates to the new `magicmerlin-plugins` crate.
- CLI `security audit` now calls gateway `security.audit` instead of returning placeholder data.
- README rewritten with installation, quick start, migration, architecture, and full CLI command map.

### Testing
- Workspace test count increased to 85.
- Added extensive unit coverage for plugins, skills, ACP runtime, and security auditing.

[0.2.0]: https://example.invalid/magicmerlin/releases/0.2.0

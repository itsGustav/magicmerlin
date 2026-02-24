# Snapshot: Runtime tool surface (from OpenClaw agent prompt)

- captured_at: 2026-02-24T04:00:00-05:00
- source: current OpenClaw runtime prompt (developer-provided tool schemas)
- note: this is a **manual snapshot** (copied from the running agent's tool definitions). If OpenClaw later provides a CLI/API to export tool schemas, prefer that and treat this file as deprecated.

## Tools (namespaces)

### functions.* (primary tool surface)

- `functions.read` — read file contents (text or image). Args: `path|file_path`, `offset`, `limit`.
- `functions.write` — create/overwrite file. Args: `path|file_path`, `content`.
- `functions.edit` — exact-text replace. Args: `path|file_path`, `oldText|old_string`, `newText|new_string`.
- `functions.exec` — run shell commands. Args: `command`, `workdir`, `yieldMs`, `background`, `timeout`, `pty`, `elevated`, `host`, `security`, `ask`, `node`.
- `functions.process` — manage running exec sessions. Args: `action`, `sessionId`, `data`, `keys`, `hex`, `literal`, `text`, `bracketed`, `eof`, `offset`, `limit`, `timeout`.

- `functions.web_search` — web search (Brave API). Args: `query`, `count`, `country`, `search_lang`, `ui_lang`, `freshness`.
- `functions.web_fetch` — fetch URL → extracted markdown/text. Args: `url`, `extractMode`, `maxChars`.

- `functions.browser` — browser automation (OpenClaw browser control server). Args include: `action`, `profile`, `targetId`, `targetUrl`, `refs`, and `request` (click/type/etc).
- `functions.canvas` — control node canvases. Args: `action`, `node`, `url`, `javaScript`, etc.
- `functions.nodes` — paired node control (notify/camera/screen/location/run/invoke). Args: `action`, `node`, and action-specific fields.

- `functions.message` — send/edit/delete/react messages via channel plugins. Args: `action`, `channel`, `target(s)`, `message`, plus many provider-specific fields.

- `functions.agents_list` — list agent ids available for `sessions_spawn`.
- `functions.sessions_list` — list sessions (filters + last messages).
- `functions.sessions_history` — fetch message history for a session.
- `functions.sessions_send` — send message into another session.
- `functions.sessions_spawn` — spawn a sub-agent session (run/session).
- `functions.subagents` — list/steer/kill spawned sub-agents.

- `functions.session_status` — /status-equivalent session status card (usage/time/cost) + optional model override.

- `functions.image` — analyze images with vision model. Args: `image|images`, `prompt`, etc.

- `functions.memory_search` — semantic search in MEMORY.md + memory/*.md (+ transcripts). Args: `query`, `maxResults`, `minScore`.
- `functions.memory_get` — safe snippet read from memory file. Args: `path`, `from`, `lines`.

- `functions.tts` — text-to-speech. Args: `text`, `channel`.

### multi_tool_use.* (wrapper)

- `multi_tool_use.parallel` — run multiple `functions.*` tools in parallel (only when truly parallelizable).

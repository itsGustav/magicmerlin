# Deepening Pass 2: Gateway + Channels — Full Implementation

Current: 34.8K lines, 18 crates. Deepen gateway and channels to production quality.

## Rules
- Read existing code first, EXTEND don't rewrite
- Full error handling, edge cases
- Comprehensive tests
- cargo check + cargo test must pass

## 1. Gateway Crate (target: 12000+ lines)

### WebSocket Server (gateway/src/ws.rs or similar)
- Full tokio-tungstenite WebSocket server
- JSON-RPC 2.0 protocol: {jsonrpc, method, params, id} → {jsonrpc, result, id} or {jsonrpc, error, id}
- Authentication: bearer token from config, reject unauthorized connections
- Connection management: track all clients, broadcast capability, per-client state
- Keepalive: ping every 30s, disconnect on 3 missed pongs
- Concurrent request handling with tokio::spawn per request
- Request ID tracking for response correlation

### Method Router (gateway/src/methods/)
Create individual handler files for each method group:
- health.rs: health check with channel statuses, uptime, version
- status.rs: full system status (agents, sessions count, models, cron summary)
- agent_run.rs: execute agent turn, stream partial results back, abort support
- sessions.rs: list (with filters: agent, kind, active), get, send, spawn, compact, delete
- cron.rs: list, add, edit, rm, run (immediate), enable, disable, runs history
- config.rs: get (dot-notation), set, unset, validate
- approvals.rs: list pending, approve, deny
- plugins.rs: list, enable, disable
- system.rs: event, heartbeat, presence, restart

### Run Queue
- Per-session run queue (one active run at a time per session)
- Queue with configurable max depth (default 5)
- Run timeout handling (kill after configurable timeout)
- Abort: cancel in-progress run, send abort signal to LLM call
- Run state tracking: pending, running, completed, failed, aborted
- Streaming: pipe LLM stream chunks back to WebSocket client in real-time

### Service Management (gateway/src/service.rs)
- macOS LaunchAgent: generate plist, launchctl load/unload/list
- Linux systemd: generate unit file, systemctl enable/start/stop
- PID file management: write on start, clean on stop, stale detection
- Graceful shutdown: SIGTERM handler, drain active runs, close connections
- Port binding: configurable, error on port in use with helpful message

### HTTP Endpoints (alongside WebSocket)
- GET /health — JSON health check
- GET /ui — serve Control UI static files
- POST /api/v1/message — HTTP API for sending messages (non-WS clients)
- GET /api/v1/sessions — REST API for session listing
- Middleware: CORS, request logging, auth token check

## 2. Channels Crate (target: 10000+ lines)

### Telegram (channels/src/telegram/ — target 3000+ lines)
- Full Bot API client with reqwest:
  - getUpdates polling loop (long polling with timeout=30)
  - getMe for bot identity on startup
  - sendMessage with MarkdownV2 parse mode
  - editMessageText for message editing
  - deleteMessage
  - setMessageReaction for emoji reactions
  - sendChatAction (typing indicator)
  - sendPhoto, sendDocument, sendVoice, sendVideo, sendVideoNote
  - getFile + download file to local path
  - sendPoll
  - Message effects (message_effect_id parameter)
  - Forum/topic support (message_thread_id)
  - Inline keyboards (InlineKeyboardMarkup)
  - Callback query handling (answerCallbackQuery)
  - Reply parameters (reply_to_message_id, quote)
- Multi-account support: one bot per agent, iterate config
- MarkdownV2 escaping (escape special chars: _*[]()~`>#+-=|{}.!)
- Message splitting at 4096 chars (split on newlines, preserve formatting)
- Rate limiting: respect Telegram 429 with retry-after
- Reconnect on network errors with exponential backoff
- Update offset tracking (prevent duplicate processing)

### Discord (channels/src/discord/ — target 3000+ lines)
- Gateway WebSocket v10:
  - Identify with intents (GUILDS, GUILD_MESSAGES, MESSAGE_CONTENT, DIRECT_MESSAGES)
  - Heartbeat loop (heartbeat_interval from Hello)
  - Resume on disconnect (session_id + sequence)
  - Reconnect with backoff
- REST API client:
  - POST /channels/{id}/messages — send message
  - PATCH /channels/{id}/messages/{id} — edit
  - DELETE /channels/{id}/messages/{id} — delete
  - PUT /channels/{id}/messages/{id}/reactions/{emoji}/@me — react
  - POST /channels/{id}/threads — create thread
  - GET /guilds/{id}/members — list members
- Embed builder (title, description, fields, color, footer, thumbnail)
- Message splitting at 2000 chars
- Rate limit handling (per-route buckets, X-RateLimit headers)
- Slash command registration (POST /applications/{id}/commands)
- Presence updates (updatePresence in gateway)

### WhatsApp (channels/src/whatsapp/ — target 1500+ lines)
- Bridge process manager: spawn/monitor external WhatsApp bridge (Baileys or similar)
- IPC protocol: JSON messages over stdin/stdout
- QR code pairing: display QR, wait for scan confirmation
- Send text, image, document, voice messages
- Receive handler: parse inbound messages from bridge
- Group chat: group JID handling, participant info
- Reactions: react with emoji
- Read receipts: mark as read

### Signal (channels/src/signal/ — target 1000+ lines)
- signal-cli wrapper: spawn process, communicate via JSON-RPC or dbus
- Send/receive messages
- Group support (group IDs, membership)
- Attachment handling
- Reactions
- Trust/safety number verification
- Daemon mode monitoring

### Slack (channels/src/slack/ — target 1000+ lines)
- Web API client: chat.postMessage, conversations.list, users.info
- Socket Mode: WebSocket connection for real-time events
- Block Kit: section, actions, divider, context blocks
- Thread support (thread_ts)
- File upload (files.upload)
- Emoji reactions (reactions.add/remove)
- Rate limiting (Tier 1-4 methods)
- App mentions event handling

### iMessage (channels/src/imessage/ — target 500+ lines)
- osascript bridge: send via 'tell application "Messages"'
- SQLite monitor: poll ~/Library/Messages/chat.db for new messages
- Parse message rows: text, sender, date, chat_id
- Group chat detection (chat_identifier with 'chat' prefix)
- Image sending via osascript
- Dedup: track last processed rowid

### LINE + Web (channels/src/line/ + web/ — target 500+ each)
- LINE: Messaging API webhook receiver, reply/push, flex messages
- Web: WebSocket chat endpoint, session auth, typing indicators

Commit after gateway deepening, then after channels deepening.

When completely finished, run:
openclaw system event --text "Deepening Pass 2 complete: Gateway (12K+) + Channels (10K+)" --mode now

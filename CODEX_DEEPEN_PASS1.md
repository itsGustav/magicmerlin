# Deepening Pass 1: Providers + Agent Engine — Full Implementation

The current codebase has 22K lines across 18 crates. Every subsystem exists but is thin. Your job is to deepen each module to production quality. This is pass 1 of 4 — focus on providers and agent.

## Rules
- Read every existing .rs file in the target crates before writing
- EXTEND existing code, don't rewrite from scratch
- Add real error handling, edge cases, retry logic
- Add comprehensive tests (unit + integration)
- Every public function must have doc comments
- No unwrap() in library code
- cargo check + cargo test must pass after each crate

## 1. Providers Crate (target: 8000+ lines)

### OpenAI Provider (providers/src/providers/openai.rs)
- Full chat completions API: system/user/assistant/tool messages
- Tool call handling: function_call format, parallel tool calls
- Vision: image_url content parts (base64 and URL)
- Streaming: SSE parsing, delta content assembly, tool call streaming
- Response format: json_object, json_schema
- Reasoning models: o-series with reasoning_effort parameter
- Token counting: tiktoken-rs integration or accurate char/4 heuristic
- Error mapping: 400 (bad request), 401 (auth), 429 (rate limit with Retry-After), 500+

### Anthropic Provider (providers/src/providers/anthropic.rs)  
- Full Messages API: system as top-level param (not in messages array)
- Tool use: tool_use content blocks, tool_result blocks
- Vision: base64 image source blocks
- Streaming: event stream parsing (message_start, content_block_start, content_block_delta, message_delta)
- Prompt caching: cache_control headers, ephemeral cache points
- Extended thinking: thinking content blocks, budget_tokens
- PDF support: document source blocks with base64
- Max tokens: required param, handle model-specific defaults
- Error handling: overloaded (529), rate limit (429)

### Google Provider (providers/src/providers/google.rs)
- Full generateContent API with proper role mapping
- Tool calls: functionCall/functionResponse parts
- Vision: inlineData parts with base64
- Streaming: streamGenerateContent endpoint
- Safety settings configuration
- System instruction (separate from contents)

### OpenAI-Compatible Module (providers/src/providers/openai_compat.rs)
- Robust SSE stream parser (handle data:, event:, retry: fields)
- Proper [DONE] sentinel handling
- Connection keepalive
- Configurable base URL + auth header name
- Rate limit extraction from response headers

### Router (providers/src/router.rs)
- Thread-safe provider registry with Arc<RwLock<>>
- Per-provider rate limit tracking with token bucket
- Circuit breaker pattern (open after N consecutive failures, half-open after cooldown)
- Detailed failover logging
- Cost calculation with model-specific rates
- Request/response middleware hooks

### Auth (providers/src/auth.rs)
- OAuth token refresh flow (OpenAI Codex style)
- Token expiry tracking and proactive refresh
- Multi-key rotation with round-robin
- Secure token storage (no logging of tokens)
- Auth health check endpoint

### Model Registry (providers/src/model_registry.rs)
- Full model database: all OpenAI, Anthropic, Google, xAI, Groq, Mistral models
- Context window sizes, max output tokens, costs
- Capability flags: vision, tools, streaming, json_mode, reasoning
- Dynamic model addition from config
- Model recommendation based on task requirements

## 2. Agent Crate (target: 5000+ lines)

### Engine (agent/src/engine.rs)
- Full agent turn loop with proper error recovery
- Tool call execution with timeout per tool
- Parallel tool call support (execute multiple tools concurrently)
- Max turns limit (prevent infinite loops)
- Token budget tracking across turns
- Streaming response assembly
- Abort handling (cancel in-progress turn)

### System Prompt Assembly (agent/src/system_prompt.rs)
- Load all workspace files: AGENTS.md, SOUL.md, USER.md, IDENTITY.md, TOOLS.md, MEMORY.md, HEARTBEAT.md, BOOTSTRAP.md
- Character limit per file (configurable, default 4000)
- Smart truncation (preserve headers and recent content)
- Skills injection: build <available_skills> XML block
- Runtime metadata: date, time, timezone, model, channel, agent info
- Inbound context injection (sender info, chat type, reply context)
- Tool schema injection (JSON schemas for all available tools)

### Session Management (agent/src/session.rs)
- Full session lifecycle: create, load, append, compact, delete
- Compaction: summarize old messages, preserve recent context
- Pre-compaction memory flush: extract important info, write to memory files
- Token tracking: accurate per-message, cumulative per-session
- Context window management: trigger compaction at configurable threshold (default 80%)
- Session metadata: model overrides, delivery context, cost accumulation
- Concurrent access protection (file locks)

### Agent Registry (agent/src/registry.rs)
- Load all agents from ~/.openclaw/agents/*/
- Agent config: model, fallbacks, workspace, identity, heartbeat settings
- Agent lifecycle: initialize, start session, stop
- Multi-agent routing: determine which agent handles a message
- Agent health monitoring

### Message Queue (agent/src/queue.rs)
- Collect mode: batch messages within configurable window
- Priority handling: urgent messages bypass queue
- Deduplication: ignore duplicate messages
- Queue persistence: survive process restart
- Backpressure: limit queue size per session

### Heartbeat (agent/src/heartbeat.rs)
- Load HEARTBEAT.md content
- Parse task list from heartbeat file
- Execute heartbeat tasks
- HEARTBEAT_OK response when nothing to do
- Heartbeat state tracking (last check times per task type)
- Time-of-day awareness (quiet hours)

Commit after providers deepening, then again after agent deepening.

When completely finished, run:
openclaw system event --text "Deepening Pass 1 complete: Providers (8K+) + Agent (5K+) fully implemented" --mode now

## ADDENDUM: CLI Parity Check
After deepening providers and agent, also verify the CLI has ALL of these OpenClaw commands as subcommands (add any missing):
status, setup/configure, onboard, health/doctor, dashboard, tui, completion, version, update, reset/uninstall, agent, agents, models, gateway, daemon, channels, message, directory, pairing, sessions, memory, cron, logs, hooks/webhooks, config, security, secrets, sandbox, approvals, plugins, skills, dns, devices, nodes, qr, browser, acp, docs, system, clawbot

Each command should have the same subcommands as OpenClaw (e.g., `models auth add/login/paste-token/setup-token/order`, `cron list/add/edit/rm/run/enable/disable/runs`, etc.)

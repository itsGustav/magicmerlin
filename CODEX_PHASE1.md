# Phase 1: Agent Runtime — Build Instructions

You are building Magic Merlin, a 100% Rust replacement for OpenClaw. Phase 0 (foundation) is complete with 4 crates: config, logging, infra, storage. Now build the Agent Runtime — the brain.

## Existing Crates Available
- `magicmerlin-config`: Config loading, get/set/unset, env overlay, profiles, secrets
- `magicmerlin-logging`: Structured tracing, file+console sinks, rotation
- `magicmerlin-infra`: HTTP client (reqwest), time/text/markdown utils
- `magicmerlin-storage`: SQLite DB, JSONL transcripts, file locks, memory files

## What To Build (Phase 1) — 3 New Crates

### 1.1 Provider Routing (new crate: `providers`)
Multi-provider LLM client that routes requests to the right API:

1. **Provider trait**: Define a `LlmProvider` trait with:
   - `async fn complete(request: CompletionRequest) -> Result<CompletionResponse>`
   - `async fn complete_stream(request: CompletionRequest) -> Result<impl Stream<Item=StreamChunk>>`
   - `fn name() -> &str`
   - `fn supports_model(model_id: &str) -> bool`

2. **Provider implementations** (each in its own module):
   - `openai` — OpenAI API (GPT-5.x, o-series). Supports: chat completions, tool calls, streaming, vision
   - `anthropic` — Anthropic API (Claude). Supports: messages API, tool use, streaming, vision, prompt caching
   - `google` — Google AI (Gemini). Supports: generateContent, tool calls, streaming
   - `xai` — xAI API (Grok). OpenAI-compatible format
   - `groq` — Groq API. OpenAI-compatible format
   - `mistral` — Mistral API. OpenAI-compatible format  
   - `minimax` — MiniMax API. OpenAI-compatible format
   - `moonshot` — Moonshot/Kimi API. OpenAI-compatible format
   - `deepseek` — DeepSeek API. OpenAI-compatible format
   - `local` — Local/Ollama. OpenAI-compatible at configurable base URL
   
   For OpenAI-compatible providers (xai, groq, mistral, minimax, moonshot, deepseek, local), create a shared `openai_compat` module that all of them use with different base URLs.

3. **Auth system**:
   - API key auth (header: `Authorization: Bearer <key>` or `x-api-key: <key>` for Anthropic)
   - OAuth token refresh (for openai-codex provider: token refresh flow)
   - Auth profiles loading from `auth-profiles.json`
   - API key rotation (multiple keys per provider, round-robin on 429)

4. **Model registry**:
   - Load model definitions from config (`models.providers.<name>.models[]`)
   - Model alias resolution: `gpt` → `openai/gpt-5.2`, `sonnet` → `anthropic/claude-sonnet-4-6`, `opus` → `anthropic/claude-opus-4-6`
   - Model metadata: context window, max tokens, cost per token, capabilities (vision, tools, streaming)
   - Provider/model ID parsing: `provider/model-id` format

5. **Failover chain**:
   - Primary model → fallback1 → fallback2
   - Automatic failover on: 401 (auth), 429 (rate limit), 500+ (server error), timeout
   - Configurable retry with exponential backoff (max 3 retries per provider)
   - Rate limit tracking per provider (respect Retry-After headers)

6. **Request/Response types**:
   ```rust
   struct CompletionRequest {
       model: String,
       messages: Vec<Message>,
       tools: Option<Vec<ToolDefinition>>,
       temperature: Option<f64>,
       max_tokens: Option<u32>,
       stream: bool,
       // provider-specific extensions
       extra: HashMap<String, Value>,
   }
   
   struct Message {
       role: Role, // System, User, Assistant, Tool
       content: MessageContent, // Text, MultiPart (text+images), ToolUse, ToolResult
   }
   
   struct CompletionResponse {
       id: String,
       model: String,
       content: Vec<ContentBlock>,
       tool_calls: Vec<ToolCall>,
       usage: Usage, // input_tokens, output_tokens, cache_read, cache_write
       stop_reason: StopReason,
   }
   ```

7. **Token counting**: Approximate token counter (chars/4 heuristic + tiktoken for OpenAI if available)
8. **Cost tracking**: Calculate cost per request based on model cost config

Dependencies: `magicmerlin-config`, `magicmerlin-infra`, `reqwest`, `serde`, `serde_json`, `tokio`, `tokio-stream`, `futures`

### 1.2 Agent Engine (new crate: `agent`)
The core agent turn loop:

1. **System prompt assembly**:
   - Load workspace files (AGENTS.md, SOUL.md, USER.md, IDENTITY.md, TOOLS.md, MEMORY.md, HEARTBEAT.md)
   - Inject into system prompt with character limits and truncation
   - Skills discovery: scan skill directories, inject `<available_skills>` block
   - Runtime metadata injection (date, time, timezone, model, channel info)

2. **Agent turn loop**:
   ```
   receive message → build messages array → call LLM → 
   if tool_calls: execute tools → append results → call LLM again →
   repeat until no more tool_calls → return final response
   ```

3. **Session management**:
   - Session key resolution (`agent:<name>:main`, `telegram:<chat_id>`)
   - Create/load/save sessions via storage crate
   - Token tracking per session
   - Context window management: when approaching limit, trigger compaction
   - Pre-compaction memory flush (write important context to memory files)

4. **Agent isolation**:
   - Each agent has: workspace dir, agent dir, sessions dir, config overrides
   - Agent config: model, fallbacks, identity emoji, heartbeat settings
   - Multi-agent registry: load all agents from `~/.openclaw/agents/*/`

5. **Message queue**:
   - Collect mode: batch incoming messages before starting agent turn
   - Debounce: wait for pause in messages before processing
   - Abort: cancel in-progress turn if new high-priority message arrives

6. **Heartbeat**:
   - Load HEARTBEAT.md
   - If empty/comments only → respond HEARTBEAT_OK
   - Otherwise execute tasks listed in HEARTBEAT.md

Dependencies: `magicmerlin-providers`, `magicmerlin-config`, `magicmerlin-storage`, `magicmerlin-infra`

### 1.3 Tool Execution Engine (new crate: `agent-tools`)
All 50+ tools that agents can invoke:

1. **Tool trait**:
   ```rust
   trait Tool: Send + Sync {
       fn name(&self) -> &str;
       fn description(&self) -> &str;
       fn schema(&self) -> ToolSchema; // JSON Schema for parameters
       async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
   }
   ```

2. **Core tools** (implement these):
   - `exec` — Shell command execution with:
     - PTY support (via `portable-pty` crate)
     - Background mode (spawn, return session ID)
     - Timeout handling
     - Working directory
     - Environment variables
   - `process` — Background process management:
     - list, poll, log (with offset/limit), write, submit, send-keys, paste, kill
   - `read` — Read file contents (text + images), with offset/limit for large files
   - `write` — Write/create files, auto-create parent directories  
   - `edit` — Precise text replacement (oldText → newText matching)
   - `web_search` — Brave Search API (query, count, freshness, country, language)
   - `web_fetch` — Fetch URL, extract as markdown or text (readability algorithm)
   - `memory_search` — Semantic search over memory files (embedding-based)
   - `memory_get` — Read snippet from memory files with line range
   - `session_status` — Return session status card (model, tokens, cost, context %)
   - `sessions_list` — List sessions with filters
   - `sessions_history` — Fetch message history for a session
   - `sessions_send` — Send message to another session
   - `sessions_spawn` — Spawn sub-agent session
   - `subagents` — List/steer/kill sub-agents
   - `agents_list` — List available agent IDs
   - `message` — Send messages to channels (Telegram, Discord, etc.)
   - `image` — Analyze images with vision model
   - `pdf` — Analyze PDFs
   - `tts` — Text to speech
   - `browser` — Browser automation (snapshot, screenshot, navigate, act)
   - `canvas` — Canvas control
   - `nodes` — Remote node control

3. **Tool registry**:
   - Register all tools at startup
   - Generate JSON Schema for each tool's parameters
   - Tool permission system: deny lists, workspace-only FS restrictions
   - Tool result size limits and truncation

4. **Tool context**:
   ```rust
   struct ToolContext {
       agent_name: String,
       workspace_dir: PathBuf,
       state_paths: StatePaths,
       config: Config,
       // Channel delivery context for message tool
       delivery: Option<DeliveryContext>,
   }
   ```

Dependencies: `magicmerlin-config`, `magicmerlin-storage`, `magicmerlin-infra`, `magicmerlin-providers`, `portable-pty`, `serde`, `tokio`

## Quality Requirements
- Every public function has a doc comment
- Error handling uses `thiserror` for custom errors, `anyhow` for application
- No unwrap() in library code
- Unit tests for: provider request formatting, auth header generation, failover logic, tool execution, session management, system prompt assembly
- Integration tests: mock LLM server → agent turn → tool execution → response
- `cargo check`, `cargo test`, `cargo clippy -- -D warnings` must all pass

## Commit Strategy
- Commit after completing each sub-crate (providers, agent, agent-tools)
- Push after all three are done

When completely finished, run this command to notify me:
openclaw system event --text "Phase 1 complete: Agent Runtime (providers, agent engine, tools) built and tested" --mode now

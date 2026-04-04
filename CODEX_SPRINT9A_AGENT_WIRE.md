# Sprint 9A — Wire AgentEngine into gateway run_agent_turn

## Goal
Connect the real `AgentEngine` (in `agent/src/engine.rs`) into `run_agent_turn` in `gateway/src/main.rs`. 
Right now `run_agent_turn` is a stub that just echoes `"Auto reply: {message}"`. After this sprint, it 
must call the full agent loop, execute real tools, build system prompts from workspace files, and return 
the actual LLM reply.

## Working directory
`~/Projects/magicmerlin`

## Step 1 — Read the current code first
Read these files completely before writing any code:
- `gateway/src/main.rs` — focus on `run_agent_turn` (line ~4089), `AppState`, imports
- `agent/src/engine.rs` — `AgentEngine`, `AgentEngineConfig`, `run_turn_with_options`
- `agent/src/system_prompt.rs` — `SystemPromptAssembler`, `PromptRuntimeMetadata`  
- `agent-tools/src/tools.rs` — `register_default_tools`, `ToolContext`, `ToolRegistry`
- `agent-tools/src/registry.rs` (or wherever `ToolContext` is defined)
- `agent/src/session.rs` — `SessionManager`, `SessionRecord`
- `providers/src/router.rs` — `ProviderRouter`

## Step 2 — Add AgentEngine + ToolRegistry to AppState

In `gateway/src/main.rs`, add to `AppState`:
```rust
agent_engine: Arc<magicmerlin_agent::AgentEngine>,
tool_registry: Arc<magicmerlin_agent_tools::ToolRegistry>,
```

Build them in the initialization block (where `AppState` is constructed):
1. Create a `ProviderRouter` from the loaded config (map `config.providers` → `ProviderConfig`)
2. Create a `SessionManager` pointing at `db_path`
3. Build `AgentEngineConfig`:
   - `model`: from `config.agents.defaults.model` or `"gpt-4o"`
   - `workspace_dir`: `~/.magicmerlin/workspace` (or `state_dir/workspace`)  
   - `agent_dir`: `state_dir/agents/merlin`
   - `agent_name`: `"merlin"`
   - `channel`: `"gateway"`
   - `timezone`: `"UTC"`
   - `max_turns`: 20
   - `max_tool_rounds`: 10
   - `context_window`: 120_000
   - `token_budget`: 100_000
   - `compact_threshold_pct`: 0.75
4. Build `AgentEngine::new(router, session_manager, engine_config)`
5. Build `ToolRegistry` + call `register_default_tools(&mut registry)`
6. Build a `ToolContext` factory (or store enough in AppState to build one per request)

Add required workspace dependencies in `gateway/Cargo.toml`:
```toml
magicmerlin-agent = { path = "../agent" }
magicmerlin-agent-tools = { path = "../agent-tools" }
```
(They may already be there — check first.)

## Step 3 — Rewrite run_agent_turn

Replace the stub body. The function signature stays the same. New logic:

```rust
async fn run_agent_turn(state: &AppState, client_id: &str, params: Value) -> Result<Value, RpcError> {
    // 1. Parse params (session_id, message, timeout_seconds)
    // 2. If slash command → handle locally (keep existing slash command handling)
    // 3. Enqueue + wait_turn (keep existing run_queue logic)
    // 4. Load or create SessionRecord from state.sessions (get_or_create_session)
    // 5. Build ToolContext:
    //    - workspace_dir: state.agent_engine.config.workspace_dir
    //    - state_paths: StatePaths { state_dir: state.db_path.parent() }
    //    - process_manager: state.process_manager (or create a ProcessManager per-state)
    //    - gateway_url: format!("http://127.0.0.1:{}", state.port)
    //    - gateway_token: state.auth.token.clone()
    //    - delivery: None (no delivery context yet)
    //    - understanding_client: None
    // 6. Build tool_schemas from state.tool_registry.schema_list()
    // 7. Create AbortSignal wired to abort_rx
    // 8. Call state.agent_engine.run_turn_with_options(&mut session, &message, &tool_executor, &inbound_ctx, &tool_schemas, Some(&abort_signal))
    // 9. Save updated session back
    // 10. Return Ok(json!({ "ok": true, "reply": reply.text, "sessionId": session_id }))
}
```

Key types to get right:
- `ToolExecutor` trait — check how `ToolRegistry` implements it, or wrap `ToolContext` + `ToolRegistry` into a struct that implements `ToolExecutor`
- `AbortSignal` — check `agent/src/engine.rs` for its definition (likely a wrapper around a `watch::Receiver<bool>`)
- `SessionRecord` — needs to be loaded from SQLite sessions table or created fresh
- `InboundContext` — use `InboundContext::default()` for now

## Step 4 — ProcessManager in AppState

The `ToolContext` needs a `ProcessManager` for the `exec` tool. Check `agent-tools` for `ProcessManager`.
Add it to `AppState` and initialize it with `ProcessManager::new()`. Wire it into `ToolContext`.

## Step 5 — Fix compilation errors iteratively

Run `cargo check -p magicmerlin-gateway 2>&1 | head -50` after each major change.
Fix errors one by one. Do not introduce `unimplemented!()` or `todo!()` — use real implementations.

Common issues you'll hit:
- `Send + Sync` bounds on `AgentEngine` — wrap in `Arc<Mutex<...>>` if needed
- `SessionRecord` ownership — use `Arc<Mutex<SessionRecord>>` or clone for the turn
- Missing trait impls — check if `ToolRegistry` impls `ToolExecutor`, if not create a bridge struct

## Step 6 — Run the full build

```bash
cargo build --release 2>&1 | tail -40
```

Must compile clean with 0 errors. Warnings are OK.

## Step 7 — Smoke test

Start the gateway:
```bash
./target/release/magicmerlin-gateway --serve 19001 --bind 127.0.0.1 &
sleep 2
```

Send a real agent turn:
```bash
curl -s -X POST http://127.0.0.1:19001/call \
  -H "Content-Type: application/json" \
  -d '{"method":"agent.run","params":{"session_id":"test-1","message":"What is 2+2?","timeout_seconds":30}}'
```

The reply field must NOT be `"Auto reply: What is 2+2?"`. It must be an actual LLM response.

If the providers aren't configured yet (no API keys in the gateway config), it's OK to return a 
provider-unavailable error. The important thing is that the stub echo is GONE and the real pipeline runs.

Kill the test gateway when done: `pkill -f "magicmerlin-gateway.*19001"`

## Step 8 — Commit

```bash
git add -A
git commit -m "feat(gateway): wire AgentEngine into run_agent_turn — real agent loop replaces stub echo"
```

## When done

Run this to notify:
```bash
openclaw system event --text "Sprint 9A done: AgentEngine wired into gateway run_agent_turn — real LLM loop active" --mode now
```

## Notes
- Do NOT modify any test files unless they fail to compile
- Do NOT delete existing stub methods elsewhere — only replace `run_agent_turn`
- If `AgentEngine` isn't `Send+Sync`, wrap it: `Arc<tokio::sync::Mutex<AgentEngine>>`
- The workspace dir for the agent should default to `~/.magicmerlin/workspace` — create it if it doesn't exist
- If provider config parsing is complex, start with a hardcoded OpenAI provider using `OPENAI_API_KEY` env var as fallback

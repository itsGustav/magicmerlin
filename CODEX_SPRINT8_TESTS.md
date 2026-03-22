# Sprint 8 — Agent A: Integration Tests + CI

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Unit tests exist in most crates but no integration tests or CI pipeline.
This sprint adds the integration test harness and GitHub Actions CI.

---

## Part 1: Integration Test Harness

Create `tests/integration/` at workspace root:

```
tests/
  integration/
    mod.rs
    gateway_roundtrip.rs    — launch gateway, test WS + HTTP methods
    agent_turn.rs           — full agent turn end-to-end with mock provider
    session_lifecycle.rs    — create, fill, compact, resume session
    cron_execution.rs       — schedule 1s job, verify fires, verify delivery
    channel_roundtrip.rs    — mock channel inbound → agent turn → reply
    tool_execution.rs       — test each tool with mocked external calls
    cli_smoke.rs            — spawn CLI binary, verify commands output
```

### `gateway_roundtrip.rs`

```rust
// Spawn gateway in-process or as subprocess, run tests, shut down

#[tokio::test]
async fn test_gateway_health() {
    let gateway = TestGateway::start().await;
    let resp = gateway.http_get("/health").await;
    assert_eq!(resp["ok"], true);
    gateway.stop().await;
}

#[tokio::test]
async fn test_call_gateway_status() {
    let gateway = TestGateway::start().await;
    let resp = gateway.call("gateway.status", json!({})).await;
    assert!(resp.get("version").is_some());
    gateway.stop().await;
}

#[tokio::test]
async fn test_cron_list_empty() {
    let gateway = TestGateway::start().await;
    let resp = gateway.call("cron.list", json!({})).await;
    assert!(resp["result"].as_array().is_some());
    gateway.stop().await;
}

pub struct TestGateway {
    port: u16,
    handle: tokio::task::JoinHandle<()>,
}

impl TestGateway {
    pub async fn start() -> Self {
        let port = find_free_port();
        // Spawn gateway on random port with temp state dir
        let handle = tokio::spawn(async move {
            run_gateway(port, temp_state_dir()).await;
        });
        tokio::time::sleep(Duration::from_millis(100)).await; // let it start
        Self { port, handle }
    }
    
    pub async fn call(&self, method: &str, params: Value) -> Value {
        reqwest::Client::new()
            .post(format!("http://127.0.0.1:{}/call", self.port))
            .json(&json!({"method": method, "params": params}))
            .send().await.unwrap()
            .json::<Value>().await.unwrap()
    }
    
    pub async fn http_get(&self, path: &str) -> Value {
        reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}{}", self.port, path))
            .send().await.unwrap()
            .json::<Value>().await.unwrap()
    }
    
    pub async fn stop(self) {
        self.handle.abort();
    }
}
```

### `agent_turn.rs` — Mock provider agent turn

```rust
// Mock LLM provider that returns deterministic responses
pub struct MockProvider {
    responses: VecDeque<String>,
}

#[tokio::test]
async fn test_agent_turn_basic() {
    let mut engine = AgentEngine::new_with_mock_provider(
        MockProvider::new(vec!["Hello! I'm Magic Merlin.".into()])
    );
    
    let result = engine.run_turn("What is your name?", "test-session").await.unwrap();
    assert!(result.reply.contains("Magic Merlin") || result.reply.len() > 0);
    assert_eq!(result.session_key, "test-session");
}

#[tokio::test]
async fn test_agent_turn_with_tool_call() {
    // Mock provider that first returns a tool call, then a final reply
    let responses = vec![
        json!({"tool_calls": [{"name": "exec", "params": {"cmd": "echo hello"}}]}).to_string(),
        "The command output was: hello".into(),
    ];
    
    let mut engine = AgentEngine::new_with_mock_provider(MockProvider::new(responses));
    let result = engine.run_turn("Run echo hello", "test-session").await.unwrap();
    assert!(result.tool_calls.len() > 0 || result.reply.len() > 0);
}
```

### `session_lifecycle.rs`

```rust
#[tokio::test]
async fn test_session_compaction() {
    let state_dir = tempdir().unwrap();
    let session_key = "test-compaction";
    
    // Create a session with many messages
    let mut transcript = Vec::new();
    for i in 0..50 {
        transcript.push(json!({"role": "user", "content": format!("Message {}", i), "ts": i}));
        transcript.push(json!({"role": "assistant", "content": format!("Reply {}", i), "ts": i}));
    }
    write_session_jsonl(session_key, &transcript, state_dir.path()).unwrap();
    
    // Compact
    let result = compact_session(session_key, state_dir.path()).await.unwrap();
    assert!(result.messages_before > result.messages_after);
    assert!(result.messages_after < 20); // should be much smaller
    
    // Verify compacted session is readable
    let compacted = read_session_jsonl(session_key, state_dir.path()).unwrap();
    assert!(compacted.len() > 0);
}

#[tokio::test]
async fn test_context_percent() {
    let state_dir = tempdir().unwrap();
    // Create session with known size
    let msg = "a".repeat(4000); // ~1000 tokens
    let transcript = vec![json!({"role": "user", "content": msg})];
    write_session_jsonl("ctx-test", &transcript, state_dir.path()).unwrap();
    
    let pct = estimate_context_percent("ctx-test", "gpt-4o", state_dir.path()).unwrap();
    assert!(pct > 0.0 && pct < 0.1); // 1000 tokens / 128000 limit ≈ 0.78%
}
```

### `cron_execution.rs`

```rust
#[tokio::test]
async fn test_cron_job_fires() {
    let gateway = TestGateway::start().await;
    
    // Add a 1s cron job
    let resp = gateway.call("cron.add", json!({
        "job": {
            "name": "test-job",
            "schedule": {"kind": "every", "everyMs": 1000},
            "payload": {"kind": "systemEvent", "text": "test fired"},
            "sessionTarget": "main",
            "enabled": true
        }
    })).await;
    
    let job_id = resp["result"]["id"].as_str().unwrap().to_string();
    
    // Wait 2.5s for it to fire at least twice
    tokio::time::sleep(Duration::from_millis(2500)).await;
    
    // Check run history
    let runs = gateway.call("cron.runs", json!({"jobId": job_id})).await;
    let run_count = runs["result"].as_array().unwrap().len();
    assert!(run_count >= 2, "Expected ≥2 runs, got {run_count}");
    
    gateway.stop().await;
}
```

### `cli_smoke.rs`

```rust
#[test]
fn test_cli_version() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_magicmerlin"))
        .arg("version")
        .output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("magicmerlin"));
}

#[test]
fn test_cli_completions_zsh() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_magicmerlin"))
        .args(["completions", "zsh"])
        .output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("compdef") || stdout.contains("magicmerlin"));
}

#[test]
fn test_cli_ping_offline() {
    // When gateway is offline, ping should fail gracefully (not panic)
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_magicmerlin"))
        .arg("ping")
        .output().unwrap();
    // Should exit with non-zero but not crash
    let stderr = String::from_utf8(out.stderr).unwrap();
    // Should mention gateway offline
    assert!(stderr.contains("offline") || stderr.contains("refused") || !out.status.success());
}
```

---

## Part 2: GitHub Actions CI Pipeline

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, "feat/**"]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Build
        run: cargo build --workspace --locked
      
      - name: Unit tests
        run: cargo test --workspace --lib
      
      - name: Integration tests
        run: cargo test --workspace --test '*'
      
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
      
      - name: Fmt check
        run: cargo fmt --all -- --check

  cross-compile:
    name: Cross-compile (${{ matrix.target }})
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - x86_64-unknown-linux-gnu
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Install cross
        run: cargo install cross --git https://github.com/cross-rs/cross
      - name: Cross build
        run: cross build --workspace --target ${{ matrix.target }} --release

  release:
    name: Release binaries
    if: startsWith(github.ref, 'refs/tags/v')
    needs: [test]
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            suffix: linux-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            suffix: macos-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            suffix: macos-x86_64
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Build release
        run: cargo build --workspace --release --target ${{ matrix.target }}
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: magicmerlin-${{ matrix.suffix }}
          path: target/${{ matrix.target }}/release/magicmerlin
```

---

## Part 3: Fix All Clippy Warnings

Run `cargo clippy --workspace 2>&1` and fix every warning:
- Remove unused variables/imports
- Replace `unwrap()` calls in non-test code with `?` or proper error handling
- Fix `dead_code` warnings by adding `#[allow(dead_code)]` only when intentional
- Fix `clippy::redundant_clone`, `clippy::needless_pass_by_value`, etc.

Target: `cargo clippy --workspace -- -D warnings` passes with 0 errors.

---

## Rules
- `cargo test --workspace` must pass with ≥ 80 tests
- CI yaml must be syntactically valid
- Integration tests must be deterministic (no flaky timing-dependent assertions except the cron test which has 2.5s buffer)
- All clippy warnings fixed

## Completion
```bash
openclaw system event --text "Sprint 8A done: integration test harness (gateway roundtrip, agent turn, session lifecycle, cron execution, CLI smoke), GitHub Actions CI, clippy clean" --mode now
```

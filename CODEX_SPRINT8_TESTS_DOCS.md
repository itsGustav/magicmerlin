# Sprint 8 — Tests, CI, and Docs

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Goal: integration test suite, GitHub Actions CI, and 332-page docs site.

---

## Part 1: Integration Test Suite

### 1A: Gateway round-trip tests

Create `gateway/tests/integration.rs`:

```rust
// Spin up a real gateway on a random port, run HTTP calls, verify responses

#[tokio::test]
async fn test_gateway_health() {
    let port = find_free_port();
    let _server = start_test_gateway(port).await;
    let client = reqwest::Client::new();
    let resp = client.get(format!("http://127.0.0.1:{port}/health"))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_gateway_call_status() {
    let port = find_free_port();
    let _server = start_test_gateway(port).await;
    let resp = call_gateway(port, "gateway.status", json!({})).await;
    assert!(resp["version"].is_string());
}

#[tokio::test]
async fn test_cron_add_list_remove() {
    let port = find_free_port();
    let _server = start_test_gateway(port).await;
    
    // Add a job
    let add = call_gateway(port, "cron.add", json!({
        "job": {
            "name": "test-job",
            "schedule": {"kind": "every", "everyMs": 60000},
            "payload": {"kind": "systemEvent", "text": "test"},
            "sessionTarget": "main"
        }
    })).await;
    let job_id = add["jobId"].as_str().unwrap();
    
    // List — should include our job
    let list = call_gateway(port, "cron.list", json!({})).await;
    let jobs = list["jobs"].as_array().unwrap();
    assert!(jobs.iter().any(|j| j["jobId"] == job_id));
    
    // Remove
    let rm = call_gateway(port, "cron.remove", json!({"jobId": job_id})).await;
    assert_eq!(rm["ok"], true);
}

async fn start_test_gateway(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Start gateway with temp DB
        let db = tempfile::NamedTempFile::new().unwrap();
        // ... start server
    })
}

async fn call_gateway(port: u16, method: &str, params: Value) -> Value {
    reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/call"))
        .json(&json!({"method": method, "params": params}))
        .send().await.unwrap()
        .json().await.unwrap()
}
```

### 1B: Tool execution tests

Create `agent-tools/tests/tool_integration.rs`:

```rust
// Test each tool with real (or mocked) backends

#[tokio::test]
async fn test_exec_tool_basic() {
    let ctx = test_tool_context();
    let result = exec_tool(json!({"cmd": "echo hello"}), &ctx).await;
    assert!(result["stdout"].as_str().unwrap().contains("hello"));
}

#[tokio::test]
async fn test_read_tool() {
    let ctx = test_tool_context();
    // Write a temp file, read it back
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.md");
    std::fs::write(&path, "# Hello\nWorld").unwrap();
    let result = read_tool(json!({"path": path.to_str().unwrap()}), &ctx).await;
    assert!(result["content"].as_str().unwrap().contains("Hello"));
}

#[tokio::test]
async fn test_memory_search_basic() {
    let ctx = test_tool_context_with_memory("# Test\nsome important fact about quantum computing");
    let result = memory_search_tool(json!({"query": "quantum computing"}), &ctx).await;
    let results = result["results"].as_array().unwrap();
    assert!(!results.is_empty());
    assert!(results[0]["score"].as_f64().unwrap() > 0.1);
}

#[tokio::test]
async fn test_web_fetch_html_extraction() {
    // Mock server returning HTML
    let html = r#"<html><nav>skip this</nav><main><h1>Title</h1><p>Content here</p></main></html>"#;
    // ... mock server setup
    let result = web_fetch_tool(json!({"url": mock_url, "extractMode": "markdown"}), &ctx).await;
    assert!(result["content"].as_str().unwrap().contains("Title"));
    assert!(!result["content"].as_str().unwrap().contains("skip this"));
}
```

### 1C: Channel tests

Create `channels/tests/`:

```rust
// Telegram message normalization
#[test]
fn test_telegram_format_markdown() {
    let input = "Hello **world** and `code`";
    let output = format_for_telegram(input);
    assert!(output.contains("*world*"));
    assert!(output.contains("`code`"));
}

// Signal message normalization
#[test]
fn test_signal_envelope_normalization() {
    let envelope = SignalEnvelope { /* test data */ };
    let msg = envelope.into_inbound().unwrap();
    assert_eq!(msg.platform, Platform::Signal);
}

// Auto-reply slash command parsing
#[test]
fn test_slash_command_parsing() {
    assert!(matches!(parse_slash_command("/status"), Some(SlashCommand::Status)));
    assert!(matches!(parse_slash_command("/model sonnet"), Some(SlashCommand::Model { .. })));
    assert!(matches!(parse_slash_command("/approve abc123 allow-always"), Some(SlashCommand::Approve { .. })));
    assert!(parse_slash_command("hello world"), None);
}

// Session compaction
#[tokio::test]
async fn test_session_compaction() {
    let dir = tempdir().unwrap();
    // Write 50 messages to session JSONL
    // Compact
    // Verify message count reduced
    // Verify last N messages preserved exactly
}
```

### 1D: Parity sentinel CI check

Update `sentinel/src/main.rs` to add a `--ci` mode:

```rust
// cargo run -p magicmerlin-sentinel -- methods-diff --ci
// Exit code 0 if diffs within acceptable threshold
// Exit code 1 if critical regressions (methods that existed before are now missing)
// Print summary: "Gateway: 108/108 (100%), CLI: 193/257 (75%), Docs: 45/332 (14%)"
```

---

## Part 2: GitHub Actions CI

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  build:
    name: Build & Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
          components: clippy, rustfmt
      
      - name: Cache
        uses: Swatinem/rust-cache@v2
      
      - name: Build
        run: cargo build --workspace --all-features
      
      - name: Test
        run: cargo test --workspace --all-features
      
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
      
      - name: Format check
        run: cargo fmt --all -- --check

  release:
    name: Release Build
    runs-on: ${{ matrix.os }}
    if: github.ref == 'refs/heads/main'
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Build release
        run: cargo build --release --target ${{ matrix.target }} -p magicmerlin -p magicmerlin-gateway
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: magicmerlin-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/magicmerlin
            target/${{ matrix.target }}/release/magicmerlin-gateway
```

---

## Part 3: Documentation Site (332 pages)

### Strategy: Auto-generate from templates + code

Create a `docs-builder` tool (Rust binary) in `tools/src/docs_builder.rs`:

```rust
// Reads: parity/openclaw_docs_index.json (332 page index)
// For each page:
//   1. Check if already written in docs/
//   2. If not: generate from template based on page category
//   3. Write to docs/{section}/{slug}.md

// Page categories and templates:
// install/* → installation instructions template
// cli/* → auto-generate from clap command docs
// gateway/* → auto-generate from SUPPORTED_METHODS
// tools/* → auto-generate from tool registry schemas
// channels/* → channel-specific setup template
// concepts/* → conceptual explanation template
// providers/* → provider setup template
// reference/* → reference template
```

### Auto-generated pages (bulk — ~280 pages)

For CLI pages: extract from clap metadata:
```rust
fn generate_cli_page(command: &str, description: &str, args: &[Arg]) -> String {
    format!("# `magicmerlin {command}`\n\n{description}\n\n## Usage\n```\nmagicmerlin {command} [OPTIONS]\n```\n\n## Options\n\n{args_table}\n")
}
```

For gateway method pages:
```rust
fn generate_method_page(method: &str, desc: &str, params: &Value, returns: &Value) -> String {
    format!("# `{method}`\n\n{desc}\n\n## Parameters\n\n{params_table}\n\n## Returns\n\n{returns_table}\n")
}
```

For tool pages:
```rust
fn generate_tool_page(name: &str, description: &str, schema: &Value) -> String {
    // Extract params from JSON schema
    format!("# `{name}` tool\n\n{description}\n\n## Parameters\n\n{params_table}\n")
}
```

### Manual pages (seed content — ~52 pages)

For `start/`, `concepts/`, `platforms/` — write real content:

Key pages to write manually:
- `docs/start/getting-started.md` — quickstart guide
- `docs/start/installation.md` — cargo install, brew, docker
- `docs/start/migrate-from-openclaw.md` — migration guide
- `docs/concepts/agents.md` — what are agents
- `docs/concepts/sessions.md` — session model
- `docs/concepts/tools.md` — tool system
- `docs/concepts/skills.md` — skill system
- `docs/concepts/memory.md` — memory model

### MkDocs config

Create `docs/mkdocs.yml`:
```yaml
site_name: Magic Merlin
site_description: Rust-first OpenClaw-compatible AI agent runtime
theme:
  name: material
  palette:
    scheme: slate
    primary: deep purple
    accent: purple
  features:
    - navigation.tabs
    - navigation.instant
    - search.highlight
nav:
  - Getting Started:
    - start/getting-started.md
    - start/installation.md
    - start/migrate-from-openclaw.md
  - CLI Reference: cli/
  - Gateway Methods: gateway/
  - Tools: tools/
  - Channels: channels/
  - Providers: providers/
  - Concepts: concepts/
```

### Run the docs builder

After implementing `tools/src/docs_builder.rs`:
```bash
cargo run -p magicmerlin-tools -- docs-build --output ./docs
```

Should generate all 332 pages. Verify count.

---

## Rules
- `cargo test --workspace` must pass with at least 60 tests total
- CI workflow must be syntactically valid YAML
- Docs: must generate ≥300 of the 332 pages (remainder can be stubs with `# TODO`)
- `cargo build --workspace` clean throughout

## Completion
```bash
openclaw system event --text "Sprint 8 done: 60+ integration tests passing, GitHub Actions CI (4-platform matrix), 300+ docs pages auto-generated, parity sentinel CI mode" --mode now
```

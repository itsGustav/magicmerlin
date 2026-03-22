# Sprint 8 — Agent B: Documentation Generation

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Docs parity: 332 pages needed (0 currently done).
`parity/openclaw_docs_index.json` has the full page list.
`parity/openclaw_llms_2026-03-03.txt` has OpenClaw's full docs content.

## Strategy
Auto-generate the docs site using a build-time Rust tool that:
1. Reads OpenClaw's docs structure from `parity/openclaw_docs_index.json`
2. Generates each page as Markdown adapted for Magic Merlin
3. Builds a `docs/` directory tree with `mkdocs.yml`

---

## Step 1: Read existing parity data

```bash
cat parity/openclaw_docs_index.json | head -50
cat parity/openclaw_llms_2026-03-03.txt | head -100
```

Use these to understand the full page list and content.

---

## Step 2: Create `tools/docgen/` crate

```
tools/docgen/
  Cargo.toml
  src/
    main.rs      — CLI: cargo run -p docgen -- --out docs/
    generator.rs — page generation logic
    templates.rs — page templates per section type
```

### `Cargo.toml`
```toml
[package]
name = "magicmerlin-docgen"
version = "0.0.0"
edition = "2021"

[[bin]]
name = "docgen"
path = "src/main.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
chrono = "0.4"
clap = { version = "4", features = ["derive"] }
```

### `main.rs`
```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "docs")]
    out: PathBuf,
    
    #[arg(long, default_value = "parity/openclaw_docs_index.json")]
    index: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    generator::generate_all(&args.index, &args.out)?;
    println!("Generated docs to {}", args.out.display());
    Ok(())
}
```

### `generator.rs`
```rust
pub fn generate_all(index_path: &Path, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;
    
    let index: Vec<DocPage> = serde_json::from_str(&std::fs::read_to_string(index_path)?)?;
    
    for page in &index {
        let content = generate_page(page);
        let out_path = out_dir.join(&page.path);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out_path, content)?;
    }
    
    // Generate mkdocs.yml
    let nav = build_nav(&index);
    std::fs::write(out_dir.join("mkdocs.yml"), generate_mkdocs_config(&nav))?;
    
    Ok(())
}

fn generate_page(page: &DocPage) -> String {
    // Use section-specific template
    let template = match page.section.as_str() {
        "cli" => generate_cli_page(page),
        "gateway" => generate_gateway_page(page),
        "tools" => generate_tools_page(page),
        "channels" => generate_channels_page(page),
        "providers" => generate_providers_page(page),
        "install" => generate_install_page(page),
        _ => generate_generic_page(page),
    };
    template
}
```

---

## Step 3: Page Templates

### CLI command pages (46 pages)
Auto-generate from the CLI binary's help output:
```rust
fn generate_cli_page(page: &DocPage) -> String {
    // Run: magicmerlin help {command} → capture output
    // Or: use clap's built-in help generation
    format!(r#"# {}

{}

## Usage

```
{}
```

## Options

{}

## Examples

{}
"#, page.title, page.description, page.usage, page.options, page.examples)
}
```

### Gateway method pages (33 pages)
```rust
fn generate_gateway_page(page: &DocPage) -> String {
    format!(r#"# {}

{}

## Method

`POST /call` with `{{"method": "{}", "params": {{...}}}}`

## Parameters

{}

## Returns

{}

## Example

```json
{{
  "method": "{}",
  "params": {}
}}
```
"#, ...)
}
```

### Tools pages (28 pages)
For each agent tool, document the tool schema:
```rust
fn generate_tools_page(page: &DocPage) -> String {
    // Read the ToolRegistry schema for this tool
    // Generate parameter table
}
```

### Install pages (20 pages)
Template for: macOS, Linux, Docker, cargo, homebrew, etc.

### Concepts pages (27 pages)
Generic template: title + description + diagram + examples

---

## Step 4: Generate ALL 332 Pages

Run the docgen tool and verify:
```bash
cargo run -p docgen -- --out docs/
ls docs/ | wc -l   # should be 332+
```

Minimum coverage per section:
- `install/` → 20 pages (macOS, Linux, Docker, systemd, launchagent, config, migration...)
- `cli/` → 43+ pages (one per command)
- `gateway/` → 33 pages (one per method group)
- `tools/` → 28 pages (one per tool)
- `channels/` → 29 pages (one per channel + config)
- `providers/` → 29 pages (one per LLM provider)
- `concepts/` → 27 pages (sessions, compaction, heartbeat, skills, plugins...)
- `start/` → 13 pages (quickstart, first run, onboarding...)
- `platforms/` → 27 pages (Telegram, Discord, Signal, etc.)
- `reference/` → 20 pages (config schema, env vars, etc.)

---

## Step 5: `mkdocs.yml` + `docs/index.md`

```yaml
# docs/mkdocs.yml
site_name: Magic Merlin Documentation
site_description: The Rust-native OpenClaw-compatible AI agent runtime
theme:
  name: material
  palette:
    scheme: slate
    primary: indigo
  features:
    - navigation.tabs
    - navigation.sections
    - search.suggest

nav:
  - Home: index.md
  - Getting Started:
    - Quickstart: start/getting-started.md
    - Installation: install/macos.md
    ...
  - CLI Reference:
    - Commands: cli/index.md
    ...
  - Gateway:
    - Methods: gateway/index.md
    ...
  - Tools:
    - Overview: tools/index.md
    ...
```

```markdown
<!-- docs/index.md -->
# Magic Merlin

Magic Merlin is a Rust-first, OpenClaw-compatible AI agent runtime.

## Features

- **Drop-in compatible** with OpenClaw config format and CLI
- **Rust-native** — single binary, no Node.js runtime required
- **All channels** — Telegram, Discord, Signal, WhatsApp, Slack, iMessage, LINE, Web
- **Full tool suite** — 23+ tools: exec, browser, memory, cron, sessions, nodes, TTS, PDF
- **Gateway API** — 108+ methods via WebSocket + HTTP
- **TUI dashboard** — `magicmerlin tui`

## Quick Start

```bash
cargo install --git https://github.com/itsGustav/magicmerlin magicmerlin magicmerlin-gateway
magicmerlin gateway start
magicmerlin status
```

## Migration from OpenClaw

See [Migration Guide](install/migration.md) — your `openclaw.json` works as-is.
```

---

## Step 6: Update `parity/docs_coverage.json`

After generation, update the coverage tracking:
```json
{
  "generatedAt": "2026-03-22T...",
  "totalPages": 332,
  "generated": 332,
  "sections": {
    "cli/": { "done": 43, "partial": 0, "todo": 0 },
    ...
  }
}
```

---

## Rules
- `cargo run -p docgen` must complete without errors
- Must generate ≥ 280 pages (acceptable if a few stubs remain)
- `docs/index.md` must exist and be readable
- `docs/mkdocs.yml` must be valid YAML
- Update `parity/docs_coverage_summary.json`

## Completion
```bash
openclaw system event --text "Sprint 8B done: docgen tool generates 332 pages, mkdocs.yml nav, CLI/gateway/tools/channels/providers/install/concepts sections all covered" --mode now
```

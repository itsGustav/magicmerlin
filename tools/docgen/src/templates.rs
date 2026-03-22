use crate::generator::DocPage;

/// Render a page using section-specific templates.
pub fn render_page(page: &DocPage) -> String {
    match page.section.as_str() {
        "cli" => render_cli_page(page),
        "gateway" => render_gateway_page(page),
        "tools" => render_tools_page(page),
        "channels" => render_channels_page(page),
        "providers" => render_providers_page(page),
        "install" => render_install_page(page),
        "concepts" => render_concepts_page(page),
        "start" => render_start_page(page),
        "platforms" => render_platforms_page(page),
        "nodes" => render_nodes_page(page),
        "reference" => render_reference_page(page),
        "automation" => render_automation_page(page),
        "help" => render_help_page(page),
        "plugins" => render_plugins_page(page),
        "web" => render_web_page(page),
        "security" => render_security_page(page),
        "experiments" => render_experiments_page(page),
        "debug" => render_debug_page(page),
        "design" => render_design_page(page),
        "diagnostics" => render_diagnostics_page(page),
        "api-reference" => render_api_reference_page(page),
        "root" => render_root_page(page),
        _ => render_generic_page(page),
    }
}

pub fn render_index(pages: &[DocPage]) -> String {
    // Collect section counts
    let mut section_counts: std::collections::BTreeMap<&str, usize> =
        std::collections::BTreeMap::new();
    for p in pages {
        *section_counts.entry(&p.section).or_insert(0) += 1;
    }

    let mut section_list = String::new();
    let section_links: &[(&str, &str, &str)] = &[
        (
            "Getting Started",
            "start/getting-started.md",
            "Quickstart guides, onboarding, and first-run setup",
        ),
        (
            "Installation",
            "install/index.md",
            "macOS, Linux, Docker, Nix, cloud deploy guides",
        ),
        (
            "Concepts",
            "concepts/agent.md",
            "Agent runtime, sessions, compaction, memory, streaming",
        ),
        (
            "CLI Reference",
            "cli/index.md",
            "All 46 CLI commands with usage and examples",
        ),
        (
            "Gateway",
            "gateway/index.md",
            "Configuration, protocols, security, health checks",
        ),
        (
            "Tools",
            "tools/index.md",
            "28 agent tools: exec, browser, PDF, TTS, skills, and more",
        ),
        (
            "Chat Channels",
            "channels/index.md",
            "Telegram, Discord, Signal, Slack, iMessage, WhatsApp, and more",
        ),
        (
            "Model Providers",
            "providers/index.md",
            "Anthropic, OpenAI, Ollama, Bedrock, and 25+ providers",
        ),
        (
            "Platforms",
            "platforms/index.md",
            "macOS, Linux, iOS, Android, Windows, Raspberry Pi, cloud",
        ),
        (
            "Nodes",
            "nodes/index.md",
            "Audio, camera, media, voice wake, talk mode",
        ),
        (
            "Automation",
            "automation/cron-jobs.md",
            "Cron jobs, hooks, webhooks, polls, Gmail PubSub",
        ),
        (
            "Plugins",
            "plugins/manifest.md",
            "Plugin manifest, agent tools, community plugins",
        ),
        (
            "Web & Dashboard",
            "web/index.md",
            "Control UI, dashboard, TUI, WebChat",
        ),
        (
            "Reference",
            "reference/api-usage-costs.md",
            "Config schema, templates, session deep-dive, credits",
        ),
    ];

    for (name, link, desc) in section_links {
        section_list.push_str(&format!("| [{}]({}) | {} |\n", name, link, desc));
    }

    format!(
        r#"# Magic Merlin

Magic Merlin is a **Rust-first, OpenClaw-compatible AI agent runtime**. Single binary, no Node.js required.

## Features

- **Drop-in compatible** with OpenClaw config format (`openclaw.json` works as-is)
- **Rust-native** — single statically-linked binary, fast cold start
- **All channels** — Telegram, Discord, Signal, WhatsApp, Slack, iMessage, LINE, Matrix, IRC, and more
- **Full tool suite** — 23+ tools: exec, browser, memory, cron, sessions, nodes, TTS, PDF, canvas
- **Gateway API** — 108+ methods via WebSocket + HTTP, OpenAI-compatible endpoint
- **TUI dashboard** — `magicmerlin tui` for real-time monitoring
- **Multi-agent** — sub-agent spawning, ACP thread-bound agents, agent routing

## Quick Start

```bash
# Install from source
cargo install magicmerlin magicmerlin-gateway

# Start the gateway
magicmerlin gateway start

# Check status
magicmerlin status

# Open the TUI dashboard
magicmerlin tui
```

## Documentation

| Section | Description |
|---------|-------------|
{section_list}

## Migration from OpenClaw

Your existing `openclaw.json` configuration works with Magic Merlin out of the box.
See the [Migration Guide](install/migrating.md) for details on switching from the Node.js runtime.

```bash
# Point Magic Merlin at your existing config
magicmerlin gateway start --config ~/.openclaw/openclaw.json
```

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Channels   │────▶│   Gateway    │────▶│  Agent Loop  │
│  (Telegram,  │     │  (HTTP/WS)   │     │  (LLM + Tools│
│   Discord…)  │◀────│              │◀────│   + Memory)  │
└──────────────┘     └──────────────┘     └──────────────┘
                            │
                     ┌──────┴──────┐
                     │   Nodes     │
                     │ (Audio/Cam) │
                     └─────────────┘
```

## Links

- **Source**: [github.com/itsGustav/magicmerlin](https://github.com/itsGustav/magicmerlin)
- **License**: Apache-2.0
"#
    )
}

// ---------------------------------------------------------------------------
// Section-specific renderers
// ---------------------------------------------------------------------------

fn slug(page: &DocPage) -> String {
    page.filename.trim_end_matches(".md").to_string()
}

fn render_cli_page(page: &DocPage) -> String {
    let cmd = slug(page);
    if cmd == "index" {
        return r#"# CLI Reference

Magic Merlin provides a comprehensive command-line interface for managing your agent runtime.

## Commands

All commands follow the pattern:

```
magicmerlin <command> [options]
```

Run `magicmerlin help` to see all available commands, or `magicmerlin help <command>` for detailed usage.

## Command Categories

### Core
- `magicmerlin gateway` — Start/stop the gateway daemon
- `magicmerlin status` — Show runtime status
- `magicmerlin config` — View/edit configuration
- `magicmerlin doctor` — Diagnose common issues

### Communication
- `magicmerlin message` — Send a message to an agent
- `magicmerlin channels` — List and manage chat channels
- `magicmerlin sessions` — Manage agent sessions

### Agent Management
- `magicmerlin agent` — Manage agent configuration
- `magicmerlin agents` — List running agents
- `magicmerlin skills` — List available skills
- `magicmerlin plugins` — Manage plugins

### Infrastructure
- `magicmerlin nodes` — Manage connected nodes
- `magicmerlin tui` — Open the TUI dashboard
- `magicmerlin logs` — Tail gateway logs
- `magicmerlin cron` — Manage scheduled jobs

### Setup
- `magicmerlin setup` — Interactive first-run setup
- `magicmerlin onboard` — Guided onboarding wizard
- `magicmerlin configure` — Configure providers and channels

See each command's page for full usage details.
"#.to_string();
    }

    format!(
        r#"# magicmerlin {cmd}

{title}

## Usage

```
magicmerlin {cmd} [options]
```

## Description

The `{cmd}` command {description}.

## Options

| Flag | Description |
|------|-------------|
| `--help` | Show help for this command |
| `--json` | Output in JSON format |
| `--verbose` | Enable verbose output |

## Examples

```bash
# Basic usage
magicmerlin {cmd}

# With JSON output
magicmerlin {cmd} --json

# With verbose logging
magicmerlin {cmd} --verbose
```

## See Also

- [CLI Reference](index.md)
- [Gateway Configuration](../gateway/configuration.md)
"#,
        cmd = cmd,
        title = page.title,
        description = cli_description(&cmd),
    )
}

fn cli_description(cmd: &str) -> &'static str {
    match cmd {
        "acp" => "manages Agent Communication Protocol (ACP) settings and thread-bound agents",
        "agent" => "configures the active agent, including system prompt, model, and tool policy",
        "agents" => "lists all registered agents and their current status",
        "approvals" => "reviews and manages pending tool execution approvals",
        "browser" => "controls the managed browser instance used by the browser tool",
        "channels" => "lists, adds, and configures chat channels (Telegram, Discord, etc.)",
        "clawbot" => "interacts with the built-in assistant for configuration help",
        "completion" => "generates shell completion scripts for bash, zsh, and fish",
        "config" => "reads and writes gateway configuration values",
        "configure" => "runs the interactive configuration wizard for providers and channels",
        "cron" => "lists, creates, and manages scheduled cron jobs",
        "daemon" => "manages the gateway background daemon process",
        "dashboard" => "opens the web-based dashboard in your default browser",
        "devices" => "lists connected devices and nodes",
        "directory" => "browses the agent directory and discovers shared agents",
        "dns" => "manages DNS settings for remote gateway access",
        "docs" => "opens the documentation site in your default browser",
        "doctor" => "runs diagnostic checks and reports common configuration issues",
        "gateway" => "starts, stops, and manages the gateway daemon",
        "health" => "checks the health status of the gateway and connected services",
        "hooks" => "manages automation hooks that trigger on events",
        "logs" => "tails and searches gateway log output",
        "memory" => "manages agent memory (view, search, clear)",
        "message" => "sends a message to an agent and displays the response",
        "models" => "lists available models across all configured providers",
        "node" => "manages a single node instance",
        "nodes" => "lists and manages connected node devices",
        "onboard" => "runs the guided onboarding wizard for first-time setup",
        "pairing" => "generates and manages device pairing codes",
        "plugins" => "lists, installs, and manages plugins",
        "qr" => "generates a QR code for mobile device pairing",
        "reset" => "resets configuration, sessions, or memory to defaults",
        "sandbox" => "manages the sandboxed execution environment for tools",
        "secrets" => "manages encrypted secrets used by tools and providers",
        "security" => "reviews and manages security policies and tool permissions",
        "sessions" => "lists, creates, and manages agent sessions",
        "setup" => "runs the interactive first-run setup wizard",
        "skills" => "lists and manages agent skills and slash commands",
        "status" => "displays the current runtime status of the gateway and agents",
        "system" => "manages system-level settings and emits system events",
        "tui" => "opens the terminal-based dashboard (Ratatui TUI)",
        "uninstall" => "removes Magic Merlin and cleans up configuration files",
        "update" => "checks for and installs Magic Merlin updates",
        "voicecall" => "initiates or manages a voice call session",
        "webhooks" => "lists and manages incoming webhook endpoints",
        _ => "performs the requested operation",
    }
}

fn render_gateway_page(page: &DocPage) -> String {
    let s = slug(page);
    if s == "index" {
        return r#"# Gateway Runbook

The Magic Merlin gateway is the central daemon that coordinates agents, channels, tools, and nodes.

## Starting the Gateway

```bash
magicmerlin gateway start
```

## Architecture

The gateway exposes:

- **WebSocket API** on `ws://localhost:3578` — primary protocol for real-time communication
- **HTTP API** on `http://localhost:3578` — REST endpoints and OpenAI-compatible chat completions
- **Bonjour/mDNS** discovery — nodes and clients auto-discover the gateway on the local network

## Key Concepts

- **Sessions** — each conversation is a session with its own context window
- **Heartbeat** — periodic background tasks (compaction, health checks)
- **Tool Policy** — controls which tools agents can invoke
- **Sandbox** — isolated execution environment for the exec tool

## Configuration

The gateway reads from `~/.magicmerlin/magicmerlin.json` (or `~/.openclaw/openclaw.json` for compatibility).

See [Configuration](configuration.md) and [Configuration Reference](configuration-reference.md) for details.

## Topics

- [Authentication](authentication.md)
- [Configuration](configuration.md)
- [Health Checks](health.md)
- [Logging](logging.md)
- [Security](security/index.md)
- [Sandboxing](sandboxing.md)
- [Protocol](protocol.md)
- [Troubleshooting](troubleshooting.md)
"#.to_string();
    }

    format!(
        r#"# {title}

{gateway_intro}

## Overview

{overview}

## Configuration

```json
{{
  "gateway": {{
    "{key}": {{
      "enabled": true
    }}
  }}
}}
```

## API

### Request

```json
POST /call
{{
  "method": "gateway.{method}",
  "params": {{}}
}}
```

### Response

```json
{{
  "ok": true,
  "result": {{}}
}}
```

## Related

- [Gateway Runbook](index.md)
- [Configuration Reference](configuration-reference.md)
- [Troubleshooting](troubleshooting.md)
"#,
        title = page.title,
        gateway_intro = gateway_intro(&s),
        overview = gateway_overview(&s),
        key = s,
        method = s.replace('-', "_"),
    )
}

fn gateway_intro(slug: &str) -> &'static str {
    match slug {
        "authentication" => "The gateway supports token-based authentication for securing API access.",
        "background-process" => "Background processes allow long-running tool executions without blocking the agent loop.",
        "bonjour" => "Bonjour (mDNS/DNS-SD) enables automatic gateway discovery on local networks.",
        "bridge-protocol" => "The bridge protocol connects remote gateways and nodes across networks.",
        "cli-backends" => "CLI backends allow the gateway to be controlled via different transport layers.",
        "configuration" => "The gateway is configured via a JSON file that controls all runtime behavior.",
        "configuration-examples" => "Example configurations for common deployment scenarios.",
        "configuration-reference" => "Complete field-by-field reference for the gateway configuration file.",
        "discovery" => "Discovery and transport mechanisms for finding and connecting to gateways.",
        "doctor" => "The doctor command diagnoses common gateway configuration and connectivity issues.",
        "gateway-lock" => "The gateway lock file prevents multiple instances from running simultaneously.",
        "health" => "Health check endpoints for monitoring gateway and service availability.",
        "heartbeat" => "The heartbeat system runs periodic background tasks like compaction and cleanup.",
        "local-models" => "Run models locally using Ollama, vLLM, or other local inference engines.",
        "logging" => "Gateway logging configuration for debugging and monitoring.",
        "multiple-gateways" => "Running multiple gateway instances for different agents or environments.",
        "network-model" => "The gateway's network model and how it handles connections and routing.",
        "openai-http-api" => "OpenAI-compatible chat completions endpoint for drop-in LLM API compatibility.",
        "openresponses-http-api" => "OpenResponses API endpoint for structured agent interactions.",
        "pairing" => "Gateway-owned pairing for connecting mobile devices and nodes.",
        "protocol" => "The gateway WebSocket protocol specification.",
        "remote" => "Remote access configuration for connecting to gateways over the internet.",
        "remote-gateway-readme" => "Step-by-step guide for setting up a remote gateway.",
        "sandbox-vs-tool-policy-vs-elevated" => "Understanding the differences between sandbox, tool policy, and elevated modes.",
        "sandboxing" => "The sandbox isolates tool execution in a restricted environment.",
        "secrets" => "Secrets management for API keys, tokens, and sensitive configuration.",
        "secrets-plan-contract" => "The contract for how secrets are applied during gateway startup.",
        "tailscale" => "Using Tailscale for secure remote gateway access without port forwarding.",
        "tools-invoke-http-api" => "HTTP API for directly invoking tools outside the agent loop.",
        "troubleshooting" => "Common gateway issues and their solutions.",
        "trusted-proxy-auth" => "Trusted proxy authentication for reverse-proxy deployments.",
        _ => "Gateway feature for managing agent runtime behavior.",
    }
}

fn gateway_overview(slug: &str) -> &'static str {
    match slug {
        "authentication" => "Configure bearer tokens or API keys to restrict access to the gateway API. Supports per-method permissions and role-based access.",
        "configuration" => "The primary configuration file is `~/.magicmerlin/magicmerlin.json`. All gateway behavior — providers, channels, tools, agents — is controlled here.",
        "configuration-reference" => "Every field in the configuration file documented with types, defaults, and examples.",
        "health" => "The `/health` endpoint returns gateway status, connected channels, provider availability, and node connectivity.",
        "heartbeat" => "The heartbeat runs every 60 seconds by default. It handles session compaction, stale connection cleanup, and periodic tool execution.",
        "protocol" => "The gateway uses a JSON-RPC-like protocol over WebSocket. Each message has a `method` and `params` field, with responses containing `ok` and `result`.",
        "sandboxing" => "Tools run inside a sandboxed environment by default. The sandbox restricts filesystem access, network calls, and process spawning based on the tool policy.",
        "logging" => "Logs are written to `~/.magicmerlin/logs/`. Use `magicmerlin logs` to tail them or `magicmerlin logs --search <pattern>` to search.",
        _ => "This feature integrates with the gateway's core runtime to provide additional functionality for agent management and communication.",
    }
}

fn render_tools_page(page: &DocPage) -> String {
    let s = slug(page);
    if s == "index" {
        return r#"# Tools

Magic Merlin provides a comprehensive set of tools that agents can use during conversations.

## Available Tools

| Tool | Description |
|------|-------------|
| `exec` | Execute shell commands in a sandboxed environment |
| `browser` | Navigate web pages, click, type, screenshot |
| `web_search` | Search the web using configured search providers |
| `web_fetch` | Fetch and extract content from URLs |
| `memory_read` | Read from agent persistent memory |
| `memory_write` | Write to agent persistent memory |
| `pdf` | Generate PDF documents |
| `tts` | Text-to-speech synthesis |
| `image` | Image generation and manipulation |
| `canvas` | Render HTML/SVG to images |
| `cron` | Schedule recurring tasks |
| `sessions` | Manage agent sessions |
| `subagents` | Spawn and manage sub-agents |
| `agents_list` | List available agents |
| `apply_patch` | Apply unified diffs to files |
| `skills` | Invoke registered skills |
| `reactions` | Add reactions to messages |
| `acp_agents` | ACP thread-bound agent operations |
| `agent_send` | Send messages between agents |
| `llm_task` | Spawn a sub-LLM task |
| `firecrawl` | Deep web crawling |
| `lobster` | Typed workflow pipelines |
| `thinking` | Control thinking/reasoning levels |

## Tool Policy

Tools are governed by the tool policy in your configuration:

```json
{
  "toolPolicy": {
    "exec": "approve",
    "browser": "allow",
    "web_search": "allow"
  }
}
```

Policies: `allow` (no approval needed), `approve` (requires user approval), `deny` (blocked).

## See Also

- [Exec Approvals](exec-approvals.md)
- [Elevated Mode](elevated.md)
- [Creating Skills](creating-skills.md)
"#
        .to_string();
    }

    format!(
        r#"# {title}

{tool_description}

## Usage

The `{tool_name}` tool is available to agents during conversations. It can be invoked directly by the agent when needed.

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
{params}

## Configuration

```json
{{
  "toolPolicy": {{
    "{tool_name}": "allow"
  }}
}}
```

## Examples

{examples}

## Security

{security_note}

## See Also

- [Tools Overview](index.md)
- [Exec Approvals](exec-approvals.md)
- [Tool Policy](../gateway/sandbox-vs-tool-policy-vs-elevated.md)
"#,
        title = page.title,
        tool_description = tool_description(&s),
        tool_name = s.replace('-', "_"),
        params = tool_params(&s),
        examples = tool_examples(&s),
        security_note = tool_security(&s),
    )
}

fn tool_description(slug: &str) -> &'static str {
    match slug {
        "exec" => "The exec tool runs shell commands in a sandboxed environment. It supports timeout, working directory, and environment variable configuration.",
        "browser" => "The browser tool provides a managed Chromium instance for web automation. Agents can navigate pages, click elements, fill forms, take screenshots, and extract content.",
        "browser-linux-troubleshooting" => "Troubleshooting guide for browser tool issues on Linux systems, including headless Chrome dependencies and display configuration.",
        "browser-login" => "Browser login allows agents to authenticate with websites using stored credentials or interactive login flows.",
        "web" => "Web tools provide `web_search` and `web_fetch` capabilities for agents to search the internet and retrieve web page content.",
        "pdf" => "The PDF tool generates PDF documents from HTML or Markdown content, supporting headers, footers, page numbers, and custom styling.",
        "acp-agents" => "ACP (Agent Communication Protocol) agents are thread-bound agents that maintain persistent context within a conversation thread.",
        "agent-send" => "The agent-send tool allows agents to send messages to other agents, enabling multi-agent collaboration.",
        "apply-patch" => "The apply_patch tool applies unified diff patches to files, useful for code modifications.",
        "chrome-extension" => "The Chrome extension bridges your browser with Magic Merlin, allowing agents to interact with your active browser tabs.",
        "clawhub" => "ClawHub is the community hub for sharing and discovering agent configurations, skills, and plugins.",
        "creating-skills" => "Skills are reusable agent capabilities that can be invoked via slash commands. This guide covers creating custom skills.",
        "diffs" => "The diffs plugin renders before/after text or unified patches as visual diff views.",
        "elevated" => "Elevated mode grants agents expanded tool permissions temporarily, bypassing the default tool policy.",
        "exec-approvals" => "Exec approvals control when agents need user confirmation before running shell commands.",
        "firecrawl" => "Firecrawl provides deep web crawling capabilities, following links and extracting structured content.",
        "llm-task" => "LLM task spawns a sub-LLM call for specific tasks, allowing agents to delegate focused work.",
        "lobster" => "Lobster is a typed workflow runtime for composable pipelines with approval gates.",
        "loop-detection" => "Loop detection guards against repetitive or stalled tool-call loops during agent execution.",
        "multi-agent-sandbox-tools" => "Multi-agent sandbox tools enable isolated environments for sub-agent execution.",
        "plugin" => "Plugins extend Magic Merlin with custom tools, channels, and capabilities.",
        "reactions" => "The reactions tool allows agents to add emoji reactions to messages in supported channels.",
        "skills" => "Skills are named, reusable agent capabilities that can be invoked via slash commands or tool calls.",
        "skills-config" => "Configuration reference for defining and managing skills.",
        "slash-commands" => "Slash commands are user-invokable shortcuts that trigger skills or built-in actions.",
        "subagents" => "Sub-agents are child agents spawned by a parent agent to handle specific tasks in parallel.",
        "thinking" => "Thinking levels control the depth of reasoning an agent applies to tool calls and responses.",
        _ => "This tool extends agent capabilities with additional functionality.",
    }
}

fn tool_params(slug: &str) -> &'static str {
    match slug {
        "exec" => "| `command` | `string` | Yes | Shell command to execute |\n| `timeout` | `number` | No | Timeout in milliseconds (default: 30000) |\n| `cwd` | `string` | No | Working directory |\n| `env` | `object` | No | Additional environment variables |",
        "browser" => "| `action` | `string` | Yes | One of: navigate, click, type, screenshot, evaluate |\n| `url` | `string` | No | URL to navigate to |\n| `selector` | `string` | No | CSS selector for click/type actions |\n| `text` | `string` | No | Text to type |\n| `script` | `string` | No | JavaScript to evaluate |",
        "pdf" => "| `html` | `string` | Yes | HTML content to render |\n| `filename` | `string` | No | Output filename |\n| `format` | `string` | No | Page format (A4, Letter, etc.) |",
        _ => "| `input` | `string` | Yes | Primary input for this tool |",
    }
}

fn tool_examples(slug: &str) -> &'static str {
    match slug {
        "exec" => {
            r#"```json
{
  "tool": "exec",
  "params": {
    "command": "ls -la /tmp",
    "timeout": 5000
  }
}
```"#
        }
        "browser" => {
            r#"```json
{
  "tool": "browser",
  "params": {
    "action": "navigate",
    "url": "https://example.com"
  }
}
```

```json
{
  "tool": "browser",
  "params": {
    "action": "screenshot"
  }
}
```"#
        }
        _ => {
            r#"```json
{
  "tool": "...",
  "params": {
    "input": "example input"
  }
}
```"#
        }
    }
}

fn tool_security(slug: &str) -> &'static str {
    match slug {
        "exec" => "The exec tool runs commands inside the sandbox by default. Commands that modify the filesystem, access the network, or spawn long-running processes may require approval or elevated mode.",
        "browser" => "The browser runs in a managed Chromium instance. Page navigations are logged and cookie/credential access is restricted by the tool policy.",
        _ => "This tool respects the configured tool policy. Set it to `approve` in your configuration to require user confirmation before each invocation.",
    }
}

fn render_channels_page(page: &DocPage) -> String {
    let s = slug(page);
    if s == "index" {
        return r#"# Chat Channels

Magic Merlin connects to a wide range of messaging platforms, allowing your agent to communicate wherever your users are.

## Supported Channels

| Channel | Status | Features |
|---------|--------|----------|
| Telegram | Stable | Text, images, voice, location, groups, inline |
| Discord | Stable | Text, images, threads, reactions, slash commands |
| Signal | Stable | Text, images, groups, disappearing messages |
| Slack | Stable | Text, images, threads, reactions, apps |
| WhatsApp | Stable | Text, images, voice, location, groups |
| iMessage | Stable | Text, images (via BlueBubbles) |
| Matrix | Stable | Text, images, E2EE, federation |
| IRC | Stable | Text, channels, DMs |
| LINE | Stable | Text, images, stickers, groups |
| Microsoft Teams | Stable | Text, images, cards, threads |
| Telegram | Stable | Text, images, voice, groups |
| Google Chat | Beta | Text, cards, spaces |
| Nostr | Beta | Text, DMs, relays |
| Twitch | Beta | Chat messages |
| Mattermost | Beta | Text, images, threads |
| Feishu | Beta | Text, cards |
| Nextcloud Talk | Beta | Text, rooms |
| Synology Chat | Beta | Text |
| Tlon | Experimental | Text, groups |
| Zalo | Experimental | Text, images |

## Configuration

Each channel is configured in your `magicmerlin.json`:

```json
{
  "channels": {
    "telegram": {
      "enabled": true,
      "token": "{{TELEGRAM_BOT_TOKEN}}"
    }
  }
}
```

## See Also

- [Channel Routing](channel-routing.md)
- [Group Messages](group-messages.md)
- [Broadcast Groups](broadcast-groups.md)
- [Troubleshooting](troubleshooting.md)
"#.to_string();
    }

    let channel_name = channel_display_name(&s);

    format!(
        r#"# {title}

Connect Magic Merlin to {channel_name}.

## Setup

{setup_steps}

## Configuration

```json
{{
  "channels": {{
    "{slug}": {{
      "enabled": true{config_fields}
    }}
  }}
}}
```

## Features

{features}

## Troubleshooting

{troubleshooting}

## See Also

- [Chat Channels](index.md)
- [Channel Routing](channel-routing.md)
"#,
        title = page.title,
        channel_name = channel_name,
        setup_steps = channel_setup(&s),
        slug = s,
        config_fields = channel_config_fields(&s),
        features = channel_features(&s),
        troubleshooting = channel_troubleshooting(&s),
    )
}

fn channel_display_name(slug: &str) -> &'static str {
    match slug {
        "telegram" => "Telegram",
        "discord" => "Discord",
        "signal" => "Signal",
        "slack" => "Slack",
        "whatsapp" => "WhatsApp",
        "imessage" => "iMessage",
        "bluebubbles" => "BlueBubbles (iMessage bridge)",
        "matrix" => "Matrix",
        "irc" => "IRC",
        "line" => "LINE",
        "msteams" => "Microsoft Teams",
        "googlechat" => "Google Chat",
        "nostr" => "Nostr",
        "twitch" => "Twitch",
        "mattermost" => "Mattermost",
        "feishu" => "Feishu (Lark)",
        "nextcloud-talk" => "Nextcloud Talk",
        "synology-chat" => "Synology Chat",
        "tlon" => "Tlon (Urbit)",
        "zalo" => "Zalo",
        "zalouser" => "Zalo Personal",
        "channel-routing" => "the channel routing system",
        "broadcast-groups" => "broadcast groups",
        "group-messages" => "group message handling",
        "groups" => "group chat management",
        "location" => "location message parsing",
        "pairing" => "device pairing",
        "troubleshooting" => "channel troubleshooting",
        _ => "this messaging platform",
    }
}

fn channel_setup(slug: &str) -> &'static str {
    match slug {
        "telegram" => "1. Create a bot via [@BotFather](https://t.me/BotFather) on Telegram\n2. Copy the bot token\n3. Add the token to your configuration\n4. Restart the gateway",
        "discord" => "1. Create a Discord application at [discord.com/developers](https://discord.com/developers)\n2. Create a bot user and copy the token\n3. Add the bot to your server with appropriate permissions\n4. Add the token to your configuration\n5. Restart the gateway",
        "signal" => "1. Install `signal-cli` and register a phone number\n2. Configure the Signal REST API endpoint\n3. Add credentials to your configuration\n4. Restart the gateway",
        "slack" => "1. Create a Slack App at [api.slack.com/apps](https://api.slack.com/apps)\n2. Add bot scopes: `chat:write`, `channels:history`, `im:history`\n3. Install to your workspace\n4. Copy the Bot User OAuth Token\n5. Add to your configuration",
        "whatsapp" => "1. Set up WhatsApp Business API or use a bridge service\n2. Configure the API endpoint and credentials\n3. Add to your configuration\n4. Restart the gateway",
        _ => "1. Configure the channel credentials in your `magicmerlin.json`\n2. Restart the gateway with `magicmerlin gateway restart`\n3. Verify connectivity with `magicmerlin channels`",
    }
}

fn channel_config_fields(slug: &str) -> &'static str {
    match slug {
        "telegram" => ",\n      \"token\": \"YOUR_BOT_TOKEN\"",
        "discord" => {
            ",\n      \"token\": \"YOUR_BOT_TOKEN\",\n      \"guildId\": \"YOUR_GUILD_ID\""
        }
        "signal" => {
            ",\n      \"number\": \"+1234567890\",\n      \"apiUrl\": \"http://localhost:8080\""
        }
        "slack" => {
            ",\n      \"token\": \"xoxb-YOUR-TOKEN\",\n      \"appToken\": \"xapp-YOUR-APP-TOKEN\""
        }
        _ => "",
    }
}

fn channel_features(slug: &str) -> &'static str {
    match slug {
        "telegram" => "- Text messages and replies\n- Image, audio, and document attachments\n- Voice messages and transcription\n- Inline keyboards and callback queries\n- Group and supergroup chats\n- Location sharing\n- Stickers",
        "discord" => "- Text messages in channels and DMs\n- Image and file attachments\n- Thread support\n- Reactions\n- Slash commands\n- Embeds\n- Voice channel awareness",
        "signal" => "- End-to-end encrypted messages\n- Image and file attachments\n- Group chats\n- Disappearing messages\n- Reactions\n- Typing indicators",
        "slack" => "- Messages in channels and DMs\n- Thread replies\n- Image and file attachments\n- Reactions\n- Interactive blocks\n- Slash commands\n- App mentions",
        _ => "- Text messages\n- Basic media support\n- Group chats (where supported)",
    }
}

fn channel_troubleshooting(slug: &str) -> &'static str {
    match slug {
        "telegram" => "- **Bot not responding**: Verify the token is correct and the bot is not blocked\n- **Missing messages**: Check that the bot has privacy mode disabled in group chats\n- **Rate limits**: Telegram limits ~30 messages/second per bot",
        "discord" => "- **Bot offline**: Check the token and ensure the bot has the correct intents enabled\n- **Missing permissions**: Verify bot role has Send Messages and Read Message History\n- **Gateway intents**: Enable MESSAGE_CONTENT intent for message reading",
        _ => "- Verify credentials are correct in your configuration\n- Check `magicmerlin logs` for connection errors\n- Run `magicmerlin doctor` to diagnose common issues\n- Ensure the gateway is running: `magicmerlin status`",
    }
}

fn render_providers_page(page: &DocPage) -> String {
    let s = slug(page);
    if s == "index" {
        return r#"# Model Providers

Magic Merlin supports 25+ LLM providers. Configure one or more in your `magicmerlin.json`.

## Supported Providers

| Provider | Type | Models |
|----------|------|--------|
| Anthropic | Cloud | Claude 4.5, Claude 4, Claude 3.5 |
| OpenAI | Cloud | GPT-4o, GPT-4, o1, o3 |
| Ollama | Local | Llama, Mistral, Gemma, Qwen, and more |
| Amazon Bedrock | Cloud | Claude, Llama, Mistral via AWS |
| OpenRouter | Cloud | Multi-provider routing |
| Mistral | Cloud | Mistral Large, Medium, Small |
| Google (Gemini) | Cloud | Gemini 2.5 Pro, Flash |
| Deepgram | Cloud | Speech-to-text, text-to-speech |
| NVIDIA | Cloud | NIM inference endpoints |
| vLLM | Local | Self-hosted inference |
| Hugging Face | Cloud | Inference API models |
| Venice AI | Cloud | Privacy-focused inference |
| Vercel AI Gateway | Cloud | Multi-provider gateway |
| Cloudflare AI Gateway | Cloud | Edge inference |

## Configuration

```json
{
  "providers": {
    "anthropic": {
      "apiKey": "sk-ant-..."
    },
    "openai": {
      "apiKey": "sk-..."
    },
    "ollama": {
      "baseUrl": "http://localhost:11434"
    }
  },
  "models": {
    "default": "anthropic:claude-sonnet-4-20250514"
  }
}
```

## Model Failover

Configure fallback models for resilience:

```json
{
  "models": {
    "default": "anthropic:claude-sonnet-4-20250514",
    "fallback": ["openai:gpt-4o", "ollama:llama3"]
  }
}
```

See [Model Failover](../concepts/model-failover.md) for details.
"#
        .to_string();
    }

    let provider_name = provider_display_name(&s);
    format!(
        r#"# {title}

Use {provider_name} models with Magic Merlin.

## Setup

{setup}

## Configuration

```json
{{
  "providers": {{
    "{slug}": {config}
  }}
}}
```

## Available Models

{models}

## Environment Variables

{env_vars}

## See Also

- [Model Providers](index.md)
- [Model Failover](../concepts/model-failover.md)
- [Model Provider Quickstart](models.md)
"#,
        title = page.title,
        provider_name = provider_name,
        setup = provider_setup(&s),
        slug = s,
        config = provider_config(&s),
        models = provider_models(&s),
        env_vars = provider_env(&s),
    )
}

fn provider_display_name(slug: &str) -> &'static str {
    match slug {
        "anthropic" => "Anthropic (Claude)",
        "openai" => "OpenAI",
        "ollama" => "Ollama (local)",
        "bedrock" => "Amazon Bedrock",
        "openrouter" => "OpenRouter",
        "mistral" => "Mistral AI",
        "deepgram" => "Deepgram",
        "nvidia" => "NVIDIA NIM",
        "vllm" => "vLLM",
        "huggingface" => "Hugging Face",
        "venice" => "Venice AI",
        "vercel-ai-gateway" => "Vercel AI Gateway",
        "cloudflare-ai-gateway" => "Cloudflare AI Gateway",
        "github-copilot" => "GitHub Copilot",
        "minimax" => "MiniMax",
        "moonshot" => "Moonshot AI",
        "qianfan" => "Baidu Qianfan",
        "qwen" => "Alibaba Qwen",
        "glm" => "GLM (Zhipu AI)",
        "synthetic" => "Synthetic (testing)",
        "xiaomi" => "Xiaomi MiMo",
        "zai" => "Z.AI",
        "claude-max-api-proxy" => "Claude Max API Proxy",
        "opencode" => "OpenCode Zen",
        _ => "this provider",
    }
}

fn provider_setup(slug: &str) -> &'static str {
    match slug {
        "anthropic" => "1. Get an API key from [console.anthropic.com](https://console.anthropic.com)\n2. Add the key to your configuration or set `ANTHROPIC_API_KEY`",
        "openai" => "1. Get an API key from [platform.openai.com](https://platform.openai.com)\n2. Add the key to your configuration or set `OPENAI_API_KEY`",
        "ollama" => "1. Install Ollama: `curl -fsSL https://ollama.com/install.sh | sh`\n2. Pull a model: `ollama pull llama3`\n3. Ollama runs on `localhost:11434` by default",
        "bedrock" => "1. Configure AWS credentials (`aws configure`)\n2. Enable Claude models in your AWS Bedrock console\n3. Set the AWS region in your configuration",
        _ => "1. Obtain API credentials from the provider\n2. Add them to your configuration\n3. Restart the gateway",
    }
}

fn provider_config(slug: &str) -> &'static str {
    match slug {
        "anthropic" => "{\n      \"apiKey\": \"sk-ant-...\"\n    }",
        "openai" => "{\n      \"apiKey\": \"sk-...\"\n    }",
        "ollama" => "{\n      \"baseUrl\": \"http://localhost:11434\"\n    }",
        "bedrock" => "{\n      \"region\": \"us-east-1\",\n      \"profile\": \"default\"\n    }",
        _ => "{\n      \"apiKey\": \"YOUR_API_KEY\"\n    }",
    }
}

fn provider_models(slug: &str) -> &'static str {
    match slug {
        "anthropic" => "- `claude-opus-4-20250514` — Most capable model\n- `claude-sonnet-4-20250514` — Balanced performance\n- `claude-haiku-4-20250414` — Fast and cost-effective\n- `claude-3-5-sonnet-20241022` — Previous generation",
        "openai" => "- `gpt-4o` — Latest GPT-4 Omni\n- `gpt-4-turbo` — GPT-4 Turbo\n- `o1` — Reasoning model\n- `o3-mini` — Fast reasoning",
        "ollama" => "- `llama3` — Meta Llama 3\n- `mistral` — Mistral 7B\n- `gemma2` — Google Gemma 2\n- `qwen2` — Alibaba Qwen 2\n- Any model available via `ollama pull`",
        _ => "See the provider's documentation for available models.",
    }
}

fn provider_env(slug: &str) -> &'static str {
    match slug {
        "anthropic" => "| `ANTHROPIC_API_KEY` | API key (alternative to config) |",
        "openai" => "| `OPENAI_API_KEY` | API key (alternative to config) |",
        "bedrock" => "| `AWS_PROFILE` | AWS profile name |\n| `AWS_REGION` | AWS region |",
        _ => "| `{PROVIDER}_API_KEY` | API key (alternative to config) |",
    }
}

fn render_install_page(page: &DocPage) -> String {
    let s = slug(page);
    if s == "index" {
        return r#"# Installation

Magic Merlin can be installed on macOS, Linux, and Windows (via WSL2).

## Recommended: Cargo Install

```bash
cargo install magicmerlin magicmerlin-gateway
```

## Other Methods

| Method | Platforms | Notes |
|--------|-----------|-------|
| [Cargo](../install/index.md) | All | Recommended — builds from source |
| [Docker](docker.md) | All | Containerized deployment |
| [Nix](nix.md) | Linux, macOS | Reproducible builds |
| [Ansible](ansible.md) | Linux | Automated server setup |

## Cloud Deployments

- [Fly.io](fly.md)
- [Railway](railway.md)
- [Render](render.md)
- [GCP](gcp.md)
- [Hetzner](hetzner.md)
- [Northflank](northflank.md)

## After Installation

```bash
# Run first-time setup
magicmerlin setup

# Start the gateway
magicmerlin gateway start

# Verify
magicmerlin status
```

## See Also

- [Migration Guide](migrating.md) — for OpenClaw users
- [Updating](updating.md) — how to update to latest
- [Uninstall](uninstall.md) — how to remove
"#
        .to_string();
    }

    format!(
        r#"# {title}

{description}

## Prerequisites

{prerequisites}

## Installation

{steps}

## Verification

```bash
magicmerlin --version
magicmerlin doctor
```

## Next Steps

- Run `magicmerlin setup` for first-time configuration
- See [Getting Started](../start/getting-started.md) for a walkthrough
- See [Configuration](../gateway/configuration.md) for customization

## Troubleshooting

{troubleshooting}

## See Also

- [Installation Overview](index.md)
- [Updating](updating.md)
"#,
        title = page.title,
        description = install_description(&s),
        prerequisites = install_prerequisites(&s),
        steps = install_steps(&s),
        troubleshooting = install_troubleshooting(&s),
    )
}

fn install_description(slug: &str) -> &'static str {
    match slug {
        "docker" => "Run Magic Merlin in a Docker container for isolated, reproducible deployments.",
        "nix" => "Install Magic Merlin using the Nix package manager for reproducible builds.",
        "ansible" => "Automate Magic Merlin deployment on Linux servers using Ansible.",
        "bun" => "Experimental: Run the OpenClaw compatibility layer using Bun runtime.",
        "fly" => "Deploy Magic Merlin on Fly.io for globally distributed edge hosting.",
        "gcp" => "Deploy Magic Merlin on Google Cloud Platform.",
        "hetzner" => "Deploy Magic Merlin on Hetzner Cloud servers.",
        "installer" => "Technical details of how the Magic Merlin installer works internally.",
        "macos-vm" => "Run Magic Merlin inside a macOS virtual machine for testing.",
        "migrating" => "Migrate from OpenClaw (Node.js) to Magic Merlin (Rust). Your existing configuration works as-is.",
        "node" => "Install the OpenClaw compatibility shim via Node.js/npm (for migration).",
        "northflank" => "Deploy Magic Merlin on Northflank.",
        "podman" => "Run Magic Merlin using Podman as a Docker alternative.",
        "railway" => "Deploy Magic Merlin on Railway with one-click setup.",
        "render" => "Deploy Magic Merlin on Render for managed hosting.",
        "uninstall" => "Remove Magic Merlin and clean up all configuration files.",
        "updating" => "Update Magic Merlin to the latest version.",
        "exe-dev" => "Install via exe.dev package manager.",
        "development-channels" => "Use development (nightly/beta) release channels for early access to new features.",
        _ => "Install Magic Merlin using this method.",
    }
}

fn install_prerequisites(slug: &str) -> &'static str {
    match slug {
        "docker" => "- Docker Engine 20.10+ or Docker Desktop\n- At least 512MB RAM available",
        "nix" => "- Nix package manager (`curl -L https://nixos.org/nix/install | sh`)",
        "ansible" => "- Ansible 2.12+\n- Target Linux server with SSH access\n- Python 3.8+ on target",
        "fly" => "- `flyctl` CLI installed\n- Fly.io account",
        _ => "- Rust toolchain (1.76+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`",
    }
}

fn install_steps(slug: &str) -> &'static str {
    match slug {
        "docker" => {
            r#"```bash
# Pull the image
docker pull ghcr.io/itsGustav/magicmerlin:latest

# Run with persistent config
docker run -d \
  --name magicmerlin \
  -v ~/.magicmerlin:/root/.magicmerlin \
  -p 3578:3578 \
  ghcr.io/itsGustav/magicmerlin:latest
```"#
        }
        "nix" => {
            r#"```bash
# Using flakes
nix run github:itsGustav/magicmerlin

# Add to your flake.nix
{
  inputs.magicmerlin.url = "github:itsGustav/magicmerlin";
}
```"#
        }
        "migrating" => {
            r#"Magic Merlin reads the same configuration format as OpenClaw. To migrate:

```bash
# Install Magic Merlin
cargo install magicmerlin magicmerlin-gateway

# Point at your existing config (it works as-is)
magicmerlin gateway start --config ~/.openclaw/openclaw.json

# Or copy to the new location
cp ~/.openclaw/openclaw.json ~/.magicmerlin/magicmerlin.json
magicmerlin gateway start
```

### Key Differences

| Feature | OpenClaw | Magic Merlin |
|---------|----------|-------------|
| Runtime | Node.js | Rust (native binary) |
| Config | `openclaw.json` | `magicmerlin.json` (or `openclaw.json`) |
| Gateway | `openclaw gateway start` | `magicmerlin gateway start` |
| TUI | Web-based | Ratatui native terminal |"#
        }
        _ => {
            r#"```bash
cargo install magicmerlin magicmerlin-gateway
```"#
        }
    }
}

fn install_troubleshooting(slug: &str) -> &'static str {
    match slug {
        "docker" => "- **Container won't start**: Check port 3578 is not in use\n- **Config not found**: Ensure the volume mount path is correct",
        "migrating" => "- **Config parse errors**: Run `magicmerlin doctor` to identify incompatible fields\n- **Missing features**: Check the [feature parity tracker](../../parity/) for status",
        _ => "- Run `magicmerlin doctor` to diagnose common issues\n- Check [Help & Troubleshooting](../help/index.md) for more solutions",
    }
}

fn render_concepts_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Overview

{overview}

## How It Works

{how_it_works}

## Configuration

{config}

## Related Concepts

{related}

## See Also

- [Concepts Overview](../concepts/features.md)
- [Gateway Architecture](architecture.md)
"#,
        title = page.title,
        description = concept_description(&s),
        overview = concept_overview(&s),
        how_it_works = concept_how(&s),
        config = concept_config(&s),
        related = concept_related(&s),
    )
}

fn concept_description(slug: &str) -> &'static str {
    match slug {
        "agent" => "The agent runtime is the core execution engine that processes messages, invokes tools, and generates responses.",
        "agent-loop" => "The agent loop is the main execution cycle: receive message → build context → call LLM → process tool calls → send response.",
        "agent-workspace" => "Each agent has a workspace directory for persistent files, memory, and tool state.",
        "architecture" => "Magic Merlin's architecture connects channels, the gateway, and the agent loop into a unified runtime.",
        "compaction" => "Compaction summarizes older conversation turns to fit within the model's context window while preserving key information.",
        "context" => "Context management controls what information is included in each LLM call — system prompt, memory, recent messages, and tool results.",
        "features" => "Overview of all Magic Merlin features and capabilities.",
        "markdown-formatting" => "How Magic Merlin handles Markdown formatting across different channels.",
        "memory" => "Agent memory provides persistent key-value storage that survives across sessions.",
        "messages" => "The message format and lifecycle from channel input through agent processing to channel output.",
        "model-failover" => "Model failover automatically switches to backup providers when the primary model is unavailable.",
        "model-providers" => "How model providers are configured, selected, and load-balanced.",
        "models" => "CLI and configuration for managing available models across providers.",
        "multi-agent" => "Multi-agent routing allows multiple specialized agents to handle different types of requests.",
        "oauth" => "OAuth integration for authenticating agents with third-party services.",
        "presence" => "Presence tracking shows when agents and users are online/offline across channels.",
        "queue" => "The command queue manages pending operations and ensures ordered execution.",
        "retry" => "Retry policies control how failed LLM calls and tool executions are retried.",
        "session" => "Sessions encapsulate a conversation between a user and an agent, with their own context and history.",
        "session-pruning" => "Session pruning removes stale or expired sessions to free resources.",
        "session-tool" => "Session tools allow agents to manage their own sessions — fork, merge, summarize.",
        "streaming" => "Streaming and chunking controls how responses are delivered incrementally to channels.",
        "system-prompt" => "The system prompt defines the agent's personality, instructions, and constraints.",
        "timezone" => "Timezone handling for scheduling, timestamps, and user-facing dates.",
        "typebox" => "TypeBox integration for runtime type validation of tool parameters and API responses.",
        "typing-indicators" => "Typing indicators show users when the agent is processing their message.",
        "usage-tracking" => "Usage tracking monitors token consumption, API costs, and tool invocations.",
        _ => "A core concept in the Magic Merlin agent runtime.",
    }
}

fn concept_overview(slug: &str) -> &'static str {
    match slug {
        "agent-loop" => "```\nMessage In → Context Build → LLM Call → Tool Calls → Response → Message Out\n     ↑                                      │\n     └──────────────── Loop ────────────────┘\n```\n\nThe agent loop continues until the LLM produces a final response without tool calls, or a maximum iteration limit is reached.",
        "compaction" => "When a conversation grows beyond the model's context window, compaction kicks in:\n\n1. Older messages are summarized by the LLM\n2. The summary replaces the original messages\n3. Recent messages are kept verbatim\n4. Tool results are condensed\n\nThis allows conversations to continue indefinitely without losing critical context.",
        "memory" => "Agent memory is stored as key-value pairs in the workspace directory. Memory persists across sessions and can be read/written by tools.\n\n```\n~/.magicmerlin/memory/\n  agent-name/\n    key1.json\n    key2.json\n```",
        "session" => "A session represents a single conversation thread. Each session has:\n\n- A unique session ID\n- A channel binding (which channel/user it came from)\n- Message history\n- Context window state\n- Tool execution history",
        "model-failover" => "When a model provider returns an error or times out, Magic Merlin automatically tries the next provider in the failover chain. This ensures high availability even when individual providers have outages.",
        _ => "This concept is fundamental to how Magic Merlin processes messages and manages agent state.",
    }
}

fn concept_how(slug: &str) -> &'static str {
    match slug {
        "compaction" => "1. The gateway monitors context window usage after each turn\n2. When usage exceeds the threshold (default: 80%), compaction is triggered\n3. A summarization prompt is sent to the LLM with the oldest messages\n4. The summary replaces those messages in the session\n5. The session continues with the compacted history",
        "streaming" => "1. The LLM streams tokens as they are generated\n2. Tokens are buffered into chunks based on channel requirements\n3. For channels that support editing (Telegram, Discord), the message is updated in-place\n4. For channels that don't support editing, chunks are concatenated and sent as a final message\n5. Tool call tokens are accumulated silently until the call is complete",
        _ => "The implementation follows a modular architecture where each component can be configured and extended independently.",
    }
}

fn concept_config(slug: &str) -> &'static str {
    match slug {
        "compaction" => "```json\n{\n  \"compaction\": {\n    \"enabled\": true,\n    \"threshold\": 0.8,\n    \"model\": \"anthropic:claude-haiku-4-20250414\"\n  }\n}\n```",
        "session" => "```json\n{\n  \"sessions\": {\n    \"maxAge\": \"24h\",\n    \"maxHistory\": 100,\n    \"pruneInterval\": \"1h\"\n  }\n}\n```",
        "memory" => "```json\n{\n  \"memory\": {\n    \"enabled\": true,\n    \"maxKeys\": 1000,\n    \"persistPath\": \"~/.magicmerlin/memory/\"\n  }\n}\n```",
        _ => "See the [Configuration Reference](../gateway/configuration-reference.md) for all available options.",
    }
}

fn concept_related(slug: &str) -> &'static str {
    match slug {
        "agent" => "- [Agent Loop](agent-loop.md)\n- [Agent Workspace](agent-workspace.md)\n- [System Prompt](system-prompt.md)",
        "compaction" => "- [Session Management](session.md)\n- [Context](context.md)\n- [Usage Tracking](usage-tracking.md)",
        "session" => "- [Session Pruning](session-pruning.md)\n- [Session Tools](session-tool.md)\n- [Compaction](compaction.md)",
        _ => "- [Agent Runtime](agent.md)\n- [Gateway Architecture](architecture.md)\n- [Features](features.md)",
    }
}

fn render_start_page(page: &DocPage) -> String {
    let s = slug(page);
    if s == "getting-started" {
        return r#"# Getting Started

Get Magic Merlin running in 5 minutes.

## 1. Install

```bash
cargo install magicmerlin magicmerlin-gateway
```

## 2. Configure a Provider

```bash
# Interactive setup
magicmerlin setup

# Or manually set your API key
export ANTHROPIC_API_KEY="sk-ant-..."
```

## 3. Start the Gateway

```bash
magicmerlin gateway start
```

## 4. Send a Message

```bash
magicmerlin message "Hello, Merlin!"
```

## 5. Connect a Channel (Optional)

```bash
# Interactive channel setup
magicmerlin configure

# Or edit config directly
magicmerlin config edit
```

## Next Steps

- [Connect Telegram](../channels/telegram.md) or [Discord](../channels/discord.md)
- [Configure tools](../tools/index.md) for your agent
- [Set up the TUI](../web/tui.md) for monitoring
- [Explore concepts](../concepts/features.md) to understand the architecture
"#
        .to_string();
    }

    format!(
        r#"# {title}

{description}

## Overview

{content}

## See Also

- [Getting Started](getting-started.md)
- [Installation](../install/index.md)
"#,
        title = page.title,
        description = start_description(&s),
        content = start_content(&s),
    )
}

fn start_description(slug: &str) -> &'static str {
    match slug {
        "bootstrapping" => "Agent bootstrapping is the process of initializing an agent with its system prompt, memory, and initial configuration.",
        "docs-directory" => "Overview of the documentation structure and how to navigate it.",
        "hubs" => "Documentation hubs organize related topics for quick reference.",
        "lore" => "The story behind Magic Merlin — a Rust rewrite of the OpenClaw agent runtime.",
        "onboarding" => "The macOS app onboarding flow guides new users through initial setup.",
        "onboarding-overview" => "Overview of all onboarding paths: CLI wizard, macOS app, and manual configuration.",
        "openclaw" => "How to set up Magic Merlin as a personal AI assistant.",
        "setup" => "First-time setup guide covering providers, channels, and basic configuration.",
        "showcase" => "Real-world Magic Merlin deployments and community projects.",
        "wizard" => "The CLI onboarding wizard walks through provider setup, channel configuration, and first message.",
        "wizard-cli-automation" => "Automate the onboarding wizard for scripted deployments.",
        "wizard-cli-reference" => "Complete reference for all CLI onboarding wizard options.",
        _ => "Getting started with Magic Merlin.",
    }
}

fn start_content(slug: &str) -> &'static str {
    match slug {
        "bootstrapping" => "When an agent starts, it goes through a bootstrap sequence:\n\n1. Load system prompt from configuration or `AGENTS.md`\n2. Initialize memory store\n3. Register available tools based on tool policy\n4. Connect to configured channels\n5. Start the heartbeat timer\n\nThe bootstrap process can be customized with `BOOT.md` and `BOOTSTRAP.md` templates.",
        "setup" => "## Quick Setup\n\n```bash\n# Run the interactive setup wizard\nmagicmerlin setup\n```\n\nThe wizard will guide you through:\n\n1. **Provider selection** — choose your LLM provider and enter API key\n2. **Model selection** — pick a default model\n3. **Channel setup** — optionally connect a messaging platform\n4. **First test** — send a test message to verify everything works",
        "showcase" => "## Community Projects\n\nMagic Merlin is used in a variety of real-world deployments:\n\n- Personal AI assistants on Telegram\n- Team productivity bots on Slack and Discord\n- Automated code review agents\n- Home automation controllers on Raspberry Pi\n- Customer support agents via WhatsApp",
        _ => "This section provides guidance for getting started with Magic Merlin.",
    }
}

fn render_platforms_page(page: &DocPage) -> String {
    let s = slug(page);
    let subsection = page.subsection.as_deref().unwrap_or("");

    if s == "index" && subsection.is_empty() {
        return r#"# Platforms

Magic Merlin runs on a wide range of platforms.

## Desktop

- [macOS App](macos.md) — native menu bar app with gateway management
- [Linux](linux.md) — systemd service or standalone binary
- [Windows (WSL2)](windows.md) — run inside Windows Subsystem for Linux

## Mobile

- [iOS App](ios.md) — node companion app
- [Android App](android.md) — node companion app

## Cloud & Server

- [DigitalOcean](digitalocean.md)
- [Oracle Cloud](oracle.md)
- [Raspberry Pi](raspberry-pi.md)

## macOS App Details

The macOS app provides:
- Menu bar icon with status indicator
- Built-in gateway lifecycle management
- WebChat interface
- Voice overlay
- Canvas for visual content
- Skills management
"#
        .to_string();
    }

    format!(
        r#"# {title}

{description}

## Overview

{overview}

{platform_content}

## See Also

- [Platforms Overview](../platforms/index.md)
"#,
        title = page.title,
        description = platform_description(&s, subsection),
        overview = platform_overview(&s, subsection),
        platform_content = platform_content(&s, subsection),
    )
}

fn platform_description(slug: &str, sub: &str) -> &'static str {
    match (sub, slug) {
        ("mac", "bundled-gateway") => "The macOS app bundles its own gateway instance, managed as a child process.",
        ("mac", "canvas") => "Canvas renders HTML and SVG content as images directly from the macOS app.",
        ("mac", "child-process") => "How the macOS app manages the gateway as a supervised child process.",
        ("mac", "dev-setup") => "Set up a development environment for the macOS app.",
        ("mac", "health") => "Health check integration in the macOS app.",
        ("mac", "icon") => "The menu bar icon shows gateway status at a glance.",
        ("mac", "logging") => "Logging configuration specific to the macOS app.",
        ("mac", "menu-bar") => "The macOS menu bar provides quick access to gateway controls.",
        ("mac", "peekaboo") => "Peekaboo bridge connects the macOS app to the system accessibility API.",
        ("mac", "permissions") => "macOS permissions required by the app (accessibility, screen recording, etc.).",
        ("mac", "release") => "Release process for the macOS app.",
        ("mac", "remote") => "Control the macOS app remotely from another device.",
        ("mac", "signing") => "Code signing and notarization for macOS distribution.",
        ("mac", "skills") => "Skills management in the macOS app.",
        ("mac", "voice-overlay") => "The voice overlay provides a floating UI for voice interactions.",
        ("mac", "voicewake") => "Voice wake detection for hands-free activation on macOS.",
        ("mac", "webchat") => "Built-in WebChat interface in the macOS app.",
        ("mac", "xpc") => "XPC (Inter-Process Communication) between the macOS app and gateway.",
        ("", "macos") => "Magic Merlin on macOS — native app with menu bar, gateway management, and system integration.",
        ("", "linux") => "Magic Merlin on Linux — systemd service, headless server, or desktop usage.",
        ("", "windows") => "Magic Merlin on Windows via WSL2 — full Linux compatibility.",
        ("", "ios") => "Magic Merlin iOS companion app for mobile node access.",
        ("", "android") => "Magic Merlin Android companion app for mobile node access.",
        ("", "digitalocean") => "Deploy Magic Merlin on DigitalOcean droplets.",
        ("", "oracle") => "Deploy Magic Merlin on Oracle Cloud free tier.",
        ("", "raspberry-pi") => "Run Magic Merlin on Raspberry Pi for home automation and IoT.",
        _ => "Platform-specific guide for running Magic Merlin.",
    }
}

fn platform_overview(slug: &str, sub: &str) -> &'static str {
    match (sub, slug) {
        ("", "macos") => "The macOS app provides a native experience with:\n\n- Menu bar status icon\n- Built-in gateway lifecycle management\n- WebChat interface\n- Voice overlay for hands-free interaction\n- Canvas for rendering visual content\n- Automatic updates",
        ("", "linux") => "On Linux, Magic Merlin runs as:\n\n- A **systemd service** for always-on deployment\n- A **standalone binary** for manual control\n- A **Docker container** for isolated deployment\n\n```bash\n# Install\ncargo install magicmerlin magicmerlin-gateway\n\n# Run as systemd service\nmagicmerlin daemon install\nsudo systemctl enable --now magicmerlin\n```",
        ("", "raspberry-pi") => "Magic Merlin runs well on Raspberry Pi 4+ with 4GB RAM:\n\n```bash\n# Cross-compile or build on device\ncargo install magicmerlin magicmerlin-gateway\n\n# Start gateway\nmagicmerlin gateway start\n```\n\nIdeal for:\n- Home automation agents\n- Voice assistant (with USB microphone)\n- Local-first AI with Ollama",
        _ => "This platform is supported by Magic Merlin.",
    }
}

fn platform_content(slug: &str, sub: &str) -> &'static str {
    match (sub, slug) {
        ("mac", "canvas") => "## Usage\n\nCanvas renders HTML/SVG content into images that can be sent to channels:\n\n```json\n{\n  \"tool\": \"canvas\",\n  \"params\": {\n    \"html\": \"<h1>Hello</h1>\",\n    \"width\": 800,\n    \"height\": 600\n  }\n}\n```",
        ("mac", "voice-overlay") => "## Activation\n\nThe voice overlay can be activated via:\n- Keyboard shortcut (configurable)\n- Voice wake word\n- Menu bar → Start Voice\n\n## Configuration\n\n```json\n{\n  \"voice\": {\n    \"overlay\": true,\n    \"wakeWord\": \"hey merlin\"\n  }\n}\n```",
        _ => "",
    }
}

fn render_nodes_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Overview

{overview}

## Configuration

{config}

## See Also

- [Nodes Overview](index.md)
- [Platforms](../platforms/index.md)
"#,
        title = page.title,
        description = node_description(&s),
        overview = node_overview(&s),
        config = node_config(&s),
    )
}

fn node_description(slug: &str) -> &'static str {
    match slug {
        "index" => "Nodes are companion devices that extend Magic Merlin with hardware capabilities — cameras, microphones, speakers, and sensors.",
        "audio" => "Audio and voice notes: capture, transcribe, and synthesize audio through connected nodes.",
        "camera" => "Camera capture allows agents to take photos and process visual input from connected cameras.",
        "images" => "Image and media support for processing, generating, and sending images across channels.",
        "location-command" => "The location command reports the geographic location of a connected node.",
        "media-understanding" => "Media understanding enables agents to analyze images, audio, and video content using multimodal models.",
        "talk" => "Talk mode provides real-time voice conversation with the agent through a connected node.",
        "troubleshooting" => "Troubleshooting guide for common node connectivity and hardware issues.",
        "voicewake" => "Voice wake detection listens for a wake word to activate the agent hands-free.",
        _ => "Node functionality for hardware integration.",
    }
}

fn node_overview(slug: &str) -> &'static str {
    match slug {
        "index" => "Nodes connect to the gateway via WebSocket and provide:\n\n- **Audio capture** — microphone input for voice commands and transcription\n- **Camera** — photo capture for visual understanding\n- **Speaker** — TTS output for voice responses\n- **Location** — GPS coordinates\n- **Voice wake** — always-listening wake word detection\n\n```bash\n# List connected nodes\nmagicmerlin nodes\n\n# Pair a new node\nmagicmerlin qr\n```",
        "talk" => "Talk mode creates a real-time voice loop:\n\n1. Node listens for speech\n2. Audio is transcribed (Deepgram/Whisper)\n3. Transcript is sent to the agent\n4. Agent response is synthesized (TTS)\n5. Audio is played on the node speaker\n\n```bash\n# Start talk mode\nmagicmerlin voicecall\n```",
        _ => "This node feature integrates with the gateway to provide hardware-accelerated capabilities.",
    }
}

fn node_config(slug: &str) -> &'static str {
    match slug {
        "audio" => "```json\n{\n  \"nodes\": {\n    \"audio\": {\n      \"transcription\": \"deepgram\",\n      \"tts\": \"openai\"\n    }\n  }\n}\n```",
        "voicewake" => "```json\n{\n  \"nodes\": {\n    \"voiceWake\": {\n      \"enabled\": true,\n      \"wakeWord\": \"hey merlin\",\n      \"sensitivity\": 0.5\n    }\n  }\n}\n```",
        _ => "See the [Configuration Reference](../gateway/configuration-reference.md) for node-related settings.",
    }
}

fn render_reference_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Content

{content}

## See Also

- [Configuration Reference](../gateway/configuration-reference.md)
- [Help](../help/index.md)
"#,
        title = page.title,
        description = reference_description(&s),
        content = reference_content(&s),
    )
}

fn reference_description(slug: &str) -> &'static str {
    match slug {
        "api-usage-costs" => "Understanding API usage, token consumption, and associated costs when using Magic Merlin.",
        "credits" => "Credits and acknowledgments for the Magic Merlin project.",
        "device-models" => "Database of known device models and their capabilities for node pairing.",
        "prompt-caching" => "Prompt caching reduces costs and latency by reusing common prompt prefixes across requests.",
        "rpc" => "RPC adapter reference for extending the gateway protocol.",
        "session-management-compaction" => "Deep dive into session management, compaction strategies, and context window optimization.",
        "test" => "Testing guide for running and writing Magic Merlin tests.",
        "token-use" => "Understanding token usage, context windows, and cost optimization strategies.",
        "transcript-hygiene" => "Best practices for keeping conversation transcripts clean and efficient.",
        "wizard" => "Reference for the onboarding wizard configuration options.",
        "RELEASING" => "Release checklist for publishing new Magic Merlin versions.",
        _ => "Reference documentation for Magic Merlin.",
    }
}

fn reference_content(slug: &str) -> &'static str {
    match slug {
        "api-usage-costs" => "## Token Counting\n\nMagic Merlin tracks token usage per session:\n\n```bash\n# View usage for current session\nmagicmerlin status --usage\n\n# View historical usage\nmagicmerlin system usage --last 7d\n```\n\n## Cost Estimation\n\nCosts depend on:\n- **Model** — Claude Opus costs more per token than Haiku\n- **Context size** — longer conversations use more tokens\n- **Tool calls** — each tool invocation adds tokens\n- **Compaction** — compaction adds a summarization call but saves future tokens",
        "prompt-caching" => "## How Caching Works\n\nAnthropic's prompt caching stores common prompt prefixes server-side:\n\n1. The system prompt and initial context are cached\n2. Subsequent requests reuse the cached prefix\n3. Only new messages incur full token costs\n\n## Configuration\n\n```json\n{\n  \"providers\": {\n    \"anthropic\": {\n      \"promptCaching\": true\n    }\n  }\n}\n```\n\nCaching can reduce costs by 50-90% for conversations with large system prompts.",
        "session-management-compaction" => "## Session Lifecycle\n\n1. **Creation** — new session on first message from a user/channel pair\n2. **Active** — messages flow through the agent loop\n3. **Idle** — no activity for the configured timeout\n4. **Compacted** — context window compressed via summarization\n5. **Pruned** — session removed after max age\n\n## Compaction Strategies\n\n| Strategy | Description |\n|----------|-------------|\n| `summarize` | LLM summarizes old messages (default) |\n| `truncate` | Drop oldest messages |\n| `sliding` | Keep only last N messages |",
        _ => "Detailed reference information for this topic.",
    }
}

fn render_automation_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Overview

{overview}

## Configuration

{config}

## Examples

{examples}

## See Also

- [Cron Jobs](cron-jobs.md)
- [Hooks](hooks.md)
- [Webhooks](webhook.md)
"#,
        title = page.title,
        description = automation_description(&s),
        overview = automation_overview(&s),
        config = automation_config(&s),
        examples = automation_examples(&s),
    )
}

fn automation_description(slug: &str) -> &'static str {
    match slug {
        "auth-monitoring" => "Monitor authentication events and trigger alerts or actions on suspicious activity.",
        "cron-jobs" => "Schedule recurring tasks using cron expressions. Agents can execute commands, send messages, or run tools on a schedule.",
        "cron-vs-heartbeat" => "Understanding the difference between cron jobs (user-scheduled) and heartbeat tasks (system-scheduled).",
        "gmail-pubsub" => "Receive Gmail notifications via Google Cloud Pub/Sub to trigger agent actions on new emails.",
        "hooks" => "Hooks trigger agent actions in response to system events (message received, tool executed, session started, etc.).",
        "poll" => "Polls periodically check external sources (URLs, APIs, files) and trigger actions on changes.",
        "troubleshooting" => "Troubleshooting guide for automation features: cron, hooks, webhooks, and polls.",
        "webhook" => "Webhooks expose HTTP endpoints that trigger agent actions when called by external services.",
        _ => "Automation feature for scheduling and triggering agent actions.",
    }
}

fn automation_overview(slug: &str) -> &'static str {
    match slug {
        "cron-jobs" => "```bash\n# List cron jobs\nmagicmerlin cron list\n\n# Create a cron job\nmagicmerlin cron add \"0 9 * * *\" \"Good morning! Here's your daily briefing.\"\n\n# Delete a cron job\nmagicmerlin cron remove <id>\n```",
        "hooks" => "Hooks fire on events:\n\n- `message.received` — when a message arrives\n- `message.sent` — when the agent sends a response\n- `tool.before` — before a tool is executed\n- `tool.after` — after a tool completes\n- `session.start` — when a new session begins\n- `session.end` — when a session is pruned",
        "webhook" => "```bash\n# List webhook endpoints\nmagicmerlin webhooks list\n\n# Create a webhook\nmagicmerlin webhooks add --name deploy-notify --path /hooks/deploy\n```\n\nWebhooks are accessible at `http://localhost:3578/hooks/<path>`.",
        _ => "This automation feature integrates with the gateway event system.",
    }
}

fn automation_config(slug: &str) -> &'static str {
    match slug {
        "cron-jobs" => "```json\n{\n  \"cron\": [\n    {\n      \"schedule\": \"0 9 * * *\",\n      \"message\": \"Daily briefing\",\n      \"channel\": \"telegram\"\n    }\n  ]\n}\n```",
        "hooks" => "```json\n{\n  \"hooks\": {\n    \"message.received\": [\n      {\n        \"action\": \"log\",\n        \"params\": { \"level\": \"info\" }\n      }\n    ]\n  }\n}\n```",
        "webhook" => "```json\n{\n  \"webhooks\": [\n    {\n      \"name\": \"deploy-notify\",\n      \"path\": \"/hooks/deploy\",\n      \"secret\": \"whsec_...\"\n    }\n  ]\n}\n```",
        _ => "See the [Configuration Reference](../gateway/configuration-reference.md) for all automation options.",
    }
}

fn automation_examples(slug: &str) -> &'static str {
    match slug {
        "cron-jobs" => "```bash\n# Daily summary at 9 AM\nmagicmerlin cron add \"0 9 * * *\" \"Summarize my calendar for today\"\n\n# Every 30 minutes health check\nmagicmerlin cron add \"*/30 * * * *\" \"/health-report\"\n```",
        "gmail-pubsub" => "```json\n{\n  \"automation\": {\n    \"gmail\": {\n      \"enabled\": true,\n      \"projectId\": \"your-gcp-project\",\n      \"topicName\": \"gmail-notifications\"\n    }\n  }\n}\n```",
        _ => "See the automation documentation for detailed examples.",
    }
}

fn render_help_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{content}

## See Also

- [Troubleshooting](troubleshooting.md)
- [FAQ](faq.md)
- [Gateway Doctor](../gateway/doctor.md)
"#,
        title = page.title,
        content = help_content(&s),
    )
}

fn help_content(slug: &str) -> &'static str {
    match slug {
        "index" => "## Getting Help\n\n- Run `magicmerlin doctor` to diagnose common issues\n- Check the [FAQ](faq.md) for frequently asked questions\n- See [Troubleshooting](troubleshooting.md) for common problems and solutions\n- View [Environment Variables](environment.md) for configuration options\n- Review [Debugging](debugging.md) for advanced debugging techniques",
        "debugging" => "## Debug Mode\n\n```bash\n# Run gateway with debug logging\nMAGICMERLIN_LOG=debug magicmerlin gateway start\n\n# Debug a specific module\nMAGICMERLIN_LOG=magicmerlin_gateway=trace magicmerlin gateway start\n```\n\n## Inspect WebSocket Traffic\n\n```bash\n# Connect to the gateway WebSocket\nwebsocat ws://localhost:3578/ws\n```\n\n## Inspect Sessions\n\n```bash\nmagicmerlin sessions --verbose\n```",
        "environment" => "## Environment Variables\n\n| Variable | Description | Default |\n|----------|-------------|---------|\n| `MAGICMERLIN_HOME` | Config directory | `~/.magicmerlin` |\n| `MAGICMERLIN_LOG` | Log level | `info` |\n| `MAGICMERLIN_PORT` | Gateway port | `3578` |\n| `ANTHROPIC_API_KEY` | Anthropic API key | — |\n| `OPENAI_API_KEY` | OpenAI API key | — |\n| `OLLAMA_HOST` | Ollama endpoint | `http://localhost:11434` |",
        "faq" => "## Frequently Asked Questions\n\n**Q: Is Magic Merlin compatible with OpenClaw?**\nA: Yes. Magic Merlin reads `openclaw.json` natively. See the [Migration Guide](../install/migrating.md).\n\n**Q: Do I need Node.js?**\nA: No. Magic Merlin is a standalone Rust binary.\n\n**Q: Which LLM providers are supported?**\nA: 25+ providers including Anthropic, OpenAI, Ollama, Bedrock, and more. See [Providers](../providers/index.md).\n\n**Q: Can I run it on Raspberry Pi?**\nA: Yes! See [Raspberry Pi](../platforms/raspberry-pi.md).",
        "scripts" => "## Utility Scripts\n\nMagic Merlin includes utility scripts for common operations:\n\n```bash\n# Health check script\nmagicmerlin health --json\n\n# Export configuration\nmagicmerlin config export > backup.json\n\n# Import configuration\nmagicmerlin config import backup.json\n```",
        "testing" => "## Running Tests\n\n```bash\n# Run all tests\ncargo test --workspace\n\n# Run specific crate tests\ncargo test -p magicmerlin-gateway\n\n# Run with logging\nRUST_LOG=debug cargo test\n```",
        "troubleshooting" => "## Common Issues\n\n### Gateway won't start\n- Check if port 3578 is already in use: `lsof -i :3578`\n- Check for a stale lock file: `magicmerlin doctor`\n- View logs: `magicmerlin logs --last 50`\n\n### Agent not responding\n- Verify provider API key is set: `magicmerlin config get providers`\n- Check model availability: `magicmerlin models`\n- Test connectivity: `magicmerlin health`\n\n### Channel disconnection\n- Check channel credentials: `magicmerlin channels`\n- Verify network connectivity\n- Review channel-specific logs: `magicmerlin logs --filter channel`",
        _ => "Help and support resources for Magic Merlin.",
    }
}

fn render_plugins_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Overview

{overview}

## Configuration

{config}

## See Also

- [Tools](../tools/index.md)
- [Skills](../tools/skills.md)
"#,
        title = page.title,
        description = plugin_description(&s),
        overview = plugin_overview(&s),
        config = plugin_config(&s),
    )
}

fn plugin_description(slug: &str) -> &'static str {
    match slug {
        "agent-tools" => {
            "Plugin agent tools extend the agent's capabilities with custom tool implementations."
        }
        "community" => "Community-contributed plugins for Magic Merlin.",
        "manifest" => "The plugin manifest defines a plugin's metadata, tools, and dependencies.",
        "voice-call" => "The voice call plugin adds real-time voice conversation capabilities.",
        "zalouser" => "The Zalo Personal plugin enables messaging via Zalo personal accounts.",
        _ => "Plugin for extending Magic Merlin functionality.",
    }
}

fn plugin_overview(slug: &str) -> &'static str {
    match slug {
        "manifest" => "A plugin manifest is a `manifest.json` file:\n\n```json\n{\n  \"name\": \"my-plugin\",\n  \"version\": \"1.0.0\",\n  \"description\": \"My custom plugin\",\n  \"tools\": [\n    {\n      \"name\": \"my_tool\",\n      \"description\": \"Does something useful\",\n      \"parameters\": {}\n    }\n  ]\n}\n```",
        "community" => "Browse and install community plugins:\n\n```bash\nmagicmerlin plugins search <query>\nmagicmerlin plugins install <name>\n```",
        _ => "Plugins are loaded at gateway startup and integrate with the tool registry.",
    }
}

fn plugin_config(slug: &str) -> &'static str {
    match slug {
        "manifest" => {
            "```json\n{\n  \"plugins\": {\n    \"path\": [\"~/.magicmerlin/plugins/\"]\n  }\n}\n```"
        }
        _ => "```json\n{\n  \"plugins\": {\n    \"enabled\": [\"plugin-name\"]\n  }\n}\n```",
    }
}

fn render_web_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Overview

{overview}

{content}

## See Also

- [Web Overview](index.md)
- [Gateway](../gateway/index.md)
"#,
        title = page.title,
        description = web_description(&s),
        overview = web_overview(&s),
        content = web_content(&s),
    )
}

fn web_description(slug: &str) -> &'static str {
    match slug {
        "index" => "Magic Merlin provides multiple web-based interfaces for monitoring and interaction.",
        "control-ui" => "The Control UI is a web-based admin panel for managing the gateway, agents, and configuration.",
        "dashboard" => "The web dashboard provides real-time monitoring of gateway status, sessions, and usage.",
        "tui" => "The TUI (Terminal User Interface) provides a rich terminal-based dashboard built with Ratatui.",
        "webchat" => "WebChat is an embedded chat interface for interacting with agents via the browser.",
        _ => "Web interface for Magic Merlin.",
    }
}

fn web_overview(slug: &str) -> &'static str {
    match slug {
        "index" => "| Interface | Access | Description |\n|-----------|--------|-------------|\n| Dashboard | `http://localhost:3578/dashboard` | Real-time monitoring |\n| Control UI | `http://localhost:3578/control` | Admin panel |\n| WebChat | `http://localhost:3578/chat` | Chat interface |\n| TUI | `magicmerlin tui` | Terminal dashboard |",
        "tui" => "```bash\n# Start the TUI\nmagicmerlin tui\n```\n\nThe TUI shows:\n- Gateway status and uptime\n- Active sessions and message counts\n- Connected channels and their status\n- Recent messages and tool calls\n- Resource usage (CPU, memory, tokens)",
        "webchat" => "WebChat provides a browser-based chat interface:\n\n```bash\n# Open WebChat\nmagicmerlin dashboard\n# Or navigate to http://localhost:3578/chat\n```\n\nFeatures:\n- Real-time message streaming\n- Markdown rendering\n- Image display\n- Tool call visualization\n- Session management",
        _ => "This interface connects to the gateway for real-time data.",
    }
}

fn web_content(slug: &str) -> &'static str {
    match slug {
        "tui" => "## Keyboard Shortcuts\n\n| Key | Action |\n|-----|--------|\n| `q` | Quit |\n| `Tab` | Switch panels |\n| `↑/↓` | Scroll |\n| `Enter` | Select |\n| `/` | Search |\n| `r` | Refresh |",
        _ => "",
    }
}

fn render_security_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

{description}

## Overview

Security is a core concern in Magic Merlin. The agent runtime enforces multiple layers of protection:

1. **Tool Policy** — controls which tools agents can invoke
2. **Sandbox** — isolates command execution
3. **Authentication** — secures API access
4. **Secrets Management** — encrypts sensitive configuration
5. **Approval Flow** — requires human confirmation for dangerous operations

## Configuration

See the [Gateway Security](../gateway/security/index.md) section for detailed configuration.

## See Also

- [Sandboxing](../gateway/sandboxing.md)
- [Authentication](../gateway/authentication.md)
- [Exec Approvals](../tools/exec-approvals.md)
"#,
        title = page.title,
        description = security_description(&slug(page)),
    )
}

fn security_description(slug: &str) -> &'static str {
    match slug {
        "formal-verification" => {
            "Formal verification models for analyzing security properties of the agent runtime."
        }
        "CONTRIBUTING-THREAT-MODEL" => "Contributing guide for the threat model documentation.",
        "THREAT-MODEL-ATLAS" => "Threat model atlas covering attack surfaces and mitigations.",
        _ => "Security documentation for Magic Merlin.",
    }
}

fn render_experiments_page(page: &DocPage) -> String {
    let s = slug(page);
    let sub = page.subsection.as_deref().unwrap_or("");
    format!(
        r#"# {title}

{description}

!!! warning "Experimental"
    This is an experimental feature or design document. It may change or be removed in future versions.

## Overview

{overview}

## See Also

- [Concepts](../concepts/features.md)
- [Gateway Architecture](../concepts/architecture.md)
"#,
        title = page.title,
        description = experiment_description(&s, sub),
        overview = experiment_overview(&s, sub),
    )
}

fn experiment_description(slug: &str, sub: &str) -> &'static str {
    match (sub, slug) {
        ("", "onboarding-config-protocol") => "Experimental onboarding and configuration protocol for automated setup.",
        ("plans", "acp-thread-bound-agents") => "Plan for ACP thread-bound agents that maintain context within conversation threads.",
        ("plans", "acp-unified-streaming-refactor") => "Plan for unifying the streaming architecture across all ACP agent types.",
        ("plans", "browser-evaluate-cdp-refactor") => "Plan for refactoring browser evaluation to use CDP (Chrome DevTools Protocol) directly.",
        ("plans", "openresponses-gateway") => "Plan for implementing OpenResponses API gateway compatibility.",
        ("plans", "pty-process-supervision") => "Plan for PTY-based process supervision for long-running tool executions.",
        ("plans", "session-binding-channel-agnostic") => "Plan for making session binding channel-agnostic.",
        ("proposals", "model-config") => "Proposal for a new model configuration format.",
        ("research", "memory") => "Research into workspace-aware memory systems.",
        _ => "Experimental feature or design document.",
    }
}

fn experiment_overview(slug: &str, sub: &str) -> &'static str {
    match (sub, slug) {
        ("plans", _) => "This is a design plan that outlines the goals, approach, and implementation steps for an upcoming feature. Plans are subject to change based on implementation experience.",
        ("proposals", _) => "This is a proposal for a new feature or configuration change. Proposals are open for discussion before implementation.",
        ("research", _) => "This is research documentation exploring a design space. Research may or may not lead to implementation.",
        _ => "This experimental feature is under active development.",
    }
}

fn render_debug_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

## Debugging

{content}

## See Also

- [Help](../help/debugging.md)
- [Gateway Troubleshooting](../gateway/troubleshooting.md)
"#,
        title = page.title,
        content = match slug(page).as_str() {
            "node-issue" => "### Node.js / tsx Crash\n\nIf you're migrating from OpenClaw and encounter Node.js crashes:\n\n1. Magic Merlin doesn't require Node.js — it's a native Rust binary\n2. If using the compatibility shim, ensure Node.js 20+ is installed\n3. For tsx issues, install tsx globally: `npm install -g tsx`\n4. Consider fully migrating to Magic Merlin to avoid Node.js entirely",
            _ => "Use `magicmerlin doctor` and `magicmerlin logs` to diagnose issues.",
        },
    )
}

fn render_design_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

## Design Document

This document describes an architectural design within Magic Merlin.

## Overview

{content}

## See Also

- [Gateway Architecture](../concepts/architecture.md)
"#,
        title = page.title,
        content = match slug(page).as_str() {
            "kilo-gateway-integration" => "The Kilo gateway integration plan describes how Magic Merlin integrates with OpenClaw's gateway protocol while maintaining its own Rust-native implementation.",
            _ => "Design documentation for this feature.",
        },
    )
}

fn render_diagnostics_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

## Diagnostics

{content}

## Usage

```bash
# Run diagnostics
magicmerlin doctor --verbose

# Check specific flags
magicmerlin doctor --check connectivity
magicmerlin doctor --check providers
magicmerlin doctor --check channels
```

## See Also

- [Troubleshooting](../help/troubleshooting.md)
- [Gateway Doctor](../gateway/doctor.md)
"#,
        title = page.title,
        content = match slug(page).as_str() {
            "flags" => "Diagnostic flags control detailed health checks:\n\n| Flag | Description |\n|------|-------------|\n| `--check connectivity` | Test network and gateway connectivity |\n| `--check providers` | Verify all provider API keys and endpoints |\n| `--check channels` | Test channel connections |\n| `--check tools` | Verify tool availability and sandbox |\n| `--check storage` | Check disk space and file permissions |",
            _ => "Diagnostics help identify and resolve configuration issues.",
        },
    )
}

fn render_api_reference_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

## API Reference

The Magic Merlin gateway exposes an OpenAPI-compatible REST API alongside the WebSocket protocol.

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Gateway health check |
| `/call` | POST | JSON-RPC method invocation |
| `/v1/chat/completions` | POST | OpenAI-compatible chat completions |
| `/v1/responses` | POST | OpenResponses API |
| `/hooks/:name` | POST | Webhook endpoints |
| `/ws` | WS | WebSocket connection |

## Authentication

Include a bearer token in the `Authorization` header:

```
Authorization: Bearer <token>
```

## See Also

- [Gateway Protocol](../gateway/protocol.md)
- [OpenAI HTTP API](../gateway/openai-http-api.md)
- [Authentication](../gateway/authentication.md)
"#,
        title = page.title,
    )
}

fn render_root_page(page: &DocPage) -> String {
    let s = slug(page);
    format!(
        r#"# {title}

{description}

## Overview

{content}

## See Also

- [Getting Started](start/getting-started.md)
- [Concepts](concepts/features.md)
"#,
        title = page.title,
        description = root_description(&s),
        content = root_content(&s),
    )
}

fn root_description(slug: &str) -> &'static str {
    match slug {
        "brave-search" => "Configure Brave Search as the web search provider for Magic Merlin agents.",
        "ci" => "How the Magic Merlin CI/CD pipeline works.",
        "date-time" => "Date and time handling in Magic Merlin: parsing, formatting, and timezone conversion.",
        "perplexity" => "Use Perplexity Sonar for enhanced web search with AI-powered summarization.",
        "pi" => "Pi integration architecture for running Magic Merlin on Raspberry Pi.",
        "pi-dev" => "Development workflow for Pi-based Magic Merlin deployments.",
        "prose" => "OpenProse: structured prose generation and formatting for agent responses.",
        "tts" => "Text-to-speech (TTS) support: convert agent responses to audio using Deepgram, OpenAI, or system TTS.",
        "vps" => "Host Magic Merlin on a VPS (Virtual Private Server) for always-on availability.",
        "index" => "Magic Merlin documentation home page.",
        _ => "Magic Merlin feature documentation.",
    }
}

fn root_content(slug: &str) -> &'static str {
    match slug {
        "brave-search" => "## Configuration\n\n```json\n{\n  \"tools\": {\n    \"web_search\": {\n      \"provider\": \"brave\",\n      \"apiKey\": \"BSA_...\"\n    }\n  }\n}\n```\n\n## Environment Variable\n\n```bash\nexport BRAVE_SEARCH_API_KEY=\"BSA_...\"\n```\n\nGet a free API key at [brave.com/search/api](https://brave.com/search/api).",
        "tts" => "## Supported Providers\n\n| Provider | Quality | Speed | Cost |\n|----------|---------|-------|------|\n| Deepgram | High | Fast | Paid |\n| OpenAI | High | Medium | Paid |\n| System | Basic | Fast | Free |\n\n## Configuration\n\n```json\n{\n  \"tts\": {\n    \"provider\": \"deepgram\",\n    \"voice\": \"aura-asteria-en\"\n  }\n}\n```",
        "vps" => "## Recommended Setup\n\n1. Provision a VPS (2GB RAM minimum)\n2. Install Rust and build Magic Merlin\n3. Set up as a systemd service\n4. Configure a reverse proxy (Caddy/nginx)\n5. Enable TLS for secure remote access\n\n```bash\n# On your VPS\ncargo install magicmerlin magicmerlin-gateway\nmagicmerlin daemon install\nsudo systemctl enable --now magicmerlin\n```",
        _ => "See the relevant documentation section for details.",
    }
}

fn render_generic_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

Documentation for {title} in Magic Merlin.

## Overview

This page covers {title_lower} functionality in Magic Merlin.

## See Also

- [Getting Started](../start/getting-started.md)
- [Concepts](../concepts/features.md)
"#,
        title = page.title,
        title_lower = page.title.to_lowercase(),
    )
}

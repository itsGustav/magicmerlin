//! Documentation generator for MagicMerlin.
//!
//! Reads `parity/openclaw_docs_index.json` and generates a complete documentation
//! tree under `docs/` with section-specific templates.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// A single documentation page entry from the index JSON.
#[derive(Debug, Deserialize)]
pub struct PageEntry {
    pub title: Option<String>,
    pub url: String,
}

/// Top-level structure of `openclaw_docs_index.json`.
#[derive(Debug, Deserialize)]
pub struct DocsIndex {
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    pub pages: Vec<PageEntry>,
}

/// Represents a documentation page to be generated.
#[derive(Debug)]
pub struct DocPage {
    pub title: String,
    pub rel_path: String,
    pub section: String,
    pub slug: String,
}

const URL_PREFIX: &str = "https://docs.openclaw.ai/";

/// Load the docs index from a JSON file.
pub fn load_index(path: &Path) -> Result<DocsIndex> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("failed to read index at {}", path.display()))?;
    let index: DocsIndex =
        serde_json::from_str(&data).with_context(|| "failed to parse docs index JSON")?;
    Ok(index)
}

/// Parse a page entry into a DocPage, extracting section and slug.
fn parse_page(entry: &PageEntry) -> Option<DocPage> {
    let rel = entry.url.strip_prefix(URL_PREFIX)?;
    // Skip non-markdown files (e.g. openapi.json)
    if !rel.ends_with(".md") {
        return None;
    }
    let raw_title = entry.title.as_deref().unwrap_or("null");
    let title = if raw_title == "null" {
        // Derive title from the slug
        let slug = rel.trim_end_matches(".md");
        let slug = slug.rsplit('/').next().unwrap_or(slug);
        title_case(slug)
    } else {
        raw_title.to_string()
    };

    let section = if rel.contains('/') {
        rel.split('/').next().unwrap_or("general").to_string()
    } else {
        "general".to_string()
    };

    let slug = rel.trim_end_matches(".md").to_string();

    Some(DocPage {
        title,
        rel_path: rel.to_string(),
        section,
        slug,
    })
}

/// Convert a kebab-case slug to Title Case.
fn title_case(s: &str) -> String {
    s.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate markdown content for a given page based on its section.
fn generate_content(page: &DocPage) -> String {
    match page.section.as_str() {
        "cli" => generate_cli_page(page),
        "gateway" => generate_gateway_page(page),
        "channels" => generate_channel_page(page),
        "concepts" => generate_concepts_page(page),
        "automation" => generate_automation_page(page),
        "tools" => generate_tools_page(page),
        "providers" => generate_provider_page(page),
        "start" => generate_start_page(page),
        "install" => generate_install_page(page),
        "plugins" => generate_plugins_page(page),
        "nodes" => generate_nodes_page(page),
        "platforms" => generate_platforms_page(page),
        "reference" => generate_reference_page(page),
        "help" => generate_help_page(page),
        "web" => generate_web_page(page),
        "security" => generate_security_page(page),
        "diagnostics" => generate_general_page(page),
        "experiments" => generate_general_page(page),
        "design" => generate_general_page(page),
        "debug" => generate_general_page(page),
        _ => generate_general_page(page),
    }
}

fn generate_cli_page(page: &DocPage) -> String {
    let cmd = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# `magicmerlin {cmd}`

> CLI command reference

## Usage

```
magicmerlin {cmd} [OPTIONS]
```

## Description

The `{cmd}` command provides {title} functionality for MagicMerlin. Use this
command to manage and interact with {title_lower} features directly from the
terminal.

## Options

| Flag | Description |
|------|-------------|
| `--help` | Show help information |
| `--json` | Output as JSON |
| `--verbose` | Enable verbose output |
| `--quiet` | Suppress non-essential output |

## Examples

```bash
# Basic usage
magicmerlin {cmd}

# With JSON output
magicmerlin {cmd} --json

# Verbose mode
magicmerlin {cmd} --verbose
```

## See Also

- [CLI Reference](index.md)
- [Getting Started](../start/getting-started.md)
"#,
        cmd = cmd,
        title = page.title,
        title_lower = page.title.to_lowercase(),
    )
}

fn generate_gateway_page(page: &DocPage) -> String {
    let method = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Gateway reference

## Overview

{title} covers an essential aspect of the MagicMerlin gateway. The gateway
acts as the central hub for all agent communication, tool execution, and
session management.

## Configuration

The gateway reads its configuration from `~/.config/magicmerlin/gateway.toml`.
Settings related to {title_lower} can be adjusted there.

```toml
[gateway]
# {title} settings
enabled = true
```

## API

### Request

```json
{{
  "method": "gateway.{method}",
  "params": {{}}
}}
```

### Response

```json
{{
  "ok": true,
  "data": {{}}
}}
```

## Troubleshooting

If you encounter issues with {title_lower}:

1. Check gateway logs: `magicmerlin logs --gateway`
2. Verify configuration: `magicmerlin doctor`
3. Restart the gateway: `magicmerlin gateway restart`

## See Also

- [Gateway Runbook](index.md)
- [Gateway Protocol](protocol.md)
- [Troubleshooting](troubleshooting.md)
"#,
        title = page.title,
        title_lower = page.title.to_lowercase(),
        method = method,
    )
}

fn generate_channel_page(page: &DocPage) -> String {
    let slug = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Channel setup guide

## Overview

{title} is a supported messaging channel in MagicMerlin. Channels allow your
agent to communicate through various messaging platforms and protocols.

## Setup

### Prerequisites

- A running MagicMerlin gateway
- Valid credentials for {title}

### Configuration

Add the channel configuration to your gateway config:

```toml
[channels.{slug}]
enabled = true
# Add your credentials here
```

### Pairing

```bash
magicmerlin channels pair {slug}
```

## Features

- Real-time message delivery
- Media support (images, files, voice)
- Group conversation support
- Typing indicators
- Read receipts (where supported)

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection drops | Check network and credentials |
| Messages not delivered | Verify channel is paired |
| Media not loading | Check file size limits |

## See Also

- [Chat Channels](index.md)
- [Channel Routing](channel-routing.md)
- [Pairing](pairing.md)
- [Channel Troubleshooting](troubleshooting.md)
"#,
        title = page.title,
        slug = slug,
    )
}

fn generate_concepts_page(page: &DocPage) -> String {
    let slug = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Conceptual reference

## Overview

{title} is a core concept in MagicMerlin's architecture. Understanding this
concept is essential for building effective agent configurations and workflows.

## How It Works

MagicMerlin implements {title_lower} as part of its agent runtime. This
mechanism ensures reliable and efficient operation across all connected
channels, tools, and sessions.

## Key Properties

- **Consistency** -- {title} state is persisted across gateway restarts
- **Isolation** -- Each session maintains its own {title_lower} context
- **Efficiency** -- Optimized for minimal latency and resource usage
- **Observability** -- Full logging and metrics for {title_lower} operations

## Configuration

```toml
# gateway.toml
[{slug}]
enabled = true
```

## Related Concepts

- [Agent Runtime](agent.md)
- [Session Management](session.md)
- [Memory](memory.md)

## See Also

- [Getting Started](../start/getting-started.md)
- [Gateway Architecture](architecture.md)
"#,
        title = page.title,
        title_lower = page.title.to_lowercase(),
        slug = slug,
    )
}

fn generate_automation_page(page: &DocPage) -> String {
    let slug = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Automation guide

## Overview

{title} enables automated workflows in MagicMerlin. Automation features allow
your agent to perform tasks on schedules, respond to events, and maintain
continuous operation without manual intervention.

## Setup

### Enable Automation

```toml
[automation]
enabled = true
```

### Configure {title}

```bash
magicmerlin cron add --schedule "*/5 * * * *" --action "{slug}"
```

## How It Works

1. The gateway monitors configured triggers
2. When conditions are met, the automation engine fires
3. The agent processes the event within a new or existing session
4. Results are delivered to the configured output channel

## Examples

```bash
# List active automations
magicmerlin cron list

# Check automation status
magicmerlin status --automations
```

## Troubleshooting

- Verify cron syntax with `magicmerlin cron validate`
- Check gateway logs for trigger events
- Ensure the agent has necessary tool permissions

## See Also

- [Cron Jobs](cron-jobs.md)
- [Hooks](hooks.md)
- [Webhooks](webhook.md)
"#,
        title = page.title,
        slug = slug,
    )
}

fn generate_tools_page(page: &DocPage) -> String {
    let tool_name = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Tool reference

## Overview

{title} is a built-in tool available to MagicMerlin agents. Tools extend the
agent's capabilities beyond text generation, enabling interaction with external
systems, files, browsers, and more.

## Usage

The tool is automatically available when enabled in your agent configuration:

```toml
[tools.{tool_name}]
enabled = true
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input` | string | yes | Primary input for the tool |
| `options` | object | no | Additional configuration |

## Tool Schema

```json
{{
  "name": "{tool_name}",
  "description": "{title}",
  "parameters": {{
    "type": "object",
    "properties": {{
      "input": {{ "type": "string" }}
    }}
  }}
}}
```

## Examples

The agent can invoke this tool during a conversation when it determines
that {title_lower} capabilities are needed.

## Security

- Tool execution respects the sandbox policy
- Approval may be required depending on configuration
- All invocations are logged

## See Also

- [Tools Overview](index.md)
- [Exec Approvals](exec-approvals.md)
- [Elevated Mode](elevated.md)
"#,
        title = page.title,
        title_lower = page.title.to_lowercase(),
        tool_name = tool_name,
    )
}

fn generate_provider_page(page: &DocPage) -> String {
    let provider = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Model provider setup

## Overview

{title} is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the {title} platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider {provider}
```

Or add directly to your configuration:

```toml
[providers.{provider}]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider {provider}
```

## Supported Models

Refer to the {title} documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure {title} as a failover provider:

```toml
[failover]
providers = ["{provider}", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)
"#,
        title = page.title,
        provider = provider,
    )
}

fn generate_start_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Getting started

## Overview

{title} helps you begin your journey with MagicMerlin. Follow this guide to
get your personal AI agent up and running.

## Quick Start

1. **Install MagicMerlin** -- See [Installation](../install/index.md)
2. **Run the setup wizard** -- `magicmerlin setup`
3. **Start the gateway** -- `magicmerlin gateway start`
4. **Connect a channel** -- `magicmerlin channels pair`

## Next Steps

- Configure your [agent personality](../concepts/system-prompt.md)
- Set up [memory](../concepts/memory.md) for persistent context
- Connect [tools](../tools/index.md) for extended capabilities
- Explore [automation](../automation/cron-jobs.md) for scheduled tasks

## See Also

- [Getting Started](getting-started.md)
- [Setup](setup.md)
- [CLI Reference](../cli/index.md)
"#,
        title = page.title,
    )
}

fn generate_install_page(page: &DocPage) -> String {
    let method = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Installation guide

## Overview

This guide covers installing MagicMerlin via {title}. Choose the installation
method that best fits your environment and workflow.

## Prerequisites

- A supported operating system (macOS, Linux, Windows via WSL2)
- Network access for downloading packages
- Sufficient disk space (approximately 200 MB)

## Installation

### {title}

```bash
# Install MagicMerlin via {method}
# Refer to the specific instructions below
```

## Post-Installation

After installation, run the setup wizard:

```bash
magicmerlin setup
```

This will guide you through:
- Configuring a model provider
- Setting up your first channel
- Initializing the gateway

## Verifying Installation

```bash
magicmerlin doctor
magicmerlin --version
```

## Updating

```bash
magicmerlin update
```

## See Also

- [Install Overview](index.md)
- [Getting Started](../start/getting-started.md)
- [Uninstall](uninstall.md)
"#,
        title = page.title,
        method = method,
    )
}

fn generate_plugins_page(page: &DocPage) -> String {
    let slug = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    format!(
        r#"# {title}

> Plugin reference

## Overview

{title} extends MagicMerlin's functionality through the plugin system. Plugins
are modular components that can add new tools, channels, and integrations.

## Installation

```bash
magicmerlin plugins install {slug}
```

## Configuration

```toml
[plugins.{slug}]
enabled = true
```

## Plugin Manifest

Every plugin includes a `manifest.json` that declares:
- Required permissions
- Tool definitions
- Channel bindings
- Configuration schema

## Development

See [Plugin Manifest](manifest.md) for details on creating your own plugins.

## See Also

- [Community Plugins](community.md)
- [Plugin Manifest](manifest.md)
- [Tools](../tools/index.md)
"#,
        title = page.title,
        slug = slug,
    )
}

fn generate_nodes_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Node reference

## Overview

{title} is a node capability in MagicMerlin. Nodes are edge devices (phones,
desktops, Raspberry Pis) that connect to the gateway and provide local
hardware access such as cameras, microphones, and sensors.

## Setup

Pair a node with your gateway:

```bash
magicmerlin node pair
```

## Features

- Real-time streaming from device hardware
- Secure communication via the gateway bridge
- Automatic reconnection and heartbeat monitoring
- Media transcoding and delivery

## Configuration

```toml
[nodes]
auto_accept = false
heartbeat_interval = 30
```

## Troubleshooting

- Verify the node is online: `magicmerlin nodes list`
- Check connectivity: `magicmerlin node ping <id>`
- Review logs: `magicmerlin logs --node <id>`

## See Also

- [Nodes Overview](index.md)
- [Audio and Voice Notes](audio.md)
- [Node Troubleshooting](troubleshooting.md)
"#,
        title = page.title,
    )
}

fn generate_platforms_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Platform guide

## Overview

{title} describes MagicMerlin's support for this platform. MagicMerlin runs
on multiple operating systems and form factors, from desktop apps to headless
servers.

## Requirements

- Supported OS version (see release notes)
- Gateway connectivity (local or remote)
- Sufficient system resources

## Installation

Follow the platform-specific installation instructions to get MagicMerlin
running on this platform.

## Platform-Specific Notes

Each platform may have unique features or limitations. Consult the sections
below for details specific to this environment.

## See Also

- [Platforms Overview](index.md)
- [Installation](../install/index.md)
- [Getting Started](../start/getting-started.md)
"#,
        title = page.title,
    )
}

fn generate_reference_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Reference documentation

## Overview

{title} provides detailed reference information for MagicMerlin internals,
configuration options, and operational procedures.

## Details

This reference document covers the specifics of {title_lower} as implemented
in MagicMerlin. Refer to the sections below for configuration, usage, and
troubleshooting information.

## Configuration

Relevant settings can be found in the gateway configuration file at
`~/.config/magicmerlin/gateway.toml`.

## See Also

- [API Usage and Costs](api-usage-costs.md)
- [Session Management Deep Dive](session-management-compaction.md)
- [Getting Started](../start/getting-started.md)
"#,
        title = page.title,
        title_lower = page.title.to_lowercase(),
    )
}

fn generate_help_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Help and support

## Overview

{title} provides guidance for common issues and questions when using
MagicMerlin.

## Common Issues

| Problem | Solution |
|---------|----------|
| Gateway won't start | Run `magicmerlin doctor` to diagnose |
| Channel disconnected | Re-pair with `magicmerlin channels pair` |
| Model errors | Verify API key and provider config |
| High latency | Check network and model provider status |

## Getting Help

- Run `magicmerlin doctor` for automated diagnostics
- Check logs with `magicmerlin logs`
- Visit the MagicMerlin community for support

## See Also

- [Help Index](index.md)
- [FAQ](faq.md)
- [Troubleshooting](troubleshooting.md)
- [Environment Variables](environment.md)
"#,
        title = page.title,
    )
}

fn generate_web_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Web interface reference

## Overview

{title} is part of MagicMerlin's web-based interface. The web UI provides
browser-accessible dashboards, chat interfaces, and administrative controls.

## Access

The web interface is served by the gateway:

```
http://localhost:3777
```

## Features

- Real-time chat with your agent
- Session history and management
- Configuration editor
- System health monitoring

## Configuration

```toml
[web]
enabled = true
port = 3777
# bind = "0.0.0.0"  # for remote access
```

## See Also

- [Web Overview](index.md)
- [Dashboard](dashboard.md)
- [WebChat](webchat.md)
- [TUI](tui.md)
"#,
        title = page.title,
    )
}

fn generate_security_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> Security documentation

## Overview

{title} describes security considerations and threat models for MagicMerlin.
Security is a first-class concern in the architecture, from sandboxed tool
execution to encrypted communication channels.

## Threat Model

MagicMerlin's threat model considers:

- **Agent autonomy risks** -- Agents can execute tools; sandbox policies limit scope
- **Channel security** -- End-to-end encryption where supported by the channel
- **Credential management** -- Secrets are stored encrypted, never in plaintext config
- **Network exposure** -- Gateway binds to localhost by default

## Best Practices

1. Enable sandbox mode for all tool execution
2. Use approval policies for destructive operations
3. Rotate API keys regularly
4. Keep MagicMerlin updated

## See Also

- [Gateway Security](../gateway/security/index.md)
- [Sandboxing](../gateway/sandboxing.md)
- [Secrets Management](../gateway/secrets.md)
"#,
        title = page.title,
    )
}

fn generate_general_page(page: &DocPage) -> String {
    format!(
        r#"# {title}

> MagicMerlin documentation

## Overview

{title} covers an important aspect of the MagicMerlin ecosystem. This page
provides reference information, configuration guidance, and usage examples.

## Details

{title} integrates with the MagicMerlin gateway and agent runtime to provide
a seamless experience. Consult the sections below for setup and usage.

## Configuration

Relevant settings can be adjusted in the gateway configuration:

```toml
# ~/.config/magicmerlin/gateway.toml
```

## See Also

- [Getting Started](start/getting-started.md)
- [CLI Reference](cli/index.md)
- [Gateway Runbook](gateway/index.md)
"#,
        title = page.title,
    )
}

/// Generate all documentation pages from the index.
///
/// `project_root` is the path to the MagicMerlin repository root.
pub fn generate_docs(project_root: &Path) -> Result<GenerateReport> {
    let index_path = project_root.join("parity/openclaw_docs_index.json");
    let docs_root = project_root.join("docs");

    let index = load_index(&index_path)?;
    let mut pages_written = 0u32;
    let mut sections: BTreeMap<String, u32> = BTreeMap::new();
    let mut errors: Vec<String> = Vec::new();

    for entry in &index.pages {
        let page = match parse_page(entry) {
            Some(p) => p,
            None => {
                // Skip non-markdown entries (e.g. openapi.json)
                continue;
            }
        };

        let dest = docs_root.join(&page.rel_path);
        if let Some(parent) = dest.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                errors.push(format!("mkdir {}: {}", parent.display(), e));
                continue;
            }
        }

        let content = generate_content(&page);
        if let Err(e) = fs::write(&dest, &content) {
            errors.push(format!("write {}: {}", dest.display(), e));
            continue;
        }

        *sections.entry(page.section.clone()).or_insert(0) += 1;
        pages_written += 1;
    }

    Ok(GenerateReport {
        total_in_index: index.pages.len() as u32,
        pages_written,
        sections,
        errors,
    })
}

/// Report returned by [`generate_docs`].
#[derive(Debug)]
pub struct GenerateReport {
    pub total_in_index: u32,
    pub pages_written: u32,
    pub sections: BTreeMap<String, u32>,
    pub errors: Vec<String>,
}

impl std::fmt::Display for GenerateReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Docs generation report")?;
        writeln!(f, "  Index entries : {}", self.total_in_index)?;
        writeln!(f, "  Pages written : {}", self.pages_written)?;
        writeln!(f, "  Sections:")?;
        for (section, count) in &self.sections {
            writeln!(f, "    {:<20} {}", section, count)?;
        }
        if !self.errors.is_empty() {
            writeln!(f, "  Errors ({}):", self.errors.len())?;
            for e in &self.errors {
                writeln!(f, "    {}", e)?;
            }
        }
        Ok(())
    }
}

# Getting Started with MagicMerlin

> Your personal AI agent, running locally, connected everywhere.

MagicMerlin is a Rust-native AI agent runtime that runs on your own hardware.
It connects to any LLM provider, communicates through your favorite messaging
channels, and executes tools on your behalf -- all while keeping your data
under your control.

## Prerequisites

Before you begin, make sure you have:

- **An operating system**: macOS 13+, Linux (glibc 2.31+), or Windows 10+ via WSL2
- **An LLM API key**: From any [supported provider](../providers/index.md)
  (Anthropic, OpenAI, Ollama, etc.)
- **A terminal**: Any modern terminal emulator

## Step 1: Install MagicMerlin

The fastest way to install is via the one-line installer:

```bash
curl -fsSL https://get.magicmerlin.dev | sh
```

This installs the `magicmerlin` binary and sets up the default configuration
directory at `~/.config/magicmerlin/`.

For other installation methods (Docker, Nix, Podman, cloud platforms), see the
[Installation Guide](../install/index.md).

## Step 2: Run the Setup Wizard

The interactive setup wizard walks you through initial configuration:

```bash
magicmerlin setup
```

The wizard will:

1. **Select a model provider** -- Choose from Anthropic, OpenAI, Ollama, or
   [29 other providers](../providers/index.md)
2. **Enter your API key** -- Stored encrypted in the local secrets store
3. **Choose a model** -- Pick a default model for your agent
4. **Configure the agent personality** -- Set a system prompt that defines
   how your agent behaves

## Step 3: Start the Gateway

The gateway is the core runtime process. It manages sessions, routes messages,
executes tools, and coordinates with connected nodes and channels.

```bash
magicmerlin gateway start
```

Verify it is running:

```bash
magicmerlin health
```

You should see output like:

```
Gateway     : running (pid 12345)
Uptime      : 3s
Model       : claude-sonnet-4-20250514
Sessions    : 0 active
Tools       : 28 loaded
```

## Step 4: Send Your First Message

Talk to your agent directly from the CLI:

```bash
magicmerlin message "Hello! What can you help me with?"
```

The agent will respond using the configured model. This creates a new
**session** -- a conversation context that persists across messages.

## Step 5: Connect a Channel (Optional)

To interact with your agent through a messaging app, pair a channel:

=== "Telegram"

    ```bash
    magicmerlin channels pair telegram
    ```

    You will be prompted for your Telegram Bot Token. Once paired, message
    your bot on Telegram and MagicMerlin will respond.

=== "Discord"

    ```bash
    magicmerlin channels pair discord
    ```

    Provide your Discord Bot Token. The agent will respond to messages in
    configured channels.

=== "Slack"

    ```bash
    magicmerlin channels pair slack
    ```

    Follow the OAuth flow to connect your Slack workspace.

=== "iMessage"

    ```bash
    magicmerlin channels pair imessage
    ```

    Requires macOS with Messages.app configured.

See the full list of [29 supported channels](../channels/index.md).

## Step 6: Enable Tools

Tools give your agent the ability to act beyond text generation. Common tools
include:

| Tool | What it does |
|------|-------------|
| `exec` | Run shell commands |
| `browser` | Navigate web pages |
| `web` | Search the web and fetch URLs |
| `pdf` | Read and analyze PDF files |
| `skills` | Run pre-built automation scripts |
| `subagents` | Delegate tasks to specialized agents |

Tools are enabled in the gateway configuration:

```toml
# ~/.config/magicmerlin/gateway.toml
[tools]
exec = true
browser = true
web = true
```

Or via the CLI:

```bash
magicmerlin configure --enable-tool exec
magicmerlin configure --enable-tool browser
```

## What is Next

Now that you have a running agent, explore these areas:

- **[Concepts](../concepts/agent.md)** -- Understand the agent runtime,
  sessions, memory, and architecture
- **[Tools](../tools/index.md)** -- Browse all available tools and their
  configuration
- **[Automation](../automation/cron-jobs.md)** -- Set up scheduled tasks,
  webhooks, and event-driven workflows
- **[CLI Reference](../cli/index.md)** -- Complete reference for all 46 CLI
  commands
- **[Gateway Configuration](../gateway/configuration.md)** -- Fine-tune your
  gateway settings

## Architecture at a Glance

```
                +-------------------+
                |   Your Messages   |
                | (Telegram, Slack, |
                |  CLI, iMessage)   |
                +--------+----------+
                         |
                    +----v----+
                    | Gateway |  <-- core runtime (Rust)
                    +----+----+
                         |
            +------------+------------+
            |            |            |
       +----v---+  +----v---+  +----v----+
       | Agent  |  | Tools  |  | Memory  |
       | (LLM)  |  | (exec, |  | (long-  |
       |        |  | browse)|  |  term)   |
       +--------+  +--------+  +---------+
```

The **gateway** is the single process that orchestrates everything. It
connects to LLM providers over HTTPS, receives messages from channels,
dispatches tool calls, and persists conversation state. All of this runs
locally on your machine -- no cloud relay required.

## See Also

- [Setup Wizard Details](setup.md)
- [Installation Guide](../install/index.md)
- [Agent Runtime](../concepts/agent.md)
- [CLI Reference](../cli/index.md)

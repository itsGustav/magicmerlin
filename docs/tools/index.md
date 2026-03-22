# Tools

> Extending your agent's capabilities beyond text.

**Tools** are the mechanism by which a MagicMerlin agent interacts with the
outside world. When the agent determines that it needs to perform an action --
run a command, browse a webpage, search the web, read a PDF -- it issues a
**tool call**. The gateway executes the tool and returns the result to the
agent, which incorporates it into its reasoning.

## How Tools Work

```
User: "What's the weather in Stockholm?"
  |
  v
Agent (LLM) --> decides to call: web_search("weather Stockholm")
  |
  v
Gateway --> executes web_search tool --> returns results
  |
  v
Agent (LLM) --> reads results --> generates response
  |
  v
User: "It's currently 5C and partly cloudy in Stockholm."
```

The agent sees tool definitions in its system prompt and can call any enabled
tool by name. The gateway validates the call, applies security policies
(sandboxing, approvals), executes it, and feeds the result back.

## Built-in Tools

### Execution

| Tool | Description | Guide |
|------|-------------|-------|
| **exec** | Run shell commands | [exec](exec.md) |
| **apply_patch** | Apply unified diffs to files | [apply_patch](apply-patch.md) |
| **llm_task** | Delegate a subtask to another LLM call | [llm_task](llm-task.md) |

### Web and Browser

| Tool | Description | Guide |
|------|-------------|-------|
| **web_search** | Search the web via Brave/Perplexity/Google | [web](web.md) |
| **web_fetch** | Fetch and extract content from URLs | [web](web.md) |
| **browser** | Full browser automation via CDP | [browser](browser.md) |
| **browser_login** | Authenticate in the browser | [browser_login](browser-login.md) |
| **chrome_extension** | Interact with Chrome extensions | [chrome_extension](chrome-extension.md) |
| **firecrawl** | Structured web scraping | [firecrawl](firecrawl.md) |

### Media

| Tool | Description | Guide |
|------|-------------|-------|
| **pdf** | Read and analyze PDF files | [pdf](pdf.md) |
| **reactions** | Send emoji reactions to messages | [reactions](reactions.md) |

### Agent Collaboration

| Tool | Description | Guide |
|------|-------------|-------|
| **agent_send** | Send a task to a sub-agent | [agent_send](agent-send.md) |
| **acp_agents** | Discover and invoke ACP agents | [acp_agents](acp-agents.md) |
| **subagents** | Manage sub-agent lifecycles | [subagents](subagents.md) |

### Automation

| Tool | Description | Guide |
|------|-------------|-------|
| **skills** | Run pre-built automation scripts | [skills](skills.md) |
| **slash_commands** | Handle `/command` style inputs | [slash_commands](slash-commands.md) |
| **plugin** | Execute plugin-provided tools | [plugin](plugin.md) |

## Enabling Tools

Tools are configured in the gateway configuration file:

```toml
# ~/.config/magicmerlin/gateway.toml

[tools]
# Enable specific tools
exec = true
browser = true
web = true
pdf = true

# Disable a tool explicitly
firecrawl = false
```

Or use the CLI:

```bash
magicmerlin configure --enable-tool exec
magicmerlin configure --disable-tool firecrawl
```

## Security Model

Tool execution in MagicMerlin follows a layered security model:

### 1. Sandbox Policy

Tools run inside a sandbox by default. The sandbox restricts file system
access, network calls, and process creation:

```toml
[tools.sandbox]
enabled = true
allow_network = true
allow_fs_read = ["~/workspace", "/tmp"]
allow_fs_write = ["~/workspace"]
```

### 2. Approval Policy

Certain tools require explicit user approval before execution. This is
especially important for destructive operations:

```toml
[tools.approvals]
# Always require approval for these tools
require = ["exec", "apply_patch"]

# Auto-approve these tools
auto_approve = ["web_search", "web_fetch", "pdf"]
```

When an approval is required, the user is prompted through their active
channel:

```
Agent wants to run: exec("rm -rf /tmp/old-build")
[Approve] [Deny] [Approve All]
```

### 3. Elevated Mode

For trusted environments, elevated mode removes sandbox restrictions:

```bash
magicmerlin gateway start --elevated
```

See [Elevated Mode](elevated.md) and
[Sandbox vs Tool Policy vs Elevated](../gateway/sandbox-vs-tool-policy-vs-elevated.md).

## Tool Loop Detection

MagicMerlin monitors tool-call patterns and detects infinite loops. If the
agent calls the same tool with the same arguments repeatedly, the loop
detector intervenes:

```toml
[tools.loop_detection]
enabled = true
max_identical_calls = 3
window_seconds = 60
```

See [Tool-loop detection](loop-detection.md).

## Custom Tools via Plugins

You can extend MagicMerlin with custom tools through the plugin system. A
plugin declares its tools in a manifest file:

```json
{
  "name": "my-tool",
  "tools": [
    {
      "name": "my_custom_tool",
      "description": "Does something custom",
      "parameters": {
        "type": "object",
        "properties": {
          "input": { "type": "string" }
        }
      }
    }
  ]
}
```

See [Plugins](plugin.md) and [Plugin Manifest](../plugins/manifest.md).

## Listing Available Tools

```bash
# List all tools and their status
magicmerlin status --tools

# Output as JSON
magicmerlin status --tools --json
```

Example output:

```
Tool            Enabled   Approvals   Sandbox
----            -------   ---------   -------
exec            yes       required    yes
browser         yes       auto        yes
web_search      yes       auto        no
web_fetch       yes       auto        no
pdf             yes       auto        no
agent_send      yes       auto        no
skills          yes       auto        yes
apply_patch     yes       required    yes
```

## See Also

- [Exec Tool](exec.md) -- Shell command execution
- [Browser](browser.md) -- Web automation
- [Web Tools](web.md) -- Search and fetch
- [Exec Approvals](exec-approvals.md) -- Approval workflow
- [Elevated Mode](elevated.md) -- Running without sandbox
- [Creating Skills](creating-skills.md) -- Building reusable automations
- [Sub-Agents](subagents.md) -- Delegating to specialized agents

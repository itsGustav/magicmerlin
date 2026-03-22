# Agent Runtime

> How MagicMerlin thinks, acts, and responds.

The **agent** is the core reasoning unit in MagicMerlin. It is the component
that receives a user message, decides what to do, calls tools if needed, and
produces a response. Under the hood, the agent wraps an LLM (Large Language
Model) with a structured execution loop, tool dispatch, memory access, and
session state.

## Architecture

```
  Message In
      |
      v
+-----+-----+
| System     |  <-- personality, instructions, context
| Prompt     |
+-----+-----+
      |
      v
+-----+-----+
| Agent Loop |  <-- reason / act / observe cycle
+-----+-----+
      |
      +-------> Tool Call? ---> Execute Tool ---> Observe Result
      |                                              |
      v                                              |
  Response  <----------------------------------------+
```

The agent loop runs inside the gateway process. Each iteration:

1. **Assembles the prompt** -- system prompt + session history + memory + tool
   definitions
2. **Calls the LLM** -- sends the assembled prompt to the configured model
   provider
3. **Parses the response** -- detects tool-call requests in the model output
4. **Executes tools** (if any) -- dispatches tool calls, collects results
5. **Loops or responds** -- if tools were called, feeds results back to the
   model; otherwise, delivers the final response to the user

This is sometimes called a ReAct (Reason + Act) loop.

## System Prompt

The system prompt defines the agent's personality, instructions, and
constraints. It is the first message in every conversation and is always
included when calling the LLM.

```toml
# gateway.toml
[agent]
system_prompt = """
You are Merlin, a helpful personal assistant.
You are concise, accurate, and proactive.
When asked to perform tasks, use the available tools.
"""
```

The system prompt can also be loaded from a file:

```toml
[agent]
system_prompt_file = "SOUL.md"
```

MagicMerlin ships with several [prompt templates](../reference/templates/SOUL.md)
that you can customize.

## Agent Configuration

Key agent settings in `gateway.toml`:

```toml
[agent]
# The default model to use
model = "claude-sonnet-4-20250514"

# Maximum tokens the model may generate per turn
max_tokens = 4096

# Temperature (0.0 = deterministic, 1.0 = creative)
temperature = 0.7

# Whether to include tool definitions in the prompt
tools_enabled = true

# Maximum number of tool-call iterations per message
max_tool_rounds = 10

# Thinking / chain-of-thought budget (tokens)
thinking_budget = 1024
```

## Multi-Agent Routing

MagicMerlin supports running multiple agents simultaneously, each with its own
system prompt, model, and tool set. Messages are routed to agents based on
configurable rules:

- **Channel-based** -- Different agents for different channels
- **Keyword-based** -- Route based on message content
- **Session-based** -- Sticky routing once a session starts

See [Multi-Agent Routing](multi-agent.md) for details.

## Sub-Agents

An agent can delegate tasks to **sub-agents** -- specialized agents that
handle specific domains. The parent agent calls the `agent_send` tool to
dispatch work and receives the result.

```
Parent Agent
    |
    +---> Sub-Agent (code review)
    +---> Sub-Agent (web research)
    +---> Sub-Agent (data analysis)
```

Sub-agents run in isolated sessions and can have different models, tools, and
system prompts. See [Sub-Agents](../tools/subagents.md).

## Agent Workspace

Each agent has access to a **workspace** -- a directory on disk where it can
read and write files. This enables file-based workflows like code editing,
document generation, and data processing.

```toml
[agent]
workspace = "~/merlin-workspace"
```

See [Agent Workspace](agent-workspace.md).

## Observability

The agent loop emits structured logs and metrics at every step:

```bash
# Watch agent reasoning in real time
magicmerlin logs --agent --follow

# See tool call history for the current session
magicmerlin sessions history --tools
```

Key metrics:

| Metric | Description |
|--------|-------------|
| `agent.turns` | Number of LLM calls per message |
| `agent.tool_calls` | Total tool invocations |
| `agent.latency_ms` | End-to-end response time |
| `agent.tokens_in` | Input tokens consumed |
| `agent.tokens_out` | Output tokens generated |

## See Also

- [Agent Loop](agent-loop.md) -- Detailed loop mechanics
- [System Prompt](system-prompt.md) -- Prompt engineering guide
- [Session Management](session.md) -- How conversations are tracked
- [Memory](memory.md) -- Long-term knowledge persistence
- [Tools](../tools/index.md) -- Available tool catalog
- [Model Providers](model-providers.md) -- Supported LLM backends

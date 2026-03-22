# Markdown Formatting

> Conceptual reference

## Overview

Markdown Formatting is a core concept in MagicMerlin's architecture. Understanding this
concept is essential for building effective agent configurations and workflows.

## How It Works

MagicMerlin implements markdown formatting as part of its agent runtime. This
mechanism ensures reliable and efficient operation across all connected
channels, tools, and sessions.

## Key Properties

- **Consistency** -- Markdown Formatting state is persisted across gateway restarts
- **Isolation** -- Each session maintains its own markdown formatting context
- **Efficiency** -- Optimized for minimal latency and resource usage
- **Observability** -- Full logging and metrics for markdown formatting operations

## Configuration

```toml
# gateway.toml
[markdown-formatting]
enabled = true
```

## Related Concepts

- [Agent Runtime](agent.md)
- [Session Management](session.md)
- [Memory](memory.md)

## See Also

- [Getting Started](../start/getting-started.md)
- [Gateway Architecture](architecture.md)

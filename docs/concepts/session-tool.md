# Session Tools

> Conceptual reference

## Overview

Session Tools is a core concept in MagicMerlin's architecture. Understanding this
concept is essential for building effective agent configurations and workflows.

## How It Works

MagicMerlin implements session tools as part of its agent runtime. This
mechanism ensures reliable and efficient operation across all connected
channels, tools, and sessions.

## Key Properties

- **Consistency** -- Session Tools state is persisted across gateway restarts
- **Isolation** -- Each session maintains its own session tools context
- **Efficiency** -- Optimized for minimal latency and resource usage
- **Observability** -- Full logging and metrics for session tools operations

## Configuration

```toml
# gateway.toml
[session-tool]
enabled = true
```

## Related Concepts

- [Agent Runtime](agent.md)
- [Session Management](session.md)
- [Memory](memory.md)

## See Also

- [Getting Started](../start/getting-started.md)
- [Gateway Architecture](architecture.md)

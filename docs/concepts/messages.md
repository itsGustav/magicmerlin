# Messages

> Conceptual reference

## Overview

Messages is a core concept in MagicMerlin's architecture. Understanding this
concept is essential for building effective agent configurations and workflows.

## How It Works

MagicMerlin implements messages as part of its agent runtime. This
mechanism ensures reliable and efficient operation across all connected
channels, tools, and sessions.

## Key Properties

- **Consistency** -- Messages state is persisted across gateway restarts
- **Isolation** -- Each session maintains its own messages context
- **Efficiency** -- Optimized for minimal latency and resource usage
- **Observability** -- Full logging and metrics for messages operations

## Configuration

```toml
# gateway.toml
[messages]
enabled = true
```

## Related Concepts

- [Agent Runtime](agent.md)
- [Session Management](session.md)
- [Memory](memory.md)

## See Also

- [Getting Started](../start/getting-started.md)
- [Gateway Architecture](architecture.md)

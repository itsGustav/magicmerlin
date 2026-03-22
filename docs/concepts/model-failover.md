# Model Failover

> Conceptual reference

## Overview

Model Failover is a core concept in MagicMerlin's architecture. Understanding this
concept is essential for building effective agent configurations and workflows.

## How It Works

MagicMerlin implements model failover as part of its agent runtime. This
mechanism ensures reliable and efficient operation across all connected
channels, tools, and sessions.

## Key Properties

- **Consistency** -- Model Failover state is persisted across gateway restarts
- **Isolation** -- Each session maintains its own model failover context
- **Efficiency** -- Optimized for minimal latency and resource usage
- **Observability** -- Full logging and metrics for model failover operations

## Configuration

```toml
# gateway.toml
[model-failover]
enabled = true
```

## Related Concepts

- [Agent Runtime](agent.md)
- [Session Management](session.md)
- [Memory](memory.md)

## See Also

- [Getting Started](../start/getting-started.md)
- [Gateway Architecture](architecture.md)

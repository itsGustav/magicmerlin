# Reactions

> Tool reference

## Overview

Reactions is a built-in tool available to MagicMerlin agents. Tools extend the
agent's capabilities beyond text generation, enabling interaction with external
systems, files, browsers, and more.

## Usage

The tool is automatically available when enabled in your agent configuration:

```toml
[tools.reactions]
enabled = true
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input` | string | yes | Primary input for the tool |
| `options` | object | no | Additional configuration |

## Tool Schema

```json
{
  "name": "reactions",
  "description": "Reactions",
  "parameters": {
    "type": "object",
    "properties": {
      "input": { "type": "string" }
    }
  }
}
```

## Examples

The agent can invoke this tool during a conversation when it determines
that reactions capabilities are needed.

## Security

- Tool execution respects the sandbox policy
- Approval may be required depending on configuration
- All invocations are logged

## See Also

- [Tools Overview](index.md)
- [Exec Approvals](exec-approvals.md)
- [Elevated Mode](elevated.md)

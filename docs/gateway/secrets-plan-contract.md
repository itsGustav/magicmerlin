# Secrets Apply Plan Contract

> Gateway reference

## Overview

Secrets Apply Plan Contract covers an essential aspect of the MagicMerlin gateway. The gateway
acts as the central hub for all agent communication, tool execution, and
session management.

## Configuration

The gateway reads its configuration from `~/.config/magicmerlin/gateway.toml`.
Settings related to secrets apply plan contract can be adjusted there.

```toml
[gateway]
# Secrets Apply Plan Contract settings
enabled = true
```

## API

### Request

```json
{
  "method": "gateway.secrets-plan-contract",
  "params": {}
}
```

### Response

```json
{
  "ok": true,
  "data": {}
}
```

## Troubleshooting

If you encounter issues with secrets apply plan contract:

1. Check gateway logs: `magicmerlin logs --gateway`
2. Verify configuration: `magicmerlin doctor`
3. Restart the gateway: `magicmerlin gateway restart`

## See Also

- [Gateway Runbook](index.md)
- [Gateway Protocol](protocol.md)
- [Troubleshooting](troubleshooting.md)

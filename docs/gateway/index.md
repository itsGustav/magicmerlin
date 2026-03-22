# Gateway Runbook

> Gateway reference

## Overview

Gateway Runbook covers an essential aspect of the MagicMerlin gateway. The gateway
acts as the central hub for all agent communication, tool execution, and
session management.

## Configuration

The gateway reads its configuration from `~/.config/magicmerlin/gateway.toml`.
Settings related to gateway runbook can be adjusted there.

```toml
[gateway]
# Gateway Runbook settings
enabled = true
```

## API

### Request

```json
{
  "method": "gateway.index",
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

If you encounter issues with gateway runbook:

1. Check gateway logs: `magicmerlin logs --gateway`
2. Verify configuration: `magicmerlin doctor`
3. Restart the gateway: `magicmerlin gateway restart`

## See Also

- [Gateway Runbook](index.md)
- [Gateway Protocol](protocol.md)
- [Troubleshooting](troubleshooting.md)

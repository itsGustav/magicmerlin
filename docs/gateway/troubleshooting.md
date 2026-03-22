# Troubleshooting

> Gateway reference

## Overview

Troubleshooting covers an essential aspect of the MagicMerlin gateway. The gateway
acts as the central hub for all agent communication, tool execution, and
session management.

## Configuration

The gateway reads its configuration from `~/.config/magicmerlin/gateway.toml`.
Settings related to troubleshooting can be adjusted there.

```toml
[gateway]
# Troubleshooting settings
enabled = true
```

## API

### Request

```json
{
  "method": "gateway.troubleshooting",
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

If you encounter issues with troubleshooting:

1. Check gateway logs: `magicmerlin logs --gateway`
2. Verify configuration: `magicmerlin doctor`
3. Restart the gateway: `magicmerlin gateway restart`

## See Also

- [Gateway Runbook](index.md)
- [Gateway Protocol](protocol.md)
- [Troubleshooting](troubleshooting.md)

# Formal Verification (Security Models)

> Security documentation

## Overview

Formal Verification (Security Models) describes security considerations and threat models for MagicMerlin.
Security is a first-class concern in the architecture, from sandboxed tool
execution to encrypted communication channels.

## Threat Model

MagicMerlin's threat model considers:

- **Agent autonomy risks** -- Agents can execute tools; sandbox policies limit scope
- **Channel security** -- End-to-end encryption where supported by the channel
- **Credential management** -- Secrets are stored encrypted, never in plaintext config
- **Network exposure** -- Gateway binds to localhost by default

## Best Practices

1. Enable sandbox mode for all tool execution
2. Use approval policies for destructive operations
3. Rotate API keys regularly
4. Keep MagicMerlin updated

## See Also

- [Gateway Security](../gateway/security/index.md)
- [Sandboxing](../gateway/sandboxing.md)
- [Secrets Management](../gateway/secrets.md)

# Chat Channels

> Channel setup guide

## Overview

Chat Channels is a supported messaging channel in MagicMerlin. Channels allow your
agent to communicate through various messaging platforms and protocols.

## Setup

### Prerequisites

- A running MagicMerlin gateway
- Valid credentials for Chat Channels

### Configuration

Add the channel configuration to your gateway config:

```toml
[channels.index]
enabled = true
# Add your credentials here
```

### Pairing

```bash
magicmerlin channels pair index
```

## Features

- Real-time message delivery
- Media support (images, files, voice)
- Group conversation support
- Typing indicators
- Read receipts (where supported)

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection drops | Check network and credentials |
| Messages not delivered | Verify channel is paired |
| Media not loading | Check file size limits |

## See Also

- [Chat Channels](index.md)
- [Channel Routing](channel-routing.md)
- [Pairing](pairing.md)
- [Channel Troubleshooting](troubleshooting.md)

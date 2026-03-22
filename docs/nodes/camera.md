# Camera Capture

> Node reference

## Overview

Camera Capture is a node capability in MagicMerlin. Nodes are edge devices (phones,
desktops, Raspberry Pis) that connect to the gateway and provide local
hardware access such as cameras, microphones, and sensors.

## Setup

Pair a node with your gateway:

```bash
magicmerlin node pair
```

## Features

- Real-time streaming from device hardware
- Secure communication via the gateway bridge
- Automatic reconnection and heartbeat monitoring
- Media transcoding and delivery

## Configuration

```toml
[nodes]
auto_accept = false
heartbeat_interval = 30
```

## Troubleshooting

- Verify the node is online: `magicmerlin nodes list`
- Check connectivity: `magicmerlin node ping <id>`
- Review logs: `magicmerlin logs --node <id>`

## See Also

- [Nodes Overview](index.md)
- [Audio and Voice Notes](audio.md)
- [Node Troubleshooting](troubleshooting.md)

# WebChat

> Web interface reference

## Overview

WebChat is part of MagicMerlin's web-based interface. The web UI provides
browser-accessible dashboards, chat interfaces, and administrative controls.

## Access

The web interface is served by the gateway:

```
http://localhost:3777
```

## Features

- Real-time chat with your agent
- Session history and management
- Configuration editor
- System health monitoring

## Configuration

```toml
[web]
enabled = true
port = 3777
# bind = "0.0.0.0"  # for remote access
```

## See Also

- [Web Overview](index.md)
- [Dashboard](dashboard.md)
- [WebChat](webchat.md)
- [TUI](tui.md)

# Zalo Personal Plugin

> Plugin reference

## Overview

Zalo Personal Plugin extends MagicMerlin's functionality through the plugin system. Plugins
are modular components that can add new tools, channels, and integrations.

## Installation

```bash
magicmerlin plugins install zalouser
```

## Configuration

```toml
[plugins.zalouser]
enabled = true
```

## Plugin Manifest

Every plugin includes a `manifest.json` that declares:
- Required permissions
- Tool definitions
- Channel bindings
- Configuration schema

## Development

See [Plugin Manifest](manifest.md) for details on creating your own plugins.

## See Also

- [Community Plugins](community.md)
- [Plugin Manifest](manifest.md)
- [Tools](../tools/index.md)

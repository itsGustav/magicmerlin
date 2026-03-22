# Voice Call Plugin

> Plugin reference

## Overview

Voice Call Plugin extends MagicMerlin's functionality through the plugin system. Plugins
are modular components that can add new tools, channels, and integrations.

## Installation

```bash
magicmerlin plugins install voice-call
```

## Configuration

```toml
[plugins.voice-call]
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

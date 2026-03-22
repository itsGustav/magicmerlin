# Ansible

> Installation guide

## Overview

This guide covers installing MagicMerlin via Ansible. Choose the installation
method that best fits your environment and workflow.

## Prerequisites

- A supported operating system (macOS, Linux, Windows via WSL2)
- Network access for downloading packages
- Sufficient disk space (approximately 200 MB)

## Installation

### Ansible

```bash
# Install MagicMerlin via ansible
# Refer to the specific instructions below
```

## Post-Installation

After installation, run the setup wizard:

```bash
magicmerlin setup
```

This will guide you through:
- Configuring a model provider
- Setting up your first channel
- Initializing the gateway

## Verifying Installation

```bash
magicmerlin doctor
magicmerlin --version
```

## Updating

```bash
magicmerlin update
```

## See Also

- [Install Overview](index.md)
- [Getting Started](../start/getting-started.md)
- [Uninstall](uninstall.md)

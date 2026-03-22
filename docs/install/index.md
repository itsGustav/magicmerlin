# Installation Guide

> Choose the installation method that fits your environment.

MagicMerlin is distributed as a single static binary for macOS, Linux, and
Windows (via WSL2). Several packaging and deployment options are available
depending on your setup.

## Quick Install (Recommended)

The one-line installer detects your platform, downloads the latest release,
and installs the `magicmerlin` binary to `~/.local/bin/`:

```bash
curl -fsSL https://get.magicmerlin.dev | sh
```

After installation, start the setup wizard:

```bash
magicmerlin setup
```

## System Requirements

| Requirement | Minimum |
|-------------|---------|
| **OS** | macOS 13+, Linux (glibc 2.31+), Windows 10+ (WSL2) |
| **CPU** | x86_64 or aarch64 (Apple Silicon native) |
| **RAM** | 256 MB (gateway idle), 512 MB+ recommended |
| **Disk** | ~100 MB for binary, ~200 MB with all tools |
| **Network** | Required for LLM API calls; local-only mode with Ollama |

## Installation Methods

### Package Managers and Runtimes

| Method | Guide | Notes |
|--------|-------|-------|
| **Node.js (npm/npx)** | [install/node](node.md) | `npx magicmerlin` for quick start |
| **Nix** | [install/nix](nix.md) | Reproducible builds, flake support |
| **Bun** | [install/bun](bun.md) | Experimental; fast JS runtime |

### Containers

| Method | Guide | Notes |
|--------|-------|-------|
| **Docker** | [install/docker](docker.md) | Official image on Docker Hub |
| **Podman** | [install/podman](podman.md) | Rootless containers |

### Cloud and VPS

| Platform | Guide | Notes |
|----------|-------|-------|
| **Fly.io** | [install/fly](fly.md) | One-click deploy |
| **Railway** | [install/railway](railway.md) | Template available |
| **Render** | [install/render](render.md) | Background worker |
| **Northflank** | [install/northflank](northflank.md) | Container deploy |
| **GCP** | [install/gcp](gcp.md) | Compute Engine / Cloud Run |
| **Hetzner** | [install/hetzner](hetzner.md) | VPS setup |

### Configuration Management

| Method | Guide | Notes |
|--------|-------|-------|
| **Ansible** | [install/ansible](ansible.md) | Playbook for fleet deploy |

### Platform-Specific

| Platform | Guide | Notes |
|----------|-------|-------|
| **macOS App** | [platforms/macos](../platforms/macos.md) | Native menu-bar app |
| **macOS VM** | [install/macos-vm](macos-vm.md) | Tart / Anka VMs |
| **Linux** | [platforms/linux](../platforms/linux.md) | Systemd service |
| **Windows** | [platforms/windows](../platforms/windows.md) | WSL2 required |
| **Raspberry Pi** | [platforms/raspberry-pi](../platforms/raspberry-pi.md) | ARM64 binary |

## Verifying Your Installation

After installing, run the doctor command to check that everything is
configured correctly:

```bash
magicmerlin doctor
```

Expected output:

```
Binary       : /usr/local/bin/magicmerlin
Version      : 0.1.0
Config dir   : ~/.config/magicmerlin
Data dir     : ~/.local/share/magicmerlin
Gateway      : not running
Provider     : (not configured -- run magicmerlin setup)
```

## Updating

MagicMerlin can update itself in place:

```bash
magicmerlin update
```

To check for updates without installing:

```bash
magicmerlin update --check
```

For details on release channels (stable, beta, nightly), see
[Development Channels](development-channels.md).

## Uninstalling

```bash
magicmerlin uninstall
```

This removes the binary and optionally clears configuration and data. See
[Uninstall](uninstall.md) for details.

## Next Steps

- [Getting Started](../start/getting-started.md) -- Set up your first agent
- [Gateway Configuration](../gateway/configuration.md) -- Fine-tune runtime settings
- [CLI Reference](../cli/index.md) -- Explore all available commands

# GitHub Copilot

> Model provider setup

## Overview

GitHub Copilot is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the GitHub Copilot platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider github-copilot
```

Or add directly to your configuration:

```toml
[providers.github-copilot]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider github-copilot
```

## Supported Models

Refer to the GitHub Copilot documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure GitHub Copilot as a failover provider:

```toml
[failover]
providers = ["github-copilot", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

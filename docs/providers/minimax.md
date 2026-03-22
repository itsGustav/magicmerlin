# MiniMax

> Model provider setup

## Overview

MiniMax is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the MiniMax platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider minimax
```

Or add directly to your configuration:

```toml
[providers.minimax]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider minimax
```

## Supported Models

Refer to the MiniMax documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure MiniMax as a failover provider:

```toml
[failover]
providers = ["minimax", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

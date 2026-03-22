# Z.AI

> Model provider setup

## Overview

Z.AI is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the Z.AI platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider zai
```

Or add directly to your configuration:

```toml
[providers.zai]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider zai
```

## Supported Models

Refer to the Z.AI documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure Z.AI as a failover provider:

```toml
[failover]
providers = ["zai", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

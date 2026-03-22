# Moonshot AI

> Model provider setup

## Overview

Moonshot AI is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the Moonshot AI platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider moonshot
```

Or add directly to your configuration:

```toml
[providers.moonshot]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider moonshot
```

## Supported Models

Refer to the Moonshot AI documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure Moonshot AI as a failover provider:

```toml
[failover]
providers = ["moonshot", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

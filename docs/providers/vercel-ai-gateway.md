# Vercel AI Gateway

> Model provider setup

## Overview

Vercel AI Gateway is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the Vercel AI Gateway platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider vercel-ai-gateway
```

Or add directly to your configuration:

```toml
[providers.vercel-ai-gateway]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider vercel-ai-gateway
```

## Supported Models

Refer to the Vercel AI Gateway documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure Vercel AI Gateway as a failover provider:

```toml
[failover]
providers = ["vercel-ai-gateway", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

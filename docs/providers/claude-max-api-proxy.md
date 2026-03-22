# Claude Max API Proxy

> Model provider setup

## Overview

Claude Max API Proxy is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the Claude Max API Proxy platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider claude-max-api-proxy
```

Or add directly to your configuration:

```toml
[providers.claude-max-api-proxy]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider claude-max-api-proxy
```

## Supported Models

Refer to the Claude Max API Proxy documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure Claude Max API Proxy as a failover provider:

```toml
[failover]
providers = ["claude-max-api-proxy", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

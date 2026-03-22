# Deepgram

> Model provider setup

## Overview

Deepgram is a supported model provider in MagicMerlin. Model providers supply
the LLM backends that power agent reasoning and generation.

## Setup

### 1. Obtain API Key

Sign up at the Deepgram platform and generate an API key.

### 2. Configure Provider

```bash
magicmerlin configure --provider deepgram
```

Or add directly to your configuration:

```toml
[providers.deepgram]
api_key = "your-key-here"
# base_url = "https://api.example.com/v1"  # optional
```

### 3. Select a Model

```bash
magicmerlin models list --provider deepgram
```

## Supported Models

Refer to the Deepgram documentation for the latest list of available models.
MagicMerlin supports all chat-completion-compatible endpoints.

## Model Failover

You can configure Deepgram as a failover provider:

```toml
[failover]
providers = ["deepgram", "openai"]
```

## See Also

- [Model Providers](index.md)
- [Model Provider Quickstart](models.md)
- [Model Failover](../concepts/model-failover.md)

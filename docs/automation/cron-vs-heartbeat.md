# Cron vs Heartbeat

> Automation guide

## Overview

Cron vs Heartbeat enables automated workflows in MagicMerlin. Automation features allow
your agent to perform tasks on schedules, respond to events, and maintain
continuous operation without manual intervention.

## Setup

### Enable Automation

```toml
[automation]
enabled = true
```

### Configure Cron vs Heartbeat

```bash
magicmerlin cron add --schedule "*/5 * * * *" --action "cron-vs-heartbeat"
```

## How It Works

1. The gateway monitors configured triggers
2. When conditions are met, the automation engine fires
3. The agent processes the event within a new or existing session
4. Results are delivered to the configured output channel

## Examples

```bash
# List active automations
magicmerlin cron list

# Check automation status
magicmerlin status --automations
```

## Troubleshooting

- Verify cron syntax with `magicmerlin cron validate`
- Check gateway logs for trigger events
- Ensure the agent has necessary tool permissions

## See Also

- [Cron Jobs](cron-jobs.md)
- [Hooks](hooks.md)
- [Webhooks](webhook.md)

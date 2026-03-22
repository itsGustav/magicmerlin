# Session Management

> How MagicMerlin tracks conversations across messages and channels.

A **session** is a conversation between a user and an agent. It holds the
message history, tool-call results, and any accumulated context. Sessions
are the fundamental unit of state in MagicMerlin -- every interaction happens
within a session.

## Session Lifecycle

```
1. CREATE   -- A new message arrives with no active session
2. ACTIVE   -- Messages flow back and forth; history grows
3. IDLE     -- No messages for a configured timeout period
4. PRUNED   -- Old messages are compacted to save tokens
5. ARCHIVED -- Session is closed and moved to cold storage
```

### Creation

A session is created automatically when:

- A user sends a message through a channel with no active session
- The CLI sends a message with `magicmerlin message`
- A cron job or webhook triggers a new conversation
- A sub-agent is spawned

Each session receives a unique ID (e.g., `ses_a1b2c3d4`) and is bound to a
specific user + channel pair.

### Active State

While active, the session holds a rolling window of messages. Each message
from the user and each response from the agent is appended to the session
history. Tool calls and their results are also recorded.

### Idle and Timeout

After a configurable period of inactivity, the session transitions to idle.
The next message from the same user on the same channel will either resume
the existing session or create a new one, depending on configuration:

```toml
[sessions]
# Time before a session is considered idle
idle_timeout = "30m"

# Whether to resume idle sessions or start fresh
resume_idle = true
```

## Session Binding

Sessions are bound to a **user + channel** pair. This means:

- The same user on Telegram and Slack has **two separate sessions**
- Group chats have **one session per group** (shared context)
- CLI messages create sessions bound to the local user

You can override this with explicit session IDs:

```bash
magicmerlin message --session ses_custom123 "Continue our earlier discussion"
```

## Session History and Context Window

The session history is the list of messages sent to the LLM on each turn. As
conversations grow, the history can exceed the model's context window. MagicMerlin
handles this with several strategies:

### Sliding Window

Keep only the most recent N messages:

```toml
[sessions]
max_history_messages = 50
```

### Compaction

Periodically summarize older messages into a condensed form, preserving key
information while reducing token count:

```toml
[sessions.compaction]
enabled = true
trigger_tokens = 8000    # compact when history exceeds this
target_tokens = 3000     # aim for this size after compaction
```

See [Compaction](compaction.md) for the algorithm details.

### Pruning

Remove the oldest messages entirely when the history exceeds a hard limit:

```toml
[sessions]
max_history_tokens = 32000
prune_strategy = "oldest_first"  # or "summarize_then_prune"
```

See [Session Pruning](session-pruning.md).

## Session Storage

Sessions are persisted to disk so they survive gateway restarts:

```
~/.local/share/magicmerlin/sessions/
  ses_a1b2c3d4.json
  ses_e5f6g7h8.json
  ...
```

The storage backend is configurable:

```toml
[sessions.storage]
backend = "sqlite"  # "json" | "sqlite"
path = "~/.local/share/magicmerlin/sessions.db"
```

## Managing Sessions

### CLI Commands

```bash
# List active sessions
magicmerlin sessions list

# View session history
magicmerlin sessions history ses_a1b2c3d4

# Send a message to a specific session
magicmerlin sessions send ses_a1b2c3d4 "What were we talking about?"

# Close a session
magicmerlin sessions close ses_a1b2c3d4

# Export session as JSON
magicmerlin sessions export ses_a1b2c3d4 > conversation.json
```

### Gateway API

Sessions can also be managed through the gateway's JSON-RPC API:

```json
{
  "method": "sessions.list",
  "params": { "status": "active" }
}
```

```json
{
  "method": "sessions.spawn",
  "params": {
    "channel": "telegram",
    "user_id": "12345",
    "initial_message": "Hello"
  }
}
```

## Session Tools

The agent itself can manage sessions using built-in tools:

- `session.yield` -- Pause the current session and wait for the next message
- `session.summarize` -- Generate a summary of the conversation so far
- `session.context` -- Retrieve or inject context into the current session

See [Session Tools](session-tool.md).

## Configuration Reference

```toml
[sessions]
# Maximum number of concurrent active sessions
max_active = 100

# Idle timeout before session transitions to idle state
idle_timeout = "30m"

# Whether to resume idle sessions automatically
resume_idle = true

# Maximum messages to keep in history
max_history_messages = 100

# Maximum tokens in history before pruning
max_history_tokens = 32000

# Pruning strategy
prune_strategy = "summarize_then_prune"

[sessions.compaction]
enabled = true
trigger_tokens = 8000
target_tokens = 3000

[sessions.storage]
backend = "sqlite"
path = "~/.local/share/magicmerlin/sessions.db"
```

## See Also

- [Compaction](compaction.md) -- How session history is compressed
- [Session Pruning](session-pruning.md) -- Strategies for managing long conversations
- [Session Tools](session-tool.md) -- Tools for in-conversation session control
- [Agent Runtime](agent.md) -- How the agent uses session context
- [Memory](memory.md) -- Long-term knowledge that spans sessions

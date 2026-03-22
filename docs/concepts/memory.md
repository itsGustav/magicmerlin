# Memory

> Long-term knowledge that persists across sessions.

**Memory** in MagicMerlin is the mechanism for storing and retrieving facts,
preferences, and knowledge that should persist beyond a single conversation.
While [sessions](session.md) track the short-term conversation history, memory
provides the agent with long-term recall -- enabling it to remember your
preferences, past decisions, and accumulated knowledge across days, weeks, and
months.

## How Memory Differs from Session History

| Aspect | Session History | Memory |
|--------|----------------|--------|
| **Scope** | One conversation | All conversations |
| **Lifetime** | Until session ends/prunes | Permanent (until deleted) |
| **Content** | Raw messages + tool results | Distilled facts and knowledge |
| **Size** | Bounded by context window | Grows over time |
| **Retrieval** | Always included (recent) | Selectively retrieved by relevance |

## Memory Architecture

```
  User Message
      |
      v
  +---+---+
  | Agent  |
  +---+---+
      |
      +----> Memory Store (read: retrieve relevant memories)
      |           |
      v           v
  LLM Call <-- relevant memories injected into context
      |
      v
  Response
      |
      +----> Memory Store (write: extract new facts to remember)
```

The agent interacts with memory in two phases:

1. **Retrieval** -- Before calling the LLM, the gateway queries the memory
   store for facts relevant to the current message. These are injected into
   the system prompt or context window.

2. **Extraction** -- After the agent responds, new facts worth remembering
   are extracted and stored. This can happen automatically or via explicit
   tool calls.

## Memory Types

### Fact Memory

Structured key-value facts about the user or world:

```
user.name = "Gustav"
user.timezone = "Europe/Stockholm"
user.preferred_language = "English"
project.magicmerlin.status = "active development"
```

Facts are automatically extracted from conversations and can be managed
manually:

```bash
# List all stored facts
magicmerlin memory list

# Add a fact manually
magicmerlin memory set "user.birthday" "March 15"

# Remove a fact
magicmerlin memory delete "user.birthday"

# Search facts
magicmerlin memory search "timezone"
```

### Semantic Memory

Free-form text snippets stored with vector embeddings for similarity search.
This is useful for storing notes, summaries, and observations that do not fit
neatly into key-value pairs.

```bash
# Store a note
magicmerlin memory add "Gustav prefers concise responses without emojis"

# Search by similarity
magicmerlin memory search "communication style"
```

### Episodic Memory

Summaries of past sessions that the agent can reference. When a session ends,
the gateway can optionally generate a brief summary and store it as an episode:

```toml
[memory.episodic]
enabled = true
summarize_on_close = true
max_episodes = 500
```

## Configuration

```toml
[memory]
# Enable the memory system
enabled = true

# Storage backend
backend = "sqlite"  # "sqlite" | "json" | "vector"

# Path to the memory database
path = "~/.local/share/magicmerlin/memory.db"

# Maximum number of memory items to inject into context
max_context_items = 20

# Maximum tokens to spend on memory in the context window
max_context_tokens = 2000

[memory.auto_extract]
# Automatically extract facts from conversations
enabled = true

# How often to run extraction (every N messages)
interval = 5

[memory.episodic]
enabled = true
summarize_on_close = true
max_episodes = 500

[memory.vector]
# For semantic search (requires an embedding model)
embedding_model = "text-embedding-3-small"
embedding_provider = "openai"
similarity_threshold = 0.7
```

## Memory in the System Prompt

Retrieved memories are injected into the system prompt under a dedicated
section:

```
## Known Facts

The following facts are known about the user and context:

- User's name is Gustav
- User's timezone is Europe/Stockholm
- User prefers concise responses
- Last discussed: project deployment to Fly.io on March 20

## Recent Episodes

- [March 19] Helped debug a Rust compilation error in the gateway crate
- [March 18] Set up Telegram channel integration
```

This section is dynamically generated on each turn based on relevance.

## CLI Commands

```bash
# List all memories
magicmerlin memory list

# List memories with details
magicmerlin memory list --verbose

# Search memories
magicmerlin memory search "deployment"

# Add a memory
magicmerlin memory set "key" "value"
magicmerlin memory add "free-form note"

# Delete a memory
magicmerlin memory delete "key"

# Export all memories
magicmerlin memory export > memories.json

# Import memories
magicmerlin memory import < memories.json

# Clear all memories (with confirmation)
magicmerlin memory clear
```

## Privacy and Control

Memory is stored locally on your machine. No memory data is sent to external
services except when:

- Generating vector embeddings (if configured with a remote embedding provider)
- The memory content is included in LLM prompts (as part of normal operation)

You always have full control:

- View all stored memories at any time
- Delete individual facts or clear everything
- Disable auto-extraction if you prefer manual control
- Exclude specific topics from memory with deny lists

```toml
[memory.privacy]
# Never store memories about these topics
deny_patterns = ["password", "secret", "credit card"]
```

## See Also

- [Session Management](session.md) -- Short-term conversation tracking
- [Compaction](compaction.md) -- How session history is compressed
- [Agent Runtime](agent.md) -- How the agent uses memory
- [System Prompt](system-prompt.md) -- Where memories appear in the prompt
- [`magicmerlin memory`](../cli/memory.md) -- CLI command reference

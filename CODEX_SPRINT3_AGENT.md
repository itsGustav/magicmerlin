# Sprint 3 — Agent Engine Depth + Session Compaction

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Agent engine: `agent/src/` — engine.rs, system_prompt.rs, session.rs, queue.rs
Sessions: `sessions/src/lib.rs` (413 lines)

This sprint completes the agent lifecycle: full workspace injection, compaction, multi-agent isolation.

---

## Part 1: Agent Engine Depth

### 1A: Full workspace file injection (agent/src/system_prompt.rs)

The system_prompt builder needs to inject ALL OpenClaw workspace files in proper order.

Implement `WorkspaceInjector`:

```rust
pub struct WorkspaceInjector {
    workspace_root: PathBuf,
    state_dir: PathBuf,       // ~/.magicmerlin/ or ~/.openclaw/
    agent_id: String,
    char_limits: HashMap<&'static str, usize>,
}

impl WorkspaceInjector {
    pub fn new(workspace_root: PathBuf, state_dir: PathBuf, agent_id: String) -> Self {
        let mut char_limits = HashMap::new();
        char_limits.insert("MEMORY.md", 14_000);
        char_limits.insert("TOOLS.md", 4_000);
        char_limits.insert("AGENTS.md", 4_000);
        char_limits.insert("SOUL.md", 4_000);
        char_limits.insert("USER.md", 4_000);
        char_limits.insert("IDENTITY.md", 2_000);
        char_limits.insert("HEARTBEAT.md", 2_000);
        char_limits.insert("daily_memory", 4_000);  // per file
        Self { workspace_root, state_dir, agent_id, char_limits }
    }
    
    /// Returns the full "## Project Context\n..." block to inject into system prompt.
    pub fn build_context_block(&self) -> String {
        let mut out = String::new();
        out.push_str("# Project Context\nThe following project context files have been loaded:\n");
        
        // Inject in this order:
        for filename in &["AGENTS.md", "SOUL.md", "USER.md", "IDENTITY.md", "TOOLS.md", "HEARTBEAT.md"] {
            let path = self.workspace_root.join(filename);
            if let Ok(content) = std::fs::read_to_string(&path) {
                let limit = self.char_limits.get(*filename).copied().unwrap_or(4000);
                let truncated = truncate_chars(&content, limit);
                out.push_str(&format!("## {}\n{}\n", path.display(), truncated));
            } else {
                // BOOTSTRAP.md special case
                if *filename == "BOOTSTRAP.md" {
                    out.push_str(&format!("[MISSING] Expected at: {}\n", path.display()));
                }
            }
        }
        
        // MEMORY.md with 14K limit
        let mem_path = self.workspace_root.join("MEMORY.md");
        if let Ok(content) = std::fs::read_to_string(&mem_path) {
            let limit = 14_000;
            let kept = &content[..content.len().min(limit)];
            let suffix = if content.len() > limit {
                format!("\n…(truncated MEMORY.md: kept {}+{} chars of {})…", limit, 4000, content.len())
            } else {
                String::new()
            };
            out.push_str(&format!("## {}\n{}{}\n", mem_path.display(), kept, suffix));
        }
        
        // Today + yesterday daily memory files
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        for date in &[today, yesterday] {
            let daily_path = self.workspace_root.join("memory").join(format!("{}.md", date));
            if let Ok(content) = std::fs::read_to_string(&daily_path) {
                let truncated = truncate_chars(&content, 4_000);
                out.push_str(&format!("## {}\n{}\n", daily_path.display(), truncated));
            }
        }
        
        out
    }
}

fn truncate_chars(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        s.to_string()
    } else {
        format!("{}…(truncated)", &s[..limit])
    }
}
```

### 1B: Skills discovery injection (agent/src/system_prompt.rs)

```rust
pub struct SkillsInjector {
    skills_dirs: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub location: PathBuf,
}

impl SkillsInjector {
    /// Scan skills directories, parse SKILL.md frontmatter, return all skills.
    pub fn discover(&self) -> Vec<SkillEntry> {
        let mut skills = Vec::new();
        for dir in &self.skills_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let skill_md = entry.path().join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(skill) = parse_skill_md(&skill_md) {
                            skills.push(skill);
                        }
                    }
                }
            }
        }
        skills
    }
    
    /// Build the <available_skills>...</available_skills> XML block.
    pub fn build_xml_block(&self) -> String {
        let skills = self.discover();
        let mut xml = String::from("<available_skills>\n");
        for skill in &skills {
            xml.push_str(&format!(
                "  <skill>\n    <name>{}</name>\n    <description>{}</description>\n    <location>{}</location>\n  </skill>\n",
                skill.name,
                skill.description,
                skill.location.display()
            ));
        }
        xml.push_str("</available_skills>");
        xml
    }
}

fn parse_skill_md(path: &Path) -> Option<SkillEntry> {
    // Parse YAML frontmatter between --- markers
    // Extract name and description fields
}
```

### 1C: Wire into engine.rs

In `agent/src/engine.rs`, the `build_system_prompt()` function should:
1. Call `WorkspaceInjector::build_context_block()`
2. Call `SkillsInjector::build_xml_block()`
3. Append them to the system prompt after the base SOUL.md content

### 1D: Reply tag + silent reply instructions injection

Add static sections to system prompt:
```rust
const REPLY_TAGS_SECTION: &str = r#"## Reply Tags
To request a native reply/quote on supported surfaces, include one tag in your reply:
- Tags must be the very first token: [[reply_to_current]] your reply.
- [[reply_to_current]] replies to the triggering message.
Tags are stripped before sending."#;

const SILENT_REPLY_SECTION: &str = r#"## Silent Replies
When you have nothing to say, respond with ONLY: NO_REPLY
Rules:
- It must be your ENTIRE message — nothing else
- Never append it to an actual response"#;
```

---

## Part 2: Session Compaction

### 2A: JSONL compaction (sessions/src/lib.rs)

Implement `compact_session(session_key: &str, state_dir: &Path) -> Result<CompactionResult>`:

```rust
pub struct CompactionResult {
    pub messages_before: usize,
    pub messages_after: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub memory_extracted: Option<String>,  // content flushed to MEMORY.md
}

pub async fn compact_session(session_key: &str, state_dir: &Path) -> Result<CompactionResult> {
    // 1. Read the session JSONL transcript
    // 2. Count messages + estimate tokens
    // 3. Extract a summary of the first 80% of messages
    //    - Keep the last N messages intact (recent context)
    //    - Summarize older messages into a "session summary" message
    // 4. Write new compact JSONL with: [system_msg, summary_msg, recent_N_messages]
    // 5. Update session metadata (message_count, compacted_at)
    // 6. Return CompactionResult
}
```

### 2B: Context window % tracking (sessions/src/lib.rs)

```rust
pub fn estimate_context_percent(session_key: &str, model: &str, state_dir: &Path) -> Result<f32> {
    // Read transcript JSONL
    // Sum all message content lengths
    // Divide by 4 for rough token estimate
    // Divide by model context window (get from model_catalog)
    // Return as 0.0..1.0 float
}

// Model context windows:
// gpt-4o: 128_000, claude-sonnet: 200_000, claude-opus: 200_000
// gpt-5: 128_000, gemini-1.5-pro: 1_000_000
// default: 128_000
```

### 2C: Pre-compaction memory flush

Before compaction, extract key information to MEMORY.md:
```rust
pub async fn flush_session_to_memory(
    session_key: &str,
    state_dir: &Path,
    workspace_root: &Path,
) -> Result<String> {
    // Read transcript
    // Build a summary prompt: "Summarize the key decisions, facts, and context from this conversation in bullet points."
    // Run through the configured LLM (local/small model preferred)
    // Append to workspace_root/memory/YYYY-MM-DD.md
    // Return the summary
}
```

### 2D: Session file locking

```rust
pub struct SessionLock {
    lock_path: PathBuf,
    pid: u32,
}

impl SessionLock {
    pub fn acquire(session_key: &str, state_dir: &Path) -> Result<Self> {
        // Create {state_dir}/sessions/{key}.lock with PID content
        // If lock exists and PID is still running → return Err(AlreadyLocked)
        // If lock exists but PID is dead → steal the lock
    }
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}
```

### 2E: Auto-compaction trigger in engine.rs

In `agent/src/engine.rs`, after each agent turn:
```rust
// Check context usage
let pct = estimate_context_percent(&session_key, &model, &state_dir)?;
if pct > 0.80 {
    tracing::info!("Context at {:.0}%, triggering compaction", pct * 100.0);
    flush_session_to_memory(&session_key, &state_dir, &workspace_root).await?;
    compact_session(&session_key, &state_dir).await?;
}
```

---

## Rules
- `cargo build --workspace` must pass clean
- Unit tests for: WorkspaceInjector truncation, SkillsInjector XML building, session compaction
- No unwrap() in production paths

## Completion

```bash
openclaw system event --text "Sprint 3 done: full workspace file injection, skills XML block, reply tag instructions, session JSONL compaction, context window tracking, pre-compaction memory flush, session file locking, auto-compaction trigger at 80%" --mode now
```

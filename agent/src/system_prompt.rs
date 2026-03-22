//! System prompt assembly and skill discovery.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AgentError, Result};

const DEFAULT_FILES: [&str; 8] = [
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "HEARTBEAT.md",
    "MEMORY.md",
    "BOOTSTRAP.md",
];

/// Returns per-file character limits for workspace injection.
fn default_char_limits() -> HashMap<&'static str, usize> {
    let mut m = HashMap::new();
    m.insert("MEMORY.md", 14_000);
    m.insert("TOOLS.md", 4_000);
    m.insert("AGENTS.md", 4_000);
    m.insert("SOUL.md", 4_000);
    m.insert("USER.md", 4_000);
    m.insert("IDENTITY.md", 2_000);
    m.insert("HEARTBEAT.md", 2_000);
    m.insert("BOOTSTRAP.md", 4_000);
    m
}

/// Runtime metadata embedded into system prompt.
#[derive(Debug, Clone)]
pub struct PromptRuntimeMetadata {
    /// Current model.
    pub model: String,
    /// Logical channel (terminal/telegram/etc).
    pub channel: String,
    /// Timezone id.
    pub timezone: String,
    /// Agent identifier.
    pub agent_name: String,
    /// Optional provider name.
    pub provider: Option<String>,
    /// Current UTC timestamp.
    pub now_utc: DateTime<Utc>,
    /// Current local timestamp.
    pub now_local: DateTime<Local>,
}

impl PromptRuntimeMetadata {
    /// Creates metadata from core fields using current clock time.
    pub fn now(model: String, channel: String, timezone: String, agent_name: String) -> Self {
        Self {
            model,
            channel,
            timezone,
            agent_name,
            provider: None,
            now_utc: Utc::now(),
            now_local: Local::now(),
        }
    }
}

/// Inbound user/context envelope injected into prompt.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InboundContext {
    /// Optional sender ID.
    pub sender_id: Option<String>,
    /// Optional sender display name.
    pub sender_name: Option<String>,
    /// Chat type (`dm`, `group`, etc).
    pub chat_type: Option<String>,
    /// Reply target summary.
    pub reply_to: Option<String>,
    /// Additional opaque context fields.
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

/// JSON schema descriptor for a callable tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchemaDescriptor {
    /// Tool function name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema parameters.
    pub parameters: Value,
}

/// Skill descriptor with frontmatter metadata.
#[derive(Debug, Clone)]
pub struct SkillDescriptor {
    /// Skill name from frontmatter.
    pub name: String,
    /// Skill description from frontmatter.
    pub description: String,
    /// Path to the SKILL.md file.
    pub path: PathBuf,
}

/// System prompt assembler from workspace and agent files.
#[derive(Debug, Clone)]
pub struct SystemPromptAssembler {
    workspace_dir: PathBuf,
    agent_dir: PathBuf,
    max_chars_per_file: usize,
    char_limits: HashMap<&'static str, usize>,
    include_files: Vec<String>,
}

impl SystemPromptAssembler {
    /// Creates a new system prompt assembler.
    pub fn new(
        workspace_dir: impl AsRef<Path>,
        agent_dir: impl AsRef<Path>,
        max_chars_per_file: usize,
    ) -> Self {
        Self {
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
            agent_dir: agent_dir.as_ref().to_path_buf(),
            max_chars_per_file,
            char_limits: default_char_limits(),
            include_files: DEFAULT_FILES.iter().map(|v| v.to_string()).collect(),
        }
    }

    /// Overrides include-files list.
    pub fn with_include_files(mut self, include_files: Vec<String>) -> Self {
        self.include_files = include_files;
        self
    }

    /// Overrides per-file character limits.
    pub fn with_char_limits(mut self, limits: HashMap<&'static str, usize>) -> Self {
        self.char_limits = limits;
        self
    }

    /// Builds full system prompt text.
    pub fn assemble(&self, metadata: &PromptRuntimeMetadata) -> Result<String> {
        self.assemble_with_context(metadata, &InboundContext::default(), &[])
    }

    /// Builds full system prompt text with inbound context and available tools.
    pub fn assemble_with_context(
        &self,
        metadata: &PromptRuntimeMetadata,
        inbound: &InboundContext,
        tools: &[ToolSchemaDescriptor],
    ) -> Result<String> {
        let mut sections = Vec::new();

        // Runtime metadata
        sections.push(render_runtime_metadata(metadata));

        // Project Context section with workspace files
        sections.push("## Project Context".to_string());
        self.inject_workspace_files(&mut sections)?;

        // Today and yesterday memory files
        self.inject_memory_files(&mut sections)?;

        // Skills discovery with frontmatter
        let skill_dirs = [
            self.workspace_dir.join(".codex/skills"),
            self.agent_dir.join("skills"),
            PathBuf::from("/Users/gustav/.codex/skills"),
        ];
        let skills = discover_skills_with_frontmatter(&skill_dirs)?;
        if !skills.is_empty() {
            sections.push(render_skills_xml_detailed(&skills));
        }

        // Reply tag instructions
        sections.push(render_reply_tag_instructions());

        // Silent reply instructions
        sections.push(render_silent_reply_instructions());

        // Inbound context
        if inbound != &InboundContext::default() {
            sections.push(render_inbound_context(inbound));
        }

        // Tool schemas
        if !tools.is_empty() {
            sections.push(render_tool_schema_block(tools)?);
        }

        Ok(sections.join("\n\n"))
    }

    /// Injects workspace files into sections with per-file limits and proper headers.
    fn inject_workspace_files(&self, sections: &mut Vec<String>) -> Result<()> {
        for file in &self.include_files {
            let max_chars = self
                .char_limits
                .get(file.as_str())
                .copied()
                .unwrap_or(self.max_chars_per_file);

            match self.load_file(file, max_chars)? {
                Some((path, content)) => {
                    sections.push(format!("## {}\n{}", path.display(), content));
                }
                None => {
                    // BOOTSTRAP.md gets a missing marker when absent
                    if file == "BOOTSTRAP.md" {
                        let expected = self.workspace_dir.join(file);
                        sections.push(format!(
                            "[MISSING] Expected at: {}",
                            expected.display()
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Injects today's and yesterday's memory files if they exist.
    fn inject_memory_files(&self, sections: &mut Vec<String>) -> Result<()> {
        let today = Local::now().date_naive();
        let yesterday = today.pred_opt().unwrap_or(today);

        for date in &[yesterday, today] {
            self.try_inject_memory_date(sections, *date)?;
        }
        Ok(())
    }

    fn try_inject_memory_date(
        &self,
        sections: &mut Vec<String>,
        date: NaiveDate,
    ) -> Result<()> {
        let filename = format!("memory/{}.md", date.format("%Y-%m-%d"));
        let from_agent = self.agent_dir.join(&filename);
        let from_workspace = self.workspace_dir.join(&filename);
        let path = if from_agent.exists() {
            from_agent
        } else if from_workspace.exists() {
            from_workspace
        } else {
            return Ok(());
        };

        let raw = fs::read_to_string(&path).map_err(|source| AgentError::Io {
            path: path.clone(),
            source,
        })?;
        let clipped = smart_clip_markdown(&raw, self.max_chars_per_file);
        sections.push(format!("## {}\n{}", path.display(), clipped));
        Ok(())
    }

    /// Loads a workspace/agent file with the given character limit.
    /// Returns the resolved path and clipped content.
    fn load_file(&self, name: &str, max_chars: usize) -> Result<Option<(PathBuf, String)>> {
        let from_agent = self.agent_dir.join(name);
        let from_workspace = self.workspace_dir.join(name);
        let path = if from_agent.exists() {
            from_agent
        } else {
            from_workspace
        };

        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&path).map_err(|source| AgentError::Io {
            path: path.clone(),
            source,
        })?;

        let was_truncated = raw.chars().count() > max_chars;
        let clipped = smart_clip_markdown(&raw, max_chars);

        // MEMORY.md gets a special truncation suffix
        let content = if name == "MEMORY.md" && was_truncated {
            let kept = clipped.chars().count();
            format!("{clipped}\n…(truncated MEMORY.md: kept {kept} chars)…")
        } else {
            clipped
        };

        Ok(Some((path, content)))
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

fn render_runtime_metadata(metadata: &PromptRuntimeMetadata) -> String {
    let provider = metadata
        .provider
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    format!(
        "<runtime>\ndate_local={}\ndate_utc={}\ntimezone={}\nmodel={}\nprovider={}\nchannel={}\nagent={}\n</runtime>",
        metadata.now_local.format("%Y-%m-%d %H:%M:%S %:z"),
        metadata.now_utc.format("%Y-%m-%d %H:%M:%S UTC"),
        metadata.timezone,
        metadata.model,
        provider,
        metadata.channel,
        metadata.agent_name,
    )
}

fn render_inbound_context(inbound: &InboundContext) -> String {
    let mut lines = vec!["<inbound_context>".to_string()];

    if let Some(sender_id) = &inbound.sender_id {
        lines.push(format!("sender_id={sender_id}"));
    }
    if let Some(sender_name) = &inbound.sender_name {
        lines.push(format!("sender_name={sender_name}"));
    }
    if let Some(chat_type) = &inbound.chat_type {
        lines.push(format!("chat_type={chat_type}"));
    }
    if let Some(reply_to) = &inbound.reply_to {
        lines.push(format!("reply_to={reply_to}"));
    }

    if !inbound.extra.is_empty() {
        let serialized = serde_json::to_string_pretty(&inbound.extra)
            .unwrap_or_else(|_| "{}".to_string());
        lines.push("extra_json=<<<JSON".to_string());
        lines.push(serialized);
        lines.push("JSON".to_string());
    }

    lines.push("</inbound_context>".to_string());
    lines.join("\n")
}

fn render_reply_tag_instructions() -> String {
    "\
## Reply Tags
When replying to a specific message, prepend one of these tags as the FIRST token in your response:
- `[[reply_to_current]]` — replies to the message that triggered this turn
- `[[reply_to:<id>]]` — replies to a specific message by its platform ID

Rules:
- The tag must be the very first token in your response
- Only use one reply tag per message
- If no tag is present, the reply is sent as a new message"
        .to_string()
}

fn render_silent_reply_instructions() -> String {
    "\
## Silent Replies
When you have nothing to say, respond with ONLY: NO_REPLY
Rules:
- It must be your ENTIRE message — nothing else
- Never append it to an actual response

For heartbeat checks, respond with ONLY: HEARTBEAT_OK
Rules:
- Used when the heartbeat system checks if you are alive
- It must be your ENTIRE message — nothing else
- Never use HEARTBEAT_OK in response to user messages"
        .to_string()
}

fn render_tool_schema_block(tools: &[ToolSchemaDescriptor]) -> Result<String> {
    let serialized = serde_json::to_string_pretty(tools)?;
    Ok(format!("<tool_schemas>\n{serialized}\n</tool_schemas>"))
}

// ---------------------------------------------------------------------------
// Skills discovery
// ---------------------------------------------------------------------------

/// Discovers skill descriptors by finding `SKILL.md` under directories.
pub fn discover_skills(dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        walk_skill_dir(dir, &mut out)?;
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Discovers skills and parses name/description from SKILL.md frontmatter.
pub fn discover_skills_with_frontmatter(dirs: &[PathBuf]) -> Result<Vec<SkillDescriptor>> {
    let paths = discover_skills(dirs)?;
    let mut skills = Vec::new();

    for path in paths {
        let content = fs::read_to_string(&path).map_err(|source| AgentError::Io {
            path: path.clone(),
            source,
        })?;
        let (name, description) = parse_skill_frontmatter(&content, &path);
        skills.push(SkillDescriptor {
            name,
            description,
            path,
        });
    }

    Ok(skills)
}

fn walk_skill_dir(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root).map_err(|source| AgentError::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| AgentError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            walk_skill_dir(&path, out)?;
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
            out.push(path);
        }
    }
    Ok(())
}

/// Parses simple YAML-like frontmatter from a SKILL.md file.
fn parse_skill_frontmatter(content: &str, path: &Path) -> (String, String) {
    let default_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    if !content.starts_with("---") {
        return (default_name, String::new());
    }

    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(pos) => pos,
        None => return (default_name, String::new()),
    };

    let frontmatter = &rest[..end];
    let mut name = default_name;
    let mut description = String::new();

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = val.trim().trim_matches('"').trim_matches('\'').to_string();
        } else if let Some(val) = line.strip_prefix("description:") {
            description = val.trim().trim_matches('"').trim_matches('\'').to_string();
        }
    }

    (name, description)
}

/// Renders skills with full frontmatter metadata as XML.
fn render_skills_xml_detailed(skills: &[SkillDescriptor]) -> String {
    let mut out = String::from("<available_skills>");
    for skill in skills {
        out.push_str("\n  <skill>");
        out.push_str(&format!("\n    <name>{}</name>", skill.name));
        out.push_str(&format!(
            "\n    <description>{}</description>",
            skill.description
        ));
        out.push_str(&format!(
            "\n    <location>{}</location>",
            skill.path.display()
        ));
        out.push_str("\n  </skill>");
    }
    out.push_str("\n</available_skills>");
    out
}

// ---------------------------------------------------------------------------
// Markdown clipping
// ---------------------------------------------------------------------------

/// Clips markdown content while preserving heading structure and recent context.
pub fn smart_clip_markdown(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    if max_chars < 128 {
        return simple_clip(input, max_chars);
    }

    let mut headers = Vec::new();
    let mut recent_lines = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            headers.push(line.to_string());
        }
        recent_lines.push(line.to_string());
        if recent_lines.len() > 120 {
            recent_lines.remove(0);
        }
    }

    let header_block = if headers.is_empty() {
        String::new()
    } else {
        format!("{}\n\n", headers.join("\n"))
    };

    let recent_block = recent_lines.join("\n");
    let scaffold = format!("{header_block}{recent_block}");
    if scaffold.chars().count() <= max_chars {
        return scaffold;
    }

    simple_clip(&scaffold, max_chars)
}

fn simple_clip(input: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in input.chars().take(max_chars) {
        out.push(ch);
    }
    out.push_str("\n[truncated]");
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_prompt_and_truncates() {
        let temp = tempfile::tempdir().expect("tmp");
        // AGENTS.md has a per-file limit of 4000, so write content exceeding that
        let big_content = "x".repeat(5_000);
        std::fs::write(temp.path().join("AGENTS.md"), &big_content).expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 3);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "openai/gpt-5.2".to_string(),
                "terminal".to_string(),
                "America/New_York".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("xxxx"));
        assert!(prompt.contains("[truncated]"));
    }

    #[test]
    fn smart_clip_preserves_headers() {
        let input = "# H1\nbody\n## H2\nmore\n".repeat(80);
        let clipped = smart_clip_markdown(&input, 500);
        assert!(clipped.contains("# H1"));
        assert!(clipped.contains("## H2"));
    }

    #[test]
    fn renders_inbound_and_tools() {
        let temp = tempfile::tempdir().expect("tmp");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let inbound = InboundContext {
            sender_id: Some("u1".to_string()),
            sender_name: Some("Alice".to_string()),
            chat_type: Some("group".to_string()),
            reply_to: Some("msg-22".to_string()),
            extra: {
                let mut m = BTreeMap::new();
                m.insert("thread".to_string(), Value::String("x".to_string()));
                m
            },
        };

        let tools = vec![ToolSchemaDescriptor {
            name: "exec".to_string(),
            description: "Run shell command".to_string(),
            parameters: serde_json::json!({"type":"object"}),
        }];

        let prompt = assembler
            .assemble_with_context(
                &PromptRuntimeMetadata::now(
                    "openai/gpt-5.2".to_string(),
                    "terminal".to_string(),
                    "America/New_York".to_string(),
                    "merlin".to_string(),
                ),
                &inbound,
                &tools,
            )
            .expect("prompt");

        assert!(prompt.contains("<inbound_context>"));
        assert!(prompt.contains("<tool_schemas>"));
        assert!(prompt.contains("Alice"));
    }

    #[test]
    fn injects_project_context_and_instruction_sections() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("SOUL.md"), "soul content here").expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");

        assert!(prompt.contains("## Project Context"));
        assert!(prompt.contains("## Reply Tags"));
        assert!(prompt.contains("## Silent Replies"));
        assert!(prompt.contains("soul content here"));
        assert!(prompt.contains("[[reply_to_current]]"));
        assert!(prompt.contains("NO_REPLY"));
        assert!(prompt.contains("HEARTBEAT_OK"));
    }

    #[test]
    fn bootstrap_missing_marker() {
        let temp = tempfile::tempdir().expect("tmp");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("[MISSING] Expected at:"));
        assert!(prompt.contains("BOOTSTRAP.md"));
    }

    #[test]
    fn bootstrap_present_no_marker() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("BOOTSTRAP.md"), "bootstrap content").expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("bootstrap content"));
        assert!(!prompt.contains("[MISSING]"));
    }

    #[test]
    fn memory_truncation_with_suffix() {
        let temp = tempfile::tempdir().expect("tmp");
        let big_memory = "x".repeat(20_000);
        std::fs::write(temp.path().join("MEMORY.md"), &big_memory).expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("truncated MEMORY.md: kept"));
    }

    #[test]
    fn memory_not_truncated_no_suffix() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("MEMORY.md"), "short memory").expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("short memory"));
        assert!(!prompt.contains("truncated MEMORY.md"));
    }

    #[test]
    fn skills_with_frontmatter() {
        let temp = tempfile::tempdir().expect("tmp");
        let skills_dir = temp.path().join(".codex/skills/weather");
        std::fs::create_dir_all(&skills_dir).expect("mkdir");
        std::fs::write(
            skills_dir.join("SKILL.md"),
            "---\nname: weather\ndescription: Get current weather\n---\nContent here",
        )
        .expect("write");

        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("<name>weather</name>"));
        assert!(prompt.contains("<description>Get current weather</description>"));
        assert!(prompt.contains("<location>"));
    }

    #[test]
    fn skills_without_frontmatter_uses_dirname() {
        let temp = tempfile::tempdir().expect("tmp");
        let skills_dir = temp.path().join(".codex/skills/calculator");
        std::fs::create_dir_all(&skills_dir).expect("mkdir");
        std::fs::write(skills_dir.join("SKILL.md"), "Just content, no frontmatter")
            .expect("write");

        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("<name>calculator</name>"));
    }

    #[test]
    fn parse_skill_frontmatter_works() {
        let content =
            "---\nname: test-skill\ndescription: A test skill\n---\n# Test\nBody content";
        let path = PathBuf::from("/skills/fallback/SKILL.md");
        let (name, desc) = parse_skill_frontmatter(content, &path);
        assert_eq!(name, "test-skill");
        assert_eq!(desc, "A test skill");
    }

    #[test]
    fn parse_skill_frontmatter_missing() {
        let content = "# No frontmatter here";
        let path = PathBuf::from("/skills/myskill/SKILL.md");
        let (name, desc) = parse_skill_frontmatter(content, &path);
        assert_eq!(name, "myskill");
        assert_eq!(desc, "");
    }

    #[test]
    fn workspace_file_headers_include_path() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("IDENTITY.md"), "I am merlin").expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 4_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        // File is injected with path-based header
        let identity_path = temp.path().join("IDENTITY.md");
        assert!(prompt.contains(&format!("## {}", identity_path.display())));
        assert!(prompt.contains("I am merlin"));
    }

    #[test]
    fn per_file_char_limits_truncate_correctly() {
        let temp = tempfile::tempdir().expect("tmp");
        // IDENTITY.md has a 2000 char limit by default
        let big_identity = "x".repeat(3_000);
        std::fs::write(temp.path().join("IDENTITY.md"), &big_identity).expect("write");
        // AGENTS.md has a 4000 char limit by default
        let big_agents = "y".repeat(5_000);
        std::fs::write(temp.path().join("AGENTS.md"), &big_agents).expect("write");

        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 8_000);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "test".to_string(),
                "terminal".to_string(),
                "UTC".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");

        // IDENTITY.md should be truncated at 2000 chars (per-file limit)
        assert!(prompt.contains("[truncated]"));
        // Both files should be present
        assert!(prompt.contains("IDENTITY.md"));
        assert!(prompt.contains("AGENTS.md"));
    }

    #[test]
    fn default_char_limits_are_correct() {
        let limits = default_char_limits();
        assert_eq!(limits.get("MEMORY.md"), Some(&14_000));
        assert_eq!(limits.get("TOOLS.md"), Some(&4_000));
        assert_eq!(limits.get("AGENTS.md"), Some(&4_000));
        assert_eq!(limits.get("SOUL.md"), Some(&4_000));
        assert_eq!(limits.get("USER.md"), Some(&4_000));
        assert_eq!(limits.get("IDENTITY.md"), Some(&2_000));
        assert_eq!(limits.get("HEARTBEAT.md"), Some(&2_000));
    }
}

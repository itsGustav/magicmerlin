//! System prompt assembly and skill discovery.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AgentError, Result};

const DEFAULT_FILES: [&str; 8] = [
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "MEMORY.md",
    "HEARTBEAT.md",
    "BOOTSTRAP.md",
];

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

/// System prompt assembler from workspace and agent files.
#[derive(Debug, Clone)]
pub struct SystemPromptAssembler {
    workspace_dir: PathBuf,
    agent_dir: PathBuf,
    max_chars_per_file: usize,
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
            include_files: DEFAULT_FILES.iter().map(|v| v.to_string()).collect(),
        }
    }

    /// Overrides include-files list.
    pub fn with_include_files(mut self, include_files: Vec<String>) -> Self {
        self.include_files = include_files;
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
        sections.push(render_runtime_metadata(metadata));

        for file in &self.include_files {
            if let Some(content) = self.load_file(file)? {
                sections.push(format!("<{file}>\n{content}\n</{file}>"));
            }
        }

        let skills = discover_skills(&[
            self.workspace_dir.join(".codex/skills"),
            self.agent_dir.join("skills"),
            PathBuf::from("/Users/gustav/.codex/skills"),
        ])?;
        if !skills.is_empty() {
            sections.push(render_skills_xml(&skills));
        }

        if inbound != &InboundContext::default() {
            sections.push(render_inbound_context(inbound));
        }

        if !tools.is_empty() {
            sections.push(render_tool_schema_block(tools)?);
        }

        Ok(sections.join("\n\n"))
    }

    fn load_file(&self, name: &str) -> Result<Option<String>> {
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
        let clipped = smart_clip_markdown(&raw, self.max_chars_per_file);
        Ok(Some(clipped))
    }
}

fn render_runtime_metadata(metadata: &PromptRuntimeMetadata) -> String {
    let provider = metadata.provider.clone().unwrap_or_else(|| "unknown".to_string());
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

fn render_skills_xml(skills: &[PathBuf]) -> String {
    let mut out = String::from("<available_skills>");
    for path in skills {
        let skill_name = path
            .parent()
            .and_then(|v| v.file_name())
            .and_then(|v| v.to_str())
            .unwrap_or("unknown");
        out.push_str(&format!("\n<skill name=\"{skill_name}\">{}\n</skill>", path.display()));
    }
    out.push_str("\n</available_skills>");
    out
}

fn render_tool_schema_block(tools: &[ToolSchemaDescriptor]) -> Result<String> {
    let serialized = serde_json::to_string_pretty(tools)?;
    Ok(format!("<tool_schemas>\n{serialized}\n</tool_schemas>"))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_prompt_and_truncates() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("AGENTS.md"), "abcdef").expect("write");
        let assembler = SystemPromptAssembler::new(temp.path(), temp.path(), 3);
        let prompt = assembler
            .assemble(&PromptRuntimeMetadata::now(
                "openai/gpt-5.2".to_string(),
                "terminal".to_string(),
                "America/New_York".to_string(),
                "merlin".to_string(),
            ))
            .expect("assemble");
        assert!(prompt.contains("abc"));
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
}

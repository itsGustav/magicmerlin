//! System prompt assembly and skill discovery.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::error::{AgentError, Result};

const DEFAULT_FILES: [&str; 7] = [
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "MEMORY.md",
    "HEARTBEAT.md",
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
}

/// System prompt assembler from workspace and agent files.
#[derive(Debug, Clone)]
pub struct SystemPromptAssembler {
    workspace_dir: PathBuf,
    agent_dir: PathBuf,
    max_chars_per_file: usize,
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
        }
    }

    /// Builds full system prompt text.
    pub fn assemble(&self, metadata: &PromptRuntimeMetadata) -> Result<String> {
        let mut sections = Vec::new();
        sections.push(format!(
            "<runtime>\ndate={}\ntimezone={}\nmodel={}\nchannel={}\n</runtime>",
            Local::now().format("%Y-%m-%d"),
            metadata.timezone,
            metadata.model,
            metadata.channel
        ));

        for file in DEFAULT_FILES {
            if let Some(content) = self.load_file(file)? {
                sections.push(format!("<{file}>\n{content}\n</{file}>"));
            }
        }

        let skills = discover_skills(&[
            self.workspace_dir.join(".codex/skills"),
            self.agent_dir.join("skills"),
        ])?;
        if !skills.is_empty() {
            sections.push(format!(
                "<available_skills>\n{}\n</available_skills>",
                skills
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
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
        let clipped = clip_text(&raw, self.max_chars_per_file);
        Ok(Some(clipped))
    }
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

fn clip_text(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }
    let mut out = String::new();
    for ch in input.chars().take(max) {
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
            .assemble(&PromptRuntimeMetadata {
                model: "openai/gpt-5.2".to_string(),
                channel: "terminal".to_string(),
                timezone: "America/New_York".to_string(),
            })
            .expect("assemble");
        assert!(prompt.contains("abc"));
        assert!(prompt.contains("[truncated]"));
    }
}

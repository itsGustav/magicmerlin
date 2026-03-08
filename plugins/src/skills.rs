use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Skill metadata parsed from `SKILL.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    /// Skill identifier.
    pub name: String,
    /// Human-readable summary.
    pub description: String,
    /// Dependency list from metadata.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Required environment variables.
    #[serde(default)]
    pub required_env: Vec<String>,
    /// Preferred environment variable hints.
    #[serde(default)]
    pub primary_env: Vec<String>,
    /// Required system binaries.
    #[serde(default)]
    pub required_binaries: Vec<String>,
    /// Optional script path for execution.
    #[serde(default)]
    pub script: Option<PathBuf>,
    /// Source SKILL.md path.
    pub source: PathBuf,
}

/// Dependency check output for one skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillDependencyStatus {
    /// Skill identifier.
    pub skill: String,
    /// Missing required environment variables.
    pub missing_required_env: Vec<String>,
    /// Missing primary environment variables.
    pub missing_primary_env: Vec<String>,
    /// Missing required binaries.
    pub missing_binaries: Vec<String>,
}

/// Consolidated dependency report across discovered skills.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependencyReport {
    /// Per-skill status entries.
    pub skills: Vec<SkillDependencyStatus>,
}

/// Discovers skills from workspace and bundled install locations.
pub fn discover_skills(extra_roots: &[PathBuf]) -> Result<Vec<Skill>> {
    let mut roots = default_skill_roots();
    roots.extend(extra_roots.iter().cloned());

    let mut skills = Vec::new();
    let mut seen = BTreeSet::new();

    for root in roots {
        if !root.exists() {
            continue;
        }
        discover_skill_files(&root, &mut skills)?;
    }

    skills.retain(|skill| seen.insert(skill.name.clone()));
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

fn discover_skill_files(root: &Path, out: &mut Vec<Skill>) -> Result<()> {
    if root.is_file() {
        if root.file_name().is_some_and(|name| name == "SKILL.md") {
            out.push(load_skill_file(root)?);
        }
        return Ok(());
    }

    let entries = fs::read_dir(root).with_context(|| format!("read {}", root.display()))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("read entry in {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            discover_skill_files(&path, out)?;
        } else if path.file_name().is_some_and(|name| name == "SKILL.md") {
            out.push(load_skill_file(&path)?);
        }
    }

    Ok(())
}

fn load_skill_file(path: &Path) -> Result<Skill> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;

    let mut name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-skill")
        .to_string();
    let mut description = String::new();
    let mut requires = Vec::new();
    let mut required_env = Vec::new();
    let mut primary_env = Vec::new();
    let mut required_binaries = Vec::new();
    let mut script: Option<PathBuf> = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("# ") {
            name = value.trim().to_string();
            continue;
        }

        if description.is_empty() && !trimmed.is_empty() && !trimmed.starts_with('#') {
            description = trimmed.to_string();
        }

        parse_csv_field(trimmed, "requires:", &mut requires);
        parse_csv_field(trimmed, "requiredEnv:", &mut required_env);
        parse_csv_field(trimmed, "primaryEnv:", &mut primary_env);
        parse_csv_field(trimmed, "requiredBinaries:", &mut required_binaries);

        if let Some(value) = trimmed.strip_prefix("script:") {
            let relative = value.trim();
            if !relative.is_empty() {
                script = Some(resolve_skill_relative(path, relative));
            }
        }
    }

    if description.is_empty() {
        description = "No description provided".to_string();
    }

    Ok(Skill {
        name,
        description,
        requires,
        required_env,
        primary_env,
        required_binaries,
        script,
        source: path.to_path_buf(),
    })
}

fn parse_csv_field(line: &str, key: &str, out: &mut Vec<String>) {
    if let Some(value) = line.strip_prefix(key) {
        out.extend(
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(std::string::ToString::to_string),
        );
    }
}

fn resolve_skill_relative(skill_file: &Path, raw: &str) -> PathBuf {
    let base = skill_file.parent().unwrap_or_else(|| Path::new("."));
    base.join(raw)
}

/// Produces XML for system-prompt skill injection.
pub fn skills_xml_block(skills: &[Skill]) -> String {
    let mut out = String::from("<available_skills>\n");
    for skill in skills {
        let description = escape_xml(&skill.description);
        let requires = if skill.requires.is_empty() {
            String::new()
        } else {
            format!(" requires=\"{}\"", escape_xml(&skill.requires.join(",")))
        };
        out.push_str(&format!(
            "  <skill name=\"{}\"{}>{}</skill>\n",
            escape_xml(&skill.name),
            requires,
            description
        ));
    }
    out.push_str("</available_skills>");
    out
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Executes a skill script and captures stdout.
pub fn execute_skill_script(skill: &Skill, args: &[String]) -> Result<String> {
    let script = skill
        .script
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("skill {} has no script", skill.name))?;

    let output = Command::new(script)
        .args(args)
        .output()
        .with_context(|| format!("execute {}", script.display()))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "script {} failed with status {}",
            script.display(),
            output.status
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Validates environment and binary dependencies for each skill.
pub fn check_skill_dependencies(skills: &[Skill]) -> DependencyReport {
    let statuses = skills
        .iter()
        .map(|skill| {
            let missing_required_env = skill
                .required_env
                .iter()
                .filter(|key| env::var(key).ok().as_deref().map_or(true, str::is_empty))
                .cloned()
                .collect();

            let missing_primary_env = skill
                .primary_env
                .iter()
                .filter(|key| env::var(key).ok().as_deref().map_or(true, str::is_empty))
                .cloned()
                .collect();

            let missing_binaries = skill
                .required_binaries
                .iter()
                .filter(|bin| !binary_exists(bin))
                .cloned()
                .collect();

            SkillDependencyStatus {
                skill: skill.name.clone(),
                missing_required_env,
                missing_primary_env,
                missing_binaries,
            }
        })
        .collect();

    DependencyReport { skills: statuses }
}

fn binary_exists(bin: &str) -> bool {
    if bin.contains(std::path::MAIN_SEPARATOR) {
        return Path::new(bin).exists();
    }

    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths).any(|path| {
                let candidate = path.join(bin);
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

fn default_skill_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home);
        let openclaw = home_path.join(".openclaw");
        if openclaw.exists() {
            if let Ok(entries) = fs::read_dir(&openclaw) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let matches_workspace = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("workspace-"));
                    if matches_workspace {
                        roots.push(path.join("skills"));
                    }
                }
            }
            roots.push(openclaw.join("skills"));
            roots.push(openclaw.join("clawhub/skills"));
        }
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_skill_metadata() {
        let temp = tempfile::tempdir().expect("tempdir");
        let skill_dir = temp.path().join("my-skill");
        fs::create_dir_all(&skill_dir).expect("skill dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            "# my-skill\n\nA test skill\nrequires: foo, bar\nrequiredEnv: API_KEY\nprimaryEnv: MODE\nrequiredBinaries: sh\nscript: run.sh\n",
        )
        .expect("write skill");

        let skills = discover_skills(&[temp.path().to_path_buf()]).expect("discover");
        let parsed = skills
            .iter()
            .find(|skill| skill.name == "my-skill")
            .expect("my-skill present");
        assert!(parsed.requires.contains(&"foo".to_string()));
        assert_eq!(
            parsed
                .script
                .as_ref()
                .map(|p| p.file_name().and_then(|n| n.to_str())),
            Some(Some("run.sh"))
        );
    }

    #[test]
    fn xml_block_contains_skill_tags() {
        let skill = Skill {
            name: "s1".to_string(),
            description: "desc".to_string(),
            requires: vec!["dep".to_string()],
            required_env: vec![],
            primary_env: vec![],
            required_binaries: vec![],
            script: None,
            source: PathBuf::from("/tmp/SKILL.md"),
        };
        let xml = skills_xml_block(&[skill]);
        assert!(xml.contains("<available_skills>"));
        assert!(xml.contains("name=\"s1\""));
        assert!(xml.contains("requires=\"dep\""));
    }

    #[test]
    fn dependency_check_reports_missing() {
        let skill = Skill {
            name: "s1".to_string(),
            description: "desc".to_string(),
            requires: vec![],
            required_env: vec!["__MISSING_ENV__".to_string()],
            primary_env: vec!["__MISSING_PRIMARY__".to_string()],
            required_binaries: vec!["__unlikely_binary__".to_string()],
            script: None,
            source: PathBuf::from("/tmp/SKILL.md"),
        };

        let report = check_skill_dependencies(&[skill]);
        assert_eq!(report.skills.len(), 1);
        assert!(report.skills[0]
            .missing_required_env
            .contains(&"__MISSING_ENV__".to_string()));
        assert!(report.skills[0]
            .missing_primary_env
            .contains(&"__MISSING_PRIMARY__".to_string()));
    }

    #[test]
    fn execute_skill_script_returns_stdout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let script = temp.path().join("run.sh");
        fs::write(&script, "#!/bin/sh\necho ok\n").expect("write script");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script).expect("metadata").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script, perms).expect("chmod");
        }

        let skill = Skill {
            name: "scripted".to_string(),
            description: "d".to_string(),
            requires: vec![],
            required_env: vec![],
            primary_env: vec![],
            required_binaries: vec![],
            script: Some(script),
            source: temp.path().join("SKILL.md"),
        };

        let output = execute_skill_script(&skill, &[]).expect("run");
        assert_eq!(output, "ok");
    }
}

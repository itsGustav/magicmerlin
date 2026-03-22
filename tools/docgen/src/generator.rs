use crate::templates;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DocsIndex {
    #[serde(rename = "generatedAt")]
    #[allow(dead_code)]
    pub generated_at: String,
    pub pages: Vec<RawPage>,
}

#[derive(Debug, Deserialize)]
pub struct RawPage {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct DocPage {
    pub title: String,
    pub section: String,
    pub subsection: Option<String>,
    pub filename: String,
    pub rel_path: String,
}

pub struct GenerateStats {
    pub total_pages: usize,
    pub sections: BTreeMap<String, usize>,
}

/// Parse a URL like "https://docs.openclaw.ai/cli/agent.md" into a DocPage.
fn parse_page(raw: &RawPage) -> DocPage {
    let path = raw
        .url
        .strip_prefix("https://docs.openclaw.ai/")
        .unwrap_or(&raw.url);

    let parts: Vec<&str> = path.split('/').collect();

    let (section, subsection, filename) = match parts.len() {
        1 => {
            let fname = parts[0];
            ("root".to_string(), None, fname.to_string())
        }
        2 => {
            let sec = parts[0].to_string();
            let fname = parts[1].to_string();
            (sec, None, fname)
        }
        _ => {
            let sec = parts[0].to_string();
            let subsec = parts[1..parts.len() - 1].join("/");
            let fname = parts[parts.len() - 1].to_string();
            (sec, Some(subsec), fname)
        }
    };

    let title = if raw.title == "null" || raw.title.is_empty() {
        // Derive title from filename
        let stem = filename.trim_end_matches(".md");
        stem.replace('-', " ")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        raw.title.clone()
    };

    let rel_path = path.to_string();

    DocPage {
        title,
        section,
        subsection,
        filename,
        rel_path,
    }
}

pub fn generate_all(index_path: &Path, out_dir: &Path) -> Result<GenerateStats> {
    let raw = std::fs::read_to_string(index_path)
        .with_context(|| format!("reading index from {}", index_path.display()))?;
    let index: DocsIndex = serde_json::from_str(&raw).context("parsing docs index JSON")?;

    std::fs::create_dir_all(out_dir)?;

    let pages: Vec<DocPage> = index.pages.iter().map(parse_page).collect();
    let mut sections: BTreeMap<String, usize> = BTreeMap::new();

    for page in &pages {
        let content = templates::render_page(page);
        let out_path = out_dir.join(&page.rel_path);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out_path, content)
            .with_context(|| format!("writing {}", out_path.display()))?;

        *sections.entry(page.section.clone()).or_insert(0) += 1;
    }

    // Generate index.md
    let index_content = templates::render_index(&pages);
    std::fs::write(out_dir.join("index.md"), index_content)?;

    // Generate mkdocs.yml
    let mkdocs = generate_mkdocs_yml(&pages);
    std::fs::write(out_dir.join("mkdocs.yml"), mkdocs)?;

    // Generate docs coverage summary
    let coverage = generate_coverage_json(&sections, pages.len());
    let coverage_path = index_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("docs_coverage_summary.json");
    std::fs::write(&coverage_path, coverage)?;

    Ok(GenerateStats {
        total_pages: pages.len(),
        sections,
    })
}

fn generate_mkdocs_yml(pages: &[DocPage]) -> String {
    // Group pages by section, then subsection
    let mut nav_sections: BTreeMap<String, Vec<&DocPage>> = BTreeMap::new();
    for page in pages {
        nav_sections
            .entry(page.section.clone())
            .or_default()
            .push(page);
    }

    let section_display_names: BTreeMap<&str, &str> = [
        ("root", "General"),
        ("start", "Getting Started"),
        ("install", "Installation"),
        ("cli", "CLI Reference"),
        ("gateway", "Gateway"),
        ("tools", "Tools"),
        ("channels", "Chat Channels"),
        ("providers", "Model Providers"),
        ("concepts", "Concepts"),
        ("platforms", "Platforms"),
        ("nodes", "Nodes"),
        ("reference", "Reference"),
        ("automation", "Automation"),
        ("help", "Help & Troubleshooting"),
        ("plugins", "Plugins"),
        ("web", "Web & Dashboard"),
        ("security", "Security"),
        ("experiments", "Experiments"),
        ("debug", "Debugging"),
        ("design", "Design"),
        ("diagnostics", "Diagnostics"),
        ("api-reference", "API Reference"),
    ]
    .into_iter()
    .collect();

    // Build nav section order
    let section_order = [
        "start",
        "install",
        "concepts",
        "cli",
        "gateway",
        "tools",
        "channels",
        "providers",
        "platforms",
        "nodes",
        "automation",
        "plugins",
        "web",
        "reference",
        "help",
        "security",
        "experiments",
        "debug",
        "design",
        "diagnostics",
        "api-reference",
        "root",
    ];

    let mut nav_yaml = String::new();
    nav_yaml.push_str("  - Home: index.md\n");

    for sec in &section_order {
        if let Some(section_pages) = nav_sections.get(*sec) {
            let display = section_display_names.get(sec).copied().unwrap_or(sec);
            nav_yaml.push_str(&format!("  - {}:\n", display));
            for page in section_pages {
                nav_yaml.push_str(&format!("    - {}: {}\n", page.title, page.rel_path));
            }
        }
    }

    format!(
        r#"site_name: Magic Merlin Documentation
site_description: The Rust-native OpenClaw-compatible AI agent runtime
site_url: https://docs.magicmerlin.dev

theme:
  name: material
  palette:
    - scheme: slate
      primary: deep purple
      accent: amber
      toggle:
        icon: material/brightness-4
        name: Switch to light mode
    - scheme: default
      primary: deep purple
      accent: amber
      toggle:
        icon: material/brightness-7
        name: Switch to dark mode
  features:
    - navigation.tabs
    - navigation.sections
    - navigation.expand
    - navigation.top
    - search.suggest
    - search.highlight
    - content.code.copy
    - content.tabs.link

markdown_extensions:
  - admonition
  - pymdownx.details
  - pymdownx.superfences
  - pymdownx.tabbed:
      alternate_style: true
  - pymdownx.highlight:
      anchor_linenums: true
  - pymdownx.inlinehilite
  - pymdownx.snippets
  - attr_list
  - md_in_html
  - toc:
      permalink: true

plugins:
  - search

nav:
{nav_yaml}
"#
    )
}

fn generate_coverage_json(sections: &BTreeMap<String, usize>, total: usize) -> String {
    let now = chrono::Utc::now().to_rfc3339();
    let mut section_entries = String::new();
    for (sec, count) in sections {
        if !section_entries.is_empty() {
            section_entries.push_str(",\n");
        }
        section_entries.push_str(&format!(
            "    \"{}/\": {{ \"done\": {}, \"partial\": 0, \"todo\": 0 }}",
            sec, count
        ));
    }
    format!(
        r#"{{
  "generatedAt": "{}",
  "totalPages": {},
  "generated": {},
  "sections": {{
{}
  }}
}}"#,
        now, total, total, section_entries
    )
}

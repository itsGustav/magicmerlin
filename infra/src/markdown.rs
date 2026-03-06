//! Markdown parsing helpers for sections, frontmatter, and plain text rendering.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Markdown section extracted by heading traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownSection {
    /// Heading depth (`1` = H1).
    pub level: u32,
    /// Heading title text.
    pub title: String,
    /// Section body text until next heading.
    pub body: String,
}

/// Frontmatter payload and content body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    /// Optional YAML payload between `---` fences.
    pub yaml: Option<String>,
    /// Remaining markdown body.
    pub body: String,
}

/// Extracts YAML frontmatter from markdown if present.
pub fn extract_frontmatter(markdown: &str) -> Frontmatter {
    let mut lines = markdown.lines();
    if lines.next() != Some("---") {
        return Frontmatter {
            yaml: None,
            body: markdown.to_string(),
        };
    }

    let mut yaml_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_yaml = true;

    for line in lines {
        if in_yaml && line == "---" {
            in_yaml = false;
            continue;
        }
        if in_yaml {
            yaml_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    if in_yaml {
        Frontmatter {
            yaml: None,
            body: markdown.to_string(),
        }
    } else {
        Frontmatter {
            yaml: Some(yaml_lines.join("\n")),
            body: body_lines.join("\n"),
        }
    }
}

/// Converts markdown to best-effort plain text.
pub fn strip_markdown_to_text(markdown: &str) -> String {
    let mut out = String::new();
    for event in Parser::new(markdown) {
        match event {
            Event::Text(text) | Event::Code(text) => out.push_str(&text),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            _ => {}
        }
    }
    out
}

/// Parses markdown into heading-based sections.
pub fn parse_sections(markdown: &str) -> Vec<MarkdownSection> {
    let mut sections = Vec::new();
    let mut current_title = String::new();
    let mut current_level = 0u32;
    let mut current_body = String::new();
    let mut in_heading = false;

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if !current_title.is_empty() {
                    sections.push(MarkdownSection {
                        level: current_level,
                        title: current_title.trim().to_string(),
                        body: current_body.trim().to_string(),
                    });
                }
                current_title.clear();
                current_body.clear();
                current_level = heading_to_u32(level);
                in_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
            }
            Event::Text(text) | Event::Code(text) => {
                if in_heading {
                    current_title.push_str(&text);
                } else {
                    current_body.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_heading {
                    current_title.push(' ');
                } else {
                    current_body.push('\n');
                }
            }
            _ => {}
        }
    }

    if !current_title.is_empty() {
        sections.push(MarkdownSection {
            level: current_level,
            title: current_title.trim().to_string(),
            body: current_body.trim().to_string(),
        });
    }

    sections
}

fn heading_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_frontmatter() {
        let doc = "---\na: b\n---\n# Hi\nBody";
        let parsed = extract_frontmatter(doc);
        assert_eq!(parsed.yaml.as_deref(), Some("a: b"));
        assert_eq!(parsed.body, "# Hi\nBody");
    }

    #[test]
    fn parses_sections() {
        let doc = "# One\nhello\n## Two\nworld";
        let sections = parse_sections(doc);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "One");
        assert_eq!(sections[1].level, 2);
    }
}

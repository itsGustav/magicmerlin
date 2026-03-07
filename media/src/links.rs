use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{MediaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkAnalysisResult {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub og_tags: HashMap<String, String>,
    pub markdown: String,
    pub text_length: usize,
    pub fetched_at_unix_ms: u128,
}

#[derive(Debug, Clone)]
pub struct LinkConfig {
    pub timeout: Duration,
    pub user_agent: String,
    pub cache_ttl: Duration,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(20),
            user_agent: "magicmerlin-media/0.1 (+https://example.invalid)".to_string(),
            cache_ttl: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: LinkAnalysisResult,
    expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct LinkAnalyzer {
    http: reqwest::Client,
    config: LinkConfig,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl LinkAnalyzer {
    pub fn new(config: LinkConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(config.user_agent.clone())
            .build()
            .map_err(MediaError::Http)?;

        Ok(Self {
            http,
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn fetch_and_extract(&self, url: &str) -> Result<LinkAnalysisResult> {
        if let Some(cached) = self.get_cached(url).await {
            return Ok(cached);
        }

        let html = self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let result = self.extract_from_html(url, &html)?;
        self.store_cached(url, result.clone()).await;
        Ok(result)
    }

    pub fn extract_from_html(&self, url: &str, html: &str) -> Result<LinkAnalysisResult> {
        if url.trim().is_empty() {
            return Err(MediaError::InvalidInput("url cannot be empty".to_string()));
        }

        let title = extract_title(html);
        let description = extract_meta_content(html, "description")
            .or_else(|| extract_meta_property_content(html, "og:description"));
        let og_tags = collect_og_tags(html);

        let main_html = extract_readable_html(html).unwrap_or_else(|| html.to_string());
        let markdown = html_fragment_to_markdown(&main_html);
        let text_length = markdown.trim().chars().count();

        Ok(LinkAnalysisResult {
            url: url.to_string(),
            title,
            description,
            og_tags,
            markdown,
            text_length,
            fetched_at_unix_ms: now_unix_ms(),
        })
    }

    pub async fn invalidate(&self, url: &str) {
        let mut cache = self.cache.lock().await;
        cache.remove(url);
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn cache_size(&self) -> usize {
        self.cache.lock().await.len()
    }

    async fn get_cached(&self, url: &str) -> Option<LinkAnalysisResult> {
        let mut cache = self.cache.lock().await;

        if let Some(entry) = cache.get(url) {
            if Instant::now() < entry.expires_at {
                return Some(entry.value.clone());
            }
        }

        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);

        None
    }

    async fn store_cached(&self, url: &str, result: LinkAnalysisResult) {
        let mut cache = self.cache.lock().await;
        cache.insert(
            url.to_string(),
            CacheEntry {
                value: result,
                expires_at: Instant::now() + self.config.cache_ttl,
            },
        );
    }
}

fn extract_title(html: &str) -> Option<String> {
    extract_tag_text(html, "title")
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
    for attrs in find_all_tag_attributes(html, "meta") {
        let attrs_lc = attrs.to_ascii_lowercase();
        if attrs_lc.contains(&format!("name=\"{}\"", name.to_ascii_lowercase()))
            || attrs_lc.contains(&format!("name='{}'", name.to_ascii_lowercase()))
        {
            if let Some(content) = extract_attr_value(&attrs, "content") {
                if !content.trim().is_empty() {
                    return Some(html_unescape(&content.trim().to_string()));
                }
            }
        }
    }
    None
}

fn extract_meta_property_content(html: &str, property: &str) -> Option<String> {
    for attrs in find_all_tag_attributes(html, "meta") {
        let attrs_lc = attrs.to_ascii_lowercase();
        if attrs_lc.contains(&format!("property=\"{}\"", property.to_ascii_lowercase()))
            || attrs_lc.contains(&format!("property='{}'", property.to_ascii_lowercase()))
        {
            if let Some(content) = extract_attr_value(&attrs, "content") {
                if !content.trim().is_empty() {
                    return Some(html_unescape(&content.trim().to_string()));
                }
            }
        }
    }
    None
}

fn collect_og_tags(html: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for attrs in find_all_tag_attributes(html, "meta") {
        let Some(property) = extract_attr_value(&attrs, "property") else {
            continue;
        };
        if !property.to_ascii_lowercase().starts_with("og:") {
            continue;
        }
        let Some(content) = extract_attr_value(&attrs, "content") else {
            continue;
        };
        if !content.trim().is_empty() {
            out.insert(
                property.trim().to_string(),
                html_unescape(&content.trim().to_string()),
            );
        }
    }
    out
}

fn extract_readable_html(html: &str) -> Option<String> {
    if let Some(article) = extract_tag_outer_html(html, "article") {
        return Some(strip_noise_blocks(&article));
    }
    if let Some(main) = extract_tag_outer_html(html, "main") {
        return Some(strip_noise_blocks(&main));
    }

    let mut best: Option<(usize, String)> = None;
    for tag in ["section", "div", "body"] {
        for block in extract_all_tag_outer_html(html, tag) {
            let text = normalize_whitespace(&strip_html_tags(&block));
            let score = text.split_whitespace().count();
            if score < 40 {
                continue;
            }
            match &best {
                Some((best_score, _)) if *best_score >= score => {}
                _ => best = Some((score, strip_noise_blocks(&block))),
            }
        }
    }

    best.map(|(_, html)| html)
}

fn strip_noise_blocks(html: &str) -> String {
    let mut cleaned = html.to_string();
    for tag in ["script", "style", "nav", "footer", "aside", "noscript"] {
        cleaned = remove_tag_content(&cleaned, tag);
    }
    cleaned
}

fn html_fragment_to_markdown(html: &str) -> String {
    let mut data = html.to_string();

    let replacements = [
        ("<h1", "\n# "),
        ("</h1>", "\n\n"),
        ("<h2", "\n## "),
        ("</h2>", "\n\n"),
        ("<h3", "\n### "),
        ("</h3>", "\n\n"),
        ("<p", "\n"),
        ("</p>", "\n\n"),
        ("<li", "\n- "),
        ("</li>", ""),
        ("<blockquote", "\n> "),
        ("</blockquote>", "\n"),
        ("<br", "\n"),
        ("<pre", "\n```\n"),
        ("</pre>", "\n```\n"),
        ("<code", "`"),
        ("</code>", "`"),
    ];

    for (needle, replacement) in replacements {
        data = replace_tag_open(&data, needle, replacement);
    }

    let stripped = strip_html_tags(&data);
    normalize_whitespace_markdown(&html_unescape(&stripped))
}

fn extract_tag_text(html: &str, tag: &str) -> Option<String> {
    let content = extract_tag_inner_html(html, tag)?;
    let text = normalize_whitespace(&strip_html_tags(&content));
    if text.is_empty() {
        None
    } else {
        Some(html_unescape(&text))
    }
}

fn extract_tag_inner_html(html: &str, tag: &str) -> Option<String> {
    let outer = extract_tag_outer_html(html, tag)?;
    let open_end = outer.find('>')?;
    let close_start = outer.rfind(&format!("</{tag}"))?;
    Some(outer[open_end + 1..close_start].to_string())
}

fn extract_tag_outer_html(html: &str, tag: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let open_pattern = format!("<{tag}");
    let close_pattern = format!("</{tag}>");

    let open_start = lower.find(&open_pattern)?;
    let after_open = lower[open_start..].find('>')? + open_start + 1;
    let close_start = lower[after_open..].find(&close_pattern)? + after_open;
    let close_end = close_start + close_pattern.len();
    Some(html[open_start..close_end].to_string())
}

fn extract_all_tag_outer_html(html: &str, tag: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let lower = html.to_ascii_lowercase();
    let open_pattern = format!("<{tag}");
    let close_pattern = format!("</{tag}>");

    while cursor < lower.len() {
        let Some(open_rel) = lower[cursor..].find(&open_pattern) else {
            break;
        };
        let open_start = cursor + open_rel;

        let Some(open_end_rel) = lower[open_start..].find('>') else {
            break;
        };
        let after_open = open_start + open_end_rel + 1;

        let Some(close_rel) = lower[after_open..].find(&close_pattern) else {
            break;
        };
        let close_start = after_open + close_rel;
        let close_end = close_start + close_pattern.len();
        out.push(html[open_start..close_end].to_string());

        cursor = close_end;
    }

    out
}

fn find_all_tag_attributes(html: &str, tag: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let pattern = format!("<{tag}");
    let mut out = Vec::new();
    let mut cursor = 0usize;

    while cursor < lower.len() {
        let Some(start_rel) = lower[cursor..].find(&pattern) else {
            break;
        };
        let start = cursor + start_rel;
        let Some(end_rel) = lower[start..].find('>') else {
            break;
        };
        let end = start + end_rel;
        out.push(html[start + pattern.len()..end].to_string());
        cursor = end + 1;
    }

    out
}

fn extract_attr_value(attrs: &str, key: &str) -> Option<String> {
    let attrs_lc = attrs.to_ascii_lowercase();

    for quote in ['"', '\''] {
        let needle = format!("{}={}", key.to_ascii_lowercase(), quote);
        if let Some(start) = attrs_lc.find(&needle) {
            let from = start + needle.len();
            let rest = &attrs[from..];
            if let Some(end_rel) = rest.find(quote) {
                return Some(rest[..end_rel].to_string());
            }
        }
    }

    None
}

fn remove_tag_content(html: &str, tag: &str) -> String {
    let mut data = html.to_string();
    loop {
        let lower = data.to_ascii_lowercase();
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let Some(open_start) = lower.find(&open) else {
            break;
        };
        let Some(open_end_rel) = lower[open_start..].find('>') else {
            break;
        };
        let after_open = open_start + open_end_rel + 1;
        let Some(close_rel) = lower[after_open..].find(&close) else {
            break;
        };
        let close_end = after_open + close_rel + close.len();
        data.replace_range(open_start..close_end, "");
    }
    data
}

fn replace_tag_open(data: &str, needle: &str, replacement: &str) -> String {
    let mut out = String::with_capacity(data.len());
    let mut cursor = 0usize;
    let lower = data.to_ascii_lowercase();
    let needle_lc = needle.to_ascii_lowercase();

    while cursor < data.len() {
        let Some(rel) = lower[cursor..].find(&needle_lc) else {
            out.push_str(&data[cursor..]);
            break;
        };
        let start = cursor + rel;
        out.push_str(&data[cursor..start]);

        if needle.starts_with("</") {
            out.push_str(replacement);
            cursor = start + needle.len();
            continue;
        }

        if let Some(end_rel) = lower[start..].find('>') {
            out.push_str(replacement);
            cursor = start + end_rel + 1;
        } else {
            out.push_str(&data[start..]);
            break;
        }
    }

    out
}

fn strip_html_tags(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn normalize_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_whitespace_markdown(input: &str) -> String {
    let mut out = String::new();
    let mut previous_blank = false;

    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() {
            if !previous_blank {
                out.push('\n');
            }
            previous_blank = true;
            continue;
        }

        if !out.is_empty() && !previous_blank {
            out.push('\n');
        }
        out.push_str(line);
        previous_blank = false;
    }

    out.trim().to_string()
}

fn html_unescape(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn now_unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_html() -> &'static str {
        r#"
<!doctype html>
<html>
  <head>
    <title>Readable Title</title>
    <meta name="description" content="Readable summary">
    <meta property="og:title" content="OG Readable Title">
    <meta property="og:image" content="https://example.com/img.png">
  </head>
  <body>
    <nav>Links</nav>
    <main>
      <h1>Article Heading</h1>
      <p>This is the first paragraph with enough words to be considered meaningful content.</p>
      <p>This is the second paragraph that should survive readability extraction and become markdown.</p>
    </main>
  </body>
</html>
"#
    }

    #[test]
    fn metadata_extraction_picks_title_description_and_og() {
        let analyzer = LinkAnalyzer::new(LinkConfig::default()).expect("analyzer build");
        let result = analyzer
            .extract_from_html("https://example.com", sample_html())
            .expect("extract should work");

        assert_eq!(result.title.as_deref(), Some("Readable Title"));
        assert_eq!(result.description.as_deref(), Some("Readable summary"));
        assert_eq!(
            result.og_tags.get("og:title").map(|v| v.as_str()),
            Some("OG Readable Title")
        );
        assert!(result.markdown.contains("Article Heading"));
    }

    #[test]
    fn readability_prefers_article_and_main() {
        let html = r#"
<html><body>
<article><h1>Article</h1><p>One two three four five six seven eight nine ten.</p></article>
<div><p>tiny</p></div>
</body></html>
"#;
        let picked = extract_readable_html(html).expect("should pick readable chunk");
        assert!(picked.contains("<article>"));
    }

    #[tokio::test]
    async fn cache_stores_and_expires_entries() {
        let mut config = LinkConfig::default();
        config.cache_ttl = Duration::from_millis(100);
        let analyzer = LinkAnalyzer::new(config).expect("analyzer build");

        let result = analyzer
            .extract_from_html("https://example.com", sample_html())
            .expect("extract should work");
        analyzer.store_cached("https://example.com", result).await;

        assert!(analyzer.get_cached("https://example.com").await.is_some());
        tokio::time::sleep(Duration::from_millis(120)).await;
        assert!(analyzer.get_cached("https://example.com").await.is_none());
    }

    #[tokio::test]
    async fn clear_cache_works() {
        let analyzer = LinkAnalyzer::new(LinkConfig::default()).expect("analyzer build");
        let result = analyzer
            .extract_from_html("https://example.com", sample_html())
            .expect("extract should work");
        analyzer.store_cached("https://example.com", result).await;
        assert_eq!(analyzer.cache_size().await, 1);
        analyzer.clear_cache().await;
        assert_eq!(analyzer.cache_size().await, 0);
    }

    #[test]
    fn invalid_url_input_rejected() {
        let analyzer = LinkAnalyzer::new(LinkConfig::default()).expect("analyzer build");
        let result = analyzer.extract_from_html("", sample_html());
        assert!(result.is_err());
    }

    #[test]
    fn attr_parsing_supports_single_and_double_quotes() {
        let attrs = "property='og:title' content=\"Hello\"";
        assert_eq!(
            extract_attr_value(attrs, "property").as_deref(),
            Some("og:title")
        );
        assert_eq!(
            extract_attr_value(attrs, "content").as_deref(),
            Some("Hello")
        );
    }

    #[test]
    fn markdown_converter_handles_blocks() {
        let markdown = html_fragment_to_markdown("<h1>T</h1><p>A B</p><ul><li>X</li></ul>");
        assert!(markdown.contains("# T"));
        assert!(markdown.contains("A B"));
        assert!(markdown.contains("- X"));
    }
}

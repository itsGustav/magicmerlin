use crate::framework::ParseMode;

use super::types::{
    TelegramEntityKind, TelegramFormattedText, TelegramMessageEntity, TELEGRAM_MAX_MESSAGE_LEN,
};

#[derive(Debug, Clone)]
struct SplitPart {
    start: usize,
    end: usize,
    prefix: String,
    body: String,
}

/// Escapes Telegram MarkdownV2 control characters.
pub fn escape_markdown_v2(text: &str) -> String {
    const SPECIAL: &[char] = &[
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];

    let mut escaped = String::with_capacity(text.len() * 2);
    for ch in text.chars() {
        if SPECIAL.contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Parses MarkdownV2 into Telegram text entities.
pub fn parse_markdown_v2(input: &str) -> TelegramFormattedText {
    let mut text = String::new();
    let mut entities = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() {
            text.push(chars[index + 1]);
            index += 2;
            continue;
        }

        if let Some((entity, consumed, content)) =
            parse_markdown_entity(&chars, index, text.chars().count())
        {
            text.push_str(&content);
            entities.push(entity);
            index += consumed;
            continue;
        }

        text.push(chars[index]);
        index += 1;
    }

    TelegramFormattedText {
        text,
        entities,
        parse_mode: ParseMode::Markdown,
    }
}

/// Parses HTML tags supported by Telegram into entities.
pub fn parse_html(input: &str) -> TelegramFormattedText {
    #[derive(Debug, Clone)]
    struct OpenTag {
        kind: TelegramEntityKind,
        start: usize,
        url: Option<String>,
    }

    let mut text = String::new();
    let mut entities = Vec::new();
    let mut stack: Vec<OpenTag> = Vec::new();
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == b'<' {
            if let Some(close) = input[index..].find('>') {
                let raw = &input[index + 1..index + close];
                let normalized = raw.trim();
                if let Some(tag_name) = normalized.strip_prefix('/') {
                    if let Some(position) =
                        stack.iter().rposition(|open| tag_matches(open, tag_name))
                    {
                        let open = stack.remove(position);
                        let end = text.chars().count();
                        if end > open.start {
                            entities.push(TelegramMessageEntity {
                                kind: open.kind,
                                offset: open.start,
                                length: end - open.start,
                                url: open.url,
                            });
                        }
                    }
                } else if let Some((kind, url)) = parse_html_open_tag(normalized) {
                    stack.push(OpenTag {
                        kind,
                        start: text.chars().count(),
                        url,
                    });
                }
                index += close + 1;
                continue;
            }
        }

        let ch = input[index..].chars().next().unwrap_or_default();
        text.push(ch);
        index += ch.len_utf8();
    }

    TelegramFormattedText {
        text,
        entities,
        parse_mode: ParseMode::Html,
    }
}

/// Formats outbound text into Telegram text entities when parse mode is set.
pub fn format_text(text: &str, parse_mode: Option<ParseMode>) -> TelegramFormattedText {
    match parse_mode.unwrap_or(ParseMode::Plain) {
        ParseMode::Markdown => parse_markdown_v2(text),
        ParseMode::Html => parse_html(text),
        ParseMode::Plain => TelegramFormattedText {
            text: text.to_string(),
            entities: Vec::new(),
            parse_mode: ParseMode::Plain,
        },
    }
}

/// Splits text into Telegram-sized chunks with continuation markers and entity preservation.
pub fn split_formatted_text(
    formatted: &TelegramFormattedText,
    limit: usize,
) -> Vec<TelegramFormattedText> {
    let limit = limit.max(1);
    let mut parts = split_parts(&formatted.text, limit);
    if parts.is_empty() {
        parts.push(SplitPart {
            start: 0,
            end: 0,
            prefix: String::new(),
            body: String::new(),
        });
    }

    parts
        .into_iter()
        .map(|part| {
            let prefix_len = part.prefix.chars().count();
            let mut entities = Vec::new();
            for entity in &formatted.entities {
                let entity_start = entity.offset;
                let entity_end = entity.offset + entity.length;
                if entity_end <= part.start || entity_start >= part.end {
                    continue;
                }

                let overlap_start = entity_start.max(part.start);
                let overlap_end = entity_end.min(part.end);
                entities.push(TelegramMessageEntity {
                    kind: entity.kind.clone(),
                    offset: overlap_start - part.start + prefix_len,
                    length: overlap_end - overlap_start,
                    url: entity.url.clone(),
                });
            }

            TelegramFormattedText {
                text: format!("{}{}", part.prefix, part.body),
                entities,
                parse_mode: formatted.parse_mode,
            }
        })
        .collect()
}

/// Splits raw text with Telegram continuation markers.
pub fn split_message(text: &str) -> Vec<String> {
    let formatted = TelegramFormattedText {
        text: text.to_string(),
        entities: Vec::new(),
        parse_mode: ParseMode::Plain,
    };
    split_formatted_text(&formatted, TELEGRAM_MAX_MESSAGE_LEN)
        .into_iter()
        .map(|part| part.text)
        .collect()
}

fn parse_markdown_entity(
    chars: &[char],
    start: usize,
    text_offset: usize,
) -> Option<(TelegramMessageEntity, usize, String)> {
    if chars.get(start..start + 2) == Some(&['_', '_']) {
        return parse_wrapped_entity(
            chars,
            start,
            "__",
            TelegramEntityKind::Underline,
            text_offset,
        );
    }
    if chars[start] == '*' {
        return parse_wrapped_entity(chars, start, "*", TelegramEntityKind::Bold, text_offset);
    }
    if chars[start] == '_' {
        return parse_wrapped_entity(chars, start, "_", TelegramEntityKind::Italic, text_offset);
    }
    if chars[start] == '~' {
        return parse_wrapped_entity(
            chars,
            start,
            "~",
            TelegramEntityKind::Strikethrough,
            text_offset,
        );
    }
    if chars[start] == '`' {
        return parse_wrapped_entity(chars, start, "`", TelegramEntityKind::Code, text_offset);
    }
    if chars[start] == '[' {
        return parse_link(chars, start, text_offset);
    }
    None
}

fn parse_wrapped_entity(
    chars: &[char],
    start: usize,
    marker: &str,
    kind: TelegramEntityKind,
    text_offset: usize,
) -> Option<(TelegramMessageEntity, usize, String)> {
    let marker_chars: Vec<char> = marker.chars().collect();
    if chars.get(start..start + marker_chars.len()) != Some(marker_chars.as_slice()) {
        return None;
    }

    let search_start = start + marker_chars.len();
    let end = find_marker(chars, search_start, &marker_chars)?;
    let content: String = chars[search_start..end].iter().collect();
    let content_len = content.chars().count();
    Some((
        TelegramMessageEntity {
            kind,
            offset: text_offset,
            length: content_len,
            url: None,
        },
        marker_chars.len() * 2 + content_len,
        content,
    ))
}

fn parse_link(
    chars: &[char],
    start: usize,
    text_offset: usize,
) -> Option<(TelegramMessageEntity, usize, String)> {
    let text_end = chars[start + 1..].iter().position(|ch| *ch == ']')? + start + 1;
    if chars.get(text_end + 1) != Some(&'(') {
        return None;
    }
    let url_end = chars[text_end + 2..].iter().position(|ch| *ch == ')')? + text_end + 2;
    let content: String = chars[start + 1..text_end].iter().collect();
    let url: String = chars[text_end + 2..url_end].iter().collect();
    Some((
        TelegramMessageEntity {
            kind: TelegramEntityKind::Link,
            offset: text_offset,
            length: content.chars().count(),
            url: Some(url),
        },
        url_end - start + 1,
        content,
    ))
}

fn find_marker(chars: &[char], start: usize, marker: &[char]) -> Option<usize> {
    let mut index = start;
    while index + marker.len() <= chars.len() {
        if chars.get(index..index + marker.len()) == Some(marker) {
            return Some(index);
        }
        if chars[index] == '\\' {
            index += 2;
        } else {
            index += 1;
        }
    }
    None
}

fn parse_html_open_tag(tag: &str) -> Option<(TelegramEntityKind, Option<String>)> {
    let tag = tag.trim();
    if tag.eq_ignore_ascii_case("b") || tag.eq_ignore_ascii_case("strong") {
        return Some((TelegramEntityKind::Bold, None));
    }
    if tag.eq_ignore_ascii_case("i") || tag.eq_ignore_ascii_case("em") {
        return Some((TelegramEntityKind::Italic, None));
    }
    if tag.eq_ignore_ascii_case("u") {
        return Some((TelegramEntityKind::Underline, None));
    }
    if tag.eq_ignore_ascii_case("s")
        || tag.eq_ignore_ascii_case("strike")
        || tag.eq_ignore_ascii_case("del")
    {
        return Some((TelegramEntityKind::Strikethrough, None));
    }
    if tag.eq_ignore_ascii_case("code") {
        return Some((TelegramEntityKind::Code, None));
    }
    if tag.eq_ignore_ascii_case("pre") {
        return Some((TelegramEntityKind::Pre, None));
    }
    if tag.starts_with('a') {
        let href = tag
            .split_whitespace()
            .find_map(|part| part.strip_prefix("href=\""))
            .map(|value| value.trim_end_matches('"').to_string());
        return Some((TelegramEntityKind::Link, href));
    }
    None
}

fn tag_matches(open: &impl std::fmt::Debug, close_tag: &str) -> bool {
    let close_tag = close_tag.trim().to_ascii_lowercase();
    match format!("{open:?}").as_str() {
        value if value.contains("Bold") => matches!(close_tag.as_str(), "b" | "strong"),
        value if value.contains("Italic") => matches!(close_tag.as_str(), "i" | "em"),
        value if value.contains("Underline") => close_tag == "u",
        value if value.contains("Strikethrough") => {
            matches!(close_tag.as_str(), "s" | "strike" | "del")
        }
        value if value.contains("Code") => close_tag == "code",
        value if value.contains("Pre") => close_tag == "pre",
        value if value.contains("Link") => close_tag == "a",
        _ => false,
    }
}

fn split_parts(text: &str, limit: usize) -> Vec<SplitPart> {
    if text.is_empty() {
        return vec![SplitPart {
            start: 0,
            end: 0,
            prefix: String::new(),
            body: String::new(),
        }];
    }

    if text.chars().count() <= limit {
        return vec![SplitPart {
            start: 0,
            end: text.chars().count(),
            prefix: String::new(),
            body: text.to_string(),
        }];
    }

    let mut base = split_without_prefix(text, limit);
    loop {
        let total = base.len();
        let marker_len = continuation_prefix(total, total).chars().count();
        let effective_limit = limit.saturating_sub(marker_len).max(1);
        let next = split_without_prefix(text, effective_limit);
        if next.len() == total {
            return next
                .into_iter()
                .enumerate()
                .map(|(index, mut part)| {
                    part.prefix = continuation_prefix(index + 1, total);
                    part
                })
                .collect();
        }
        base = next;
    }
}

fn split_without_prefix(text: &str, limit: usize) -> Vec<SplitPart> {
    let chars: Vec<char> = text.chars().collect();
    let mut parts = Vec::new();
    let mut start = 0usize;

    while start < chars.len() {
        let mut end = (start + limit).min(chars.len());
        if end < chars.len() {
            let mut candidate = end;
            while candidate > start && chars[candidate - 1] != '\n' && chars[candidate - 1] != ' ' {
                candidate -= 1;
            }
            if candidate > start {
                end = candidate;
            }
        }

        let body: String = chars[start..end].iter().collect();
        parts.push(SplitPart {
            start,
            end,
            prefix: String::new(),
            body,
        });
        start = end;
        while start < chars.len() && chars[start] == ' ' {
            start += 1;
        }
    }

    parts
}

fn continuation_prefix(index: usize, total: usize) -> String {
    if total <= 1 {
        String::new()
    } else {
        format!("[{index}/{total}] ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_parser_extracts_entities() {
        let formatted = parse_markdown_v2("*bold* _italic_ [link](https://example.com)");
        assert_eq!(formatted.text, "bold italic link");
        assert_eq!(formatted.entities.len(), 3);
        assert_eq!(formatted.entities[0].kind, TelegramEntityKind::Bold);
        assert_eq!(
            formatted.entities[2].url.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn html_parser_extracts_entities() {
        let formatted = parse_html("<b>bold</b><i>ital</i><a href=\"https://x\">link</a>");
        assert_eq!(formatted.text, "bolditallink");
        assert_eq!(formatted.entities.len(), 3);
        assert_eq!(formatted.entities[1].kind, TelegramEntityKind::Italic);
        assert_eq!(formatted.entities[2].url.as_deref(), Some("https://x"));
    }

    #[test]
    fn split_formatted_text_preserves_entities() {
        let source = TelegramFormattedText {
            text: "x".repeat(TELEGRAM_MAX_MESSAGE_LEN + 50),
            entities: vec![TelegramMessageEntity {
                kind: TelegramEntityKind::Bold,
                offset: TELEGRAM_MAX_MESSAGE_LEN - 10,
                length: 30,
                url: None,
            }],
            parse_mode: ParseMode::Markdown,
        };
        let chunks = split_formatted_text(&source, TELEGRAM_MAX_MESSAGE_LEN);
        assert_eq!(chunks.len(), 2);
        assert!(chunks
            .iter()
            .all(|chunk| chunk.text.chars().count() <= TELEGRAM_MAX_MESSAGE_LEN));
        assert_eq!(chunks[0].entities.len(), 1);
        assert_eq!(chunks[1].entities.len(), 1);
    }
}

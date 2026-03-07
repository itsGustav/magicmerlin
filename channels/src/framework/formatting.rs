use super::{OutboundMessage, ParseMode, Platform};

/// Returns platform-specific message length limits.
pub fn platform_message_limit(platform: Platform) -> usize {
    match platform {
        Platform::Telegram => 4096,
        Platform::Discord => 2000,
        Platform::WhatsApp => 65536,
        Platform::Signal => 4096,
        Platform::Slack => 4000,
        Platform::IMessage => 4096,
        Platform::Line => 5000,
        Platform::Web => 8192,
    }
}

/// Converts outbound text into a platform-targeted representation.
pub fn format_for_platform(platform: Platform, message: &OutboundMessage) -> String {
    match (platform, message.parse_mode.unwrap_or(ParseMode::Plain)) {
        (Platform::Telegram, ParseMode::Markdown) => escape_markdown_v2(&message.text),
        (_, ParseMode::Html) => strip_html_tags(&message.text),
        _ => message.text.clone(),
    }
}

/// Splits outbound text into chunks according to platform limits.
pub fn split_for_platform(platform: Platform, message: &OutboundMessage) -> Vec<String> {
    split_text_by_limit(
        &format_for_platform(platform, message),
        platform_message_limit(platform),
    )
}

/// Splits text without exceeding provided byte-length limit.
pub fn split_text_by_limit(text: &str, limit: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if text.len() <= limit {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if word.len() > limit {
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }

            let mut partial = String::new();
            for ch in word.chars() {
                if partial.len() + ch.len_utf8() > limit {
                    chunks.push(partial.clone());
                    partial.clear();
                }
                partial.push(ch);
            }
            if !partial.is_empty() {
                chunks.push(partial);
            }
            continue;
        }

        let candidate_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if candidate_len > limit {
            chunks.push(current.clone());
            current.clear();
            current.push_str(word);
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn escape_markdown_v2(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|'
            | '{' | '}' | '.' | '!' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

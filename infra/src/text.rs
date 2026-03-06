//! Generic text and payload utility helpers.

use base64::Engine;

use crate::InfraError;

/// Truncates text to `max_chars`, preserving word boundaries when possible.
pub fn truncate_with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    if max_chars <= 1 {
        return "…".to_string();
    }

    let cutoff = max_chars - 1;
    let mut result = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= cutoff {
            break;
        }
        result.push(ch);
    }

    if let Some(last_space) = result.rfind(char::is_whitespace) {
        let candidate = result[..last_space].trim_end();
        if !candidate.is_empty() {
            return format!("{candidate}…");
        }
    }

    format!("{}…", result.trim_end())
}

/// Pretty-prints a JSON value.
pub fn json_pretty(value: &serde_json::Value) -> Result<String, InfraError> {
    serde_json::to_string_pretty(value).map_err(InfraError::Json)
}

/// Base64-encodes UTF-8 text.
pub fn base64_encode(input: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(input)
}

/// Decodes a base64 payload into UTF-8 text.
pub fn base64_decode(input: &str) -> Result<String, InfraError> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(input)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Removes unsafe control characters while preserving common whitespace.
pub fn sanitize_unicode(input: &str) -> String {
    input
        .chars()
        .filter(|ch| {
            if ch.is_control() {
                matches!(ch, '\n' | '\r' | '\t')
            } else {
                true
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_with_word_boundary() {
        // max_chars=12, cutoff=11 chars then ellipsis. "hello world" is 11 chars,
        // last space at 5 → truncates to "hello…" (word boundary).
        assert_eq!(
            truncate_with_ellipsis("hello world from merlin", 12),
            "hello…"
        );
        // With enough room, keeps more words
        assert_eq!(
            truncate_with_ellipsis("hello world from merlin", 18),
            "hello world from…"
        );
        // No truncation needed
        assert_eq!(truncate_with_ellipsis("short", 10), "short");
    }

    #[test]
    fn base64_roundtrip() {
        let encoded = base64_encode("magic");
        assert_eq!(base64_decode(&encoded).expect("decode"), "magic");
    }
}

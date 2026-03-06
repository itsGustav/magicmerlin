//! Reusable infrastructure utilities: HTTP, time, text, and markdown helpers.

mod error;
pub mod http;
pub mod markdown;
pub mod text;
pub mod time;

pub use error::InfraError;

/// Strips markdown formatting to plain text.
pub fn strip_markdown(input: &str) -> String {
    markdown::strip_markdown_to_text(input)
}

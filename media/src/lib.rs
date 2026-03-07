//! Media processing, browser automation, canvas hosting, link understanding, and TTS.

#[cfg(feature = "browser")]
pub mod browser;
#[cfg(feature = "canvas")]
pub mod canvas;
pub mod links;
#[cfg(feature = "tts")]
pub mod tts;
#[cfg(feature = "media-understanding")]
pub mod understanding;

use serde::{Deserialize, Serialize};

/// High-level media input type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Pdf,
}

/// Unified media analysis result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisResult {
    pub media_type: MediaType,
    pub provider: String,
    pub text: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Common crate-level error for media operations.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("execution failed: {0}")]
    Execution(String),
    #[error("feature disabled: {0}")]
    FeatureDisabled(&'static str),
}

pub type Result<T> = std::result::Result<T, MediaError>;

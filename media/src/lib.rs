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

#[cfg(feature = "media-understanding")]
pub use understanding::{AnalysisRequest, AnalysisResult, MediaSource, MediaType, VisionProvider};

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

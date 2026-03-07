//! Provider trait and concrete provider modules.

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{CompletionRequest, CompletionResponse, StreamChunk};

pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod groq;
pub mod local;
pub mod minimax;
pub mod mistral;
pub mod moonshot;
pub mod openai;
pub mod openai_compat;
pub mod xai;

/// Provider stream represented as ordered chunks.
pub type ProviderStream = Vec<Result<StreamChunk>>;

/// Common provider interface for chat completion + streaming.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Performs non-streaming completion.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Performs streaming completion.
    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream>;

    /// Returns provider identifier.
    fn name(&self) -> &str;

    /// Returns true if provider supports the model id.
    fn supports_model(&self, model_id: &str) -> bool;
}

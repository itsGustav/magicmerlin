//! local provider wrapper on OpenAI-compatible protocol.

use async_trait::async_trait;

use crate::auth::AuthProfiles;
use crate::error::Result;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{CompletionRequest, CompletionResponse};

/// local provider wrapper.
#[derive(Clone, Debug)]
pub struct LocalProvider {
    inner: OpenAiCompatProvider,
}

impl LocalProvider {
    /// Creates a local provider with default Ollama-compatible endpoint.
    pub fn new(auth: AuthProfiles) -> Self {
        Self::new_with_base_url("http://localhost:11434/v1", auth)
    }

    /// Creates a local provider with custom OpenAI-compatible endpoint.
    pub fn new_with_base_url(base_url: impl Into<String>, auth: AuthProfiles) -> Self {
        Self {
            inner: OpenAiCompatProvider::new("local", base_url, "local", auth),
        }
    }
}

#[async_trait]
impl LlmProvider for LocalProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream> {
        self.inner.complete_stream(request).await
    }

    fn name(&self) -> &str {
        "local"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

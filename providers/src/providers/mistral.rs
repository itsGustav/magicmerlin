//! mistral provider wrapper on OpenAI-compatible protocol.

use async_trait::async_trait;

use crate::auth::AuthProfiles;
use crate::error::Result;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{CompletionRequest, CompletionResponse};

/// mistral provider wrapper.
#[derive(Clone, Debug)]
pub struct MistralProvider {
    inner: OpenAiCompatProvider,
}

impl MistralProvider {
    /// Creates a mistral provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            inner: OpenAiCompatProvider::new("mistral", "https://api.mistral.ai", "mistral", auth),
        }
    }
}

#[async_trait]
impl LlmProvider for MistralProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream> {
        self.inner.complete_stream(request).await
    }

    fn name(&self) -> &str {
        "mistral"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

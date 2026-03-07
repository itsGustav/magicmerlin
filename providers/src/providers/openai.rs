//! OpenAI provider implementation.

use async_trait::async_trait;

use crate::auth::AuthProfiles;
use crate::error::Result;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{CompletionRequest, CompletionResponse};

/// OpenAI provider (Chat Completions API).
#[derive(Clone, Debug)]
pub struct OpenAiProvider {
    inner: OpenAiCompatProvider,
}

impl OpenAiProvider {
    /// Creates a new OpenAI provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            inner: OpenAiCompatProvider::new("openai", "https://api.openai.com", "openai", auth),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream> {
        self.inner.complete_stream(request).await
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn supports_model(&self, model_id: &str) -> bool {
        model_id.starts_with("gpt") || model_id.starts_with('o')
    }
}

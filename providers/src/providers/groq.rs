//! groq provider wrapper on OpenAI-compatible protocol.

use async_trait::async_trait;

use crate::auth::AuthProfiles;
use crate::error::Result;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{CompletionRequest, CompletionResponse};

/// groq provider wrapper.
#[derive(Clone, Debug)]
pub struct GroqProvider {
    inner: OpenAiCompatProvider,
}

impl GroqProvider {
    /// Creates a groq provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            inner: OpenAiCompatProvider::new("groq", "https://api.groq.com/openai", "groq", auth),
        }
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream> {
        self.inner.complete_stream(request).await
    }

    fn name(&self) -> &str {
        "groq"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

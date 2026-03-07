//! deepseek provider wrapper on OpenAI-compatible protocol.

use async_trait::async_trait;

use crate::auth::AuthProfiles;
use crate::error::Result;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{CompletionRequest, CompletionResponse};

/// deepseek provider wrapper.
#[derive(Clone, Debug)]
pub struct DeepseekProvider {
    inner: OpenAiCompatProvider,
}

impl DeepseekProvider {
    /// Creates a deepseek provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            inner: OpenAiCompatProvider::new(
                "deepseek",
                "https://api.deepseek.com",
                "deepseek",
                auth,
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for DeepseekProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream> {
        self.inner.complete_stream(request).await
    }

    fn name(&self) -> &str {
        "deepseek"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

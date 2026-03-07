//! moonshot provider wrapper on OpenAI-compatible protocol.

use async_trait::async_trait;

use crate::auth::AuthProfiles;
use crate::error::Result;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{CompletionRequest, CompletionResponse};

/// moonshot provider wrapper.
#[derive(Clone, Debug)]
pub struct MoonshotProvider {
    inner: OpenAiCompatProvider,
}

impl MoonshotProvider {
    /// Creates a moonshot provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            inner: OpenAiCompatProvider::new(
                "moonshot",
                "https://api.moonshot.cn",
                "moonshot",
                auth,
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for MoonshotProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ProviderStream> {
        self.inner.complete_stream(request).await
    }

    fn name(&self) -> &str {
        "moonshot"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

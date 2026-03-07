//! Provider router with model resolution, retries, and failover.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::model_registry::ModelRegistry;
use crate::providers::anthropic::AnthropicProvider;
use crate::providers::deepseek::DeepseekProvider;
use crate::providers::google::GoogleProvider;
use crate::providers::groq::GroqProvider;
use crate::providers::local::LocalProvider;
use crate::providers::minimax::MinimaxProvider;
use crate::providers::mistral::MistralProvider;
use crate::providers::moonshot::MoonshotProvider;
use crate::providers::openai::OpenAiProvider;
use crate::providers::xai::XaiProvider;
use crate::providers::LlmProvider;
use crate::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, Usage,
};

/// Retry configuration for one provider before failover.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retries per provider.
    pub max_retries: u32,
    /// Base delay for backoff.
    pub base_delay: Duration,
    /// Max delay cap for backoff.
    pub max_delay: Duration,
    /// Request timeout.
    pub request_timeout: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(5),
            request_timeout: Duration::from_secs(120),
        }
    }
}

/// Routes completion requests to providers with retry and failover semantics.
#[derive(Clone)]
pub struct ProviderRouter {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    /// Registry for model resolution and metadata.
    pub model_registry: ModelRegistry,
    /// Retry settings.
    pub retry: RetryConfig,
    rate_limit_until: Arc<Mutex<HashMap<String, Instant>>>,
}

impl ProviderRouter {
    /// Builds a router with default provider registrations.
    pub fn with_defaults(
        model_registry: ModelRegistry,
        auth: AuthProfiles,
        local_base_url: Option<String>,
    ) -> Self {
        let mut providers: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();
        providers.insert(
            "openai".to_string(),
            Arc::new(OpenAiProvider::new(auth.clone())),
        );
        providers.insert(
            "anthropic".to_string(),
            Arc::new(AnthropicProvider::new(auth.clone())),
        );
        providers.insert(
            "google".to_string(),
            Arc::new(GoogleProvider::new(auth.clone())),
        );
        providers.insert("xai".to_string(), Arc::new(XaiProvider::new(auth.clone())));
        providers.insert(
            "groq".to_string(),
            Arc::new(GroqProvider::new(auth.clone())),
        );
        providers.insert(
            "mistral".to_string(),
            Arc::new(MistralProvider::new(auth.clone())),
        );
        providers.insert(
            "minimax".to_string(),
            Arc::new(MinimaxProvider::new(auth.clone())),
        );
        providers.insert(
            "moonshot".to_string(),
            Arc::new(MoonshotProvider::new(auth.clone())),
        );
        providers.insert(
            "deepseek".to_string(),
            Arc::new(DeepseekProvider::new(auth.clone())),
        );

        let local = if let Some(base) = local_base_url {
            LocalProvider::new_with_base_url(base, auth)
        } else {
            LocalProvider::new(auth)
        };
        providers.insert("local".to_string(), Arc::new(local));

        Self {
            providers,
            model_registry,
            retry: RetryConfig::default(),
            rate_limit_until: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a router with explicit providers.
    pub fn new(model_registry: ModelRegistry) -> Self {
        Self {
            providers: HashMap::new(),
            model_registry,
            retry: RetryConfig::default(),
            rate_limit_until: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers one provider implementation.
    pub fn register_provider(&mut self, provider: Arc<dyn LlmProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    /// Completes with primary model + failover chain.
    pub async fn complete_with_failover(
        &self,
        request: CompletionRequest,
        fallbacks: &[String],
    ) -> Result<CompletionResponse> {
        let primary = self.model_registry.resolve_model(&request.model)?;
        let mut chain = vec![primary];
        for fallback in fallbacks {
            chain.push(self.model_registry.resolve_model(fallback)?);
        }

        let mut last_error: Option<ProviderError> = None;
        for canonical_model in chain {
            let (provider_name, provider_model_id) =
                ModelRegistry::parse_provider_model(&canonical_model)?;
            let provider = self
                .providers
                .get(&provider_name)
                .cloned()
                .ok_or_else(|| ProviderError::ProviderNotFound(provider_name.clone()))?;

            let mut provider_request = request.clone();
            provider_request.model = provider_model_id.clone();

            match self
                .try_provider(&provider_name, provider, provider_request)
                .await
            {
                Ok(mut response) => {
                    if response.usage.input_tokens == 0 && response.usage.output_tokens == 0 {
                        let input = request
                            .messages
                            .iter()
                            .map(|m| serde_json::to_string(m).unwrap_or_else(|_| String::new()))
                            .collect::<Vec<_>>()
                            .join("\n");
                        let output = response
                            .content
                            .iter()
                            .map(content_to_text)
                            .collect::<Vec<_>>()
                            .join("\n");
                        response.usage = Usage {
                            input_tokens: approximate_tokens(&input),
                            output_tokens: approximate_tokens(&output),
                            cache_read: 0,
                            cache_write: 0,
                        };
                    }

                    response.model = canonical_model.clone();
                    response.estimated_cost_usd = self
                        .model_registry
                        .estimate_cost_usd(&canonical_model, &response.usage);
                    return Ok(response);
                }
                Err(err) if err.is_retryable() => {
                    last_error = Some(err);
                }
                Err(err) => return Err(err),
            }
        }

        Err(ProviderError::Exhausted(format!(
            "all models failed: {}",
            last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown failure".to_string())
        )))
    }

    async fn try_provider(
        &self,
        provider_name: &str,
        provider: Arc<dyn LlmProvider>,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        for attempt in 0..=self.retry.max_retries {
            self.wait_rate_limit(provider_name).await;
            let timed = tokio::time::timeout(
                self.retry.request_timeout,
                provider.complete(request.clone()),
            )
            .await;

            match timed {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(err)) if err.is_retryable() => {
                    if let Some(wait) = err.retry_after_hint() {
                        self.set_rate_limit(provider_name, wait).await;
                    }
                    if attempt == self.retry.max_retries {
                        return Err(err);
                    }
                    tokio::time::sleep(backoff_delay(&self.retry, attempt)).await;
                }
                Ok(Err(err)) => return Err(err),
                Err(_) => {
                    if attempt == self.retry.max_retries {
                        return Err(ProviderError::Timeout(self.retry.request_timeout));
                    }
                    tokio::time::sleep(backoff_delay(&self.retry, attempt)).await;
                }
            }
        }

        Err(ProviderError::Exhausted(
            "retry loop exhausted unexpectedly".to_string(),
        ))
    }

    async fn set_rate_limit(&self, provider_name: &str, wait: Duration) {
        let mut lock = self.rate_limit_until.lock().await;
        lock.insert(provider_name.to_string(), Instant::now() + wait);
    }

    async fn wait_rate_limit(&self, provider_name: &str) {
        let wait_until = {
            let lock = self.rate_limit_until.lock().await;
            lock.get(provider_name).copied()
        };
        if let Some(until) = wait_until {
            let now = Instant::now();
            if until > now {
                tokio::time::sleep(until.duration_since(now)).await;
            }
        }
    }
}

fn backoff_delay(config: &RetryConfig, attempt: u32) -> Duration {
    let factor = 2_u64.saturating_pow(attempt.min(10));
    let delay = config.base_delay.as_millis() as u64 * factor;
    Duration::from_millis(delay.min(config.max_delay.as_millis() as u64))
}

fn content_to_text(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Text { text } => text.clone(),
        ContentBlock::Json { value } => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::model_registry::{ModelCapabilities, ModelDefinition};
    use crate::providers::ProviderStream;
    use crate::types::{CompletionResponse, Message, MessageContent, Role, StopReason, Usage};

    #[derive(Clone)]
    struct MockProvider {
        name: String,
        attempts: Arc<Mutex<u32>>,
        fail_first: bool,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
            let mut lock = self
                .attempts
                .lock()
                .map_err(|_| ProviderError::Exhausted("mock mutex poisoned".to_string()))?;
            *lock += 1;
            if self.fail_first && *lock == 1 {
                return Err(ProviderError::Api {
                    status: 500,
                    body: "boom".to_string(),
                });
            }
            Ok(CompletionResponse {
                id: "resp_1".to_string(),
                model: request.model,
                content: vec![ContentBlock::Text {
                    text: "ok".to_string(),
                }],
                tool_calls: Vec::new(),
                usage: Usage {
                    input_tokens: 10,
                    output_tokens: 20,
                    cache_read: 0,
                    cache_write: 0,
                },
                stop_reason: StopReason::EndTurn,
                estimated_cost_usd: None,
            })
        }

        async fn complete_stream(&self, _request: CompletionRequest) -> Result<ProviderStream> {
            Err(ProviderError::Exhausted("not used".to_string()))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn supports_model(&self, _model_id: &str) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn retries_and_succeeds_on_same_provider() {
        let mut registry = ModelRegistry::default();
        registry.upsert_model(ModelDefinition {
            provider: "openai".to_string(),
            model_id: "gpt-5.2".to_string(),
            context_window: 1,
            max_tokens: 1,
            input_cost_per_mtok: 1.0,
            output_cost_per_mtok: 1.0,
            capabilities: ModelCapabilities::default(),
        });

        let attempts = Arc::new(Mutex::new(0));
        let mut router = ProviderRouter::new(registry);
        router.retry.max_retries = 1;
        router.register_provider(Arc::new(MockProvider {
            name: "openai".to_string(),
            attempts: attempts.clone(),
            fail_first: true,
        }));

        let response = router
            .complete_with_failover(
                CompletionRequest {
                    model: "openai/gpt-5.2".to_string(),
                    messages: vec![Message {
                        role: Role::User,
                        content: MessageContent::Text("hi".to_string()),
                    }],
                    tools: None,
                    temperature: None,
                    max_tokens: None,
                    stream: false,
                    extra: HashMap::new(),
                },
                &[],
            )
            .await
            .expect("complete");

        assert_eq!(response.model, "openai/gpt-5.2");
        let lock = attempts.lock().expect("lock");
        assert_eq!(*lock, 2);
    }

    #[tokio::test]
    async fn fails_over_to_secondary_model() {
        let mut registry = ModelRegistry::default();
        registry.upsert_model(ModelDefinition {
            provider: "openai".to_string(),
            model_id: "gpt-5.2".to_string(),
            context_window: 1,
            max_tokens: 1,
            input_cost_per_mtok: 1.0,
            output_cost_per_mtok: 1.0,
            capabilities: ModelCapabilities::default(),
        });
        registry.upsert_model(ModelDefinition {
            provider: "anthropic".to_string(),
            model_id: "claude-sonnet-4-6".to_string(),
            context_window: 1,
            max_tokens: 1,
            input_cost_per_mtok: 1.0,
            output_cost_per_mtok: 1.0,
            capabilities: ModelCapabilities::default(),
        });

        let mut router = ProviderRouter::new(registry);
        router.retry.max_retries = 0;

        router.register_provider(Arc::new(MockProvider {
            name: "openai".to_string(),
            attempts: Arc::new(Mutex::new(0)),
            fail_first: true,
        }));
        router.register_provider(Arc::new(MockProvider {
            name: "anthropic".to_string(),
            attempts: Arc::new(Mutex::new(0)),
            fail_first: false,
        }));

        let response = router
            .complete_with_failover(
                CompletionRequest {
                    model: "openai/gpt-5.2".to_string(),
                    messages: vec![Message {
                        role: Role::User,
                        content: MessageContent::Text("hi".to_string()),
                    }],
                    tools: None,
                    temperature: None,
                    max_tokens: None,
                    stream: false,
                    extra: HashMap::new(),
                },
                &["anthropic/claude-sonnet-4-6".to_string()],
            )
            .await
            .expect("complete");

        assert_eq!(response.model, "anthropic/claude-sonnet-4-6");
    }
}

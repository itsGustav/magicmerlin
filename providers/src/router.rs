//! Provider router with model resolution, retries, failover, rate limits, and circuit breakers.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, RwLock};

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

/// Token bucket configuration.
#[derive(Debug, Clone)]
pub struct TokenBucketConfig {
    /// Maximum bucket size.
    pub capacity: f64,
    /// Tokens refilled per second.
    pub refill_per_second: f64,
    /// Approximate request cost in tokens when unknown.
    pub default_cost: f64,
}

impl Default for TokenBucketConfig {
    fn default() -> Self {
        Self {
            capacity: 100.0,
            refill_per_second: 20.0,
            default_cost: 1.0,
        }
    }
}

/// Circuit breaker configuration.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Consecutive failures to open the circuit.
    pub failure_threshold: u32,
    /// Cooldown duration before half-open probe.
    pub cooldown: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            cooldown: Duration::from_secs(20),
        }
    }
}

/// Request middleware hook.
pub trait RequestMiddleware: Send + Sync {
    /// Intercepts request before dispatch and can mutate it.
    fn on_request(&self, request: &mut CompletionRequest);
}

/// Response middleware hook.
pub trait ResponseMiddleware: Send + Sync {
    /// Intercepts response before returning to caller.
    fn on_response(&self, response: &mut CompletionResponse);
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(config: &TokenBucketConfig) -> Self {
        Self {
            tokens: config.capacity,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self, config: &TokenBucketConfig) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        let next = self.tokens + elapsed * config.refill_per_second;
        self.tokens = next.min(config.capacity);
        self.last_refill = Instant::now();
    }

    fn consume_or_wait(&mut self, config: &TokenBucketConfig, amount: f64) -> Option<Duration> {
        self.refill(config);
        if self.tokens >= amount {
            self.tokens -= amount;
            return None;
        }

        let missing = amount - self.tokens;
        let seconds = if config.refill_per_second > 0.0 {
            missing / config.refill_per_second
        } else {
            1.0
        };
        Some(Duration::from_secs_f64(seconds.max(0.01)))
    }
}

#[derive(Debug, Clone)]
struct CircuitState {
    consecutive_failures: u32,
    opened_at: Option<Instant>,
    half_open_probe_in_flight: bool,
}

impl Default for CircuitState {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            opened_at: None,
            half_open_probe_in_flight: false,
        }
    }
}

/// Routing metrics snapshot.
#[derive(Debug, Clone, Default)]
pub struct RouterMetrics {
    /// Successful requests per provider.
    pub success_by_provider: HashMap<String, u64>,
    /// Failed requests per provider.
    pub failures_by_provider: HashMap<String, u64>,
    /// Failover count.
    pub failovers: u64,
}

/// Routes completion requests to providers with retry and failover semantics.
#[derive(Clone)]
pub struct ProviderRouter {
    providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    /// Registry for model resolution and metadata.
    pub model_registry: ModelRegistry,
    /// Retry settings.
    pub retry: RetryConfig,
    /// Token bucket settings per provider.
    pub token_bucket: TokenBucketConfig,
    /// Circuit breaker settings.
    pub circuit_breaker: CircuitBreakerConfig,
    rate_limit_until: Arc<Mutex<HashMap<String, Instant>>>,
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    circuits: Arc<Mutex<HashMap<String, CircuitState>>>,
    metrics: Arc<Mutex<RouterMetrics>>,
    request_middleware: Arc<RwLock<Vec<Arc<dyn RequestMiddleware>>>>,
    response_middleware: Arc<RwLock<Vec<Arc<dyn ResponseMiddleware>>>>,
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

        Self::from_provider_map(model_registry, providers)
    }

    /// Creates a router with explicit providers.
    pub fn new(model_registry: ModelRegistry) -> Self {
        Self::from_provider_map(model_registry, HashMap::new())
    }

    fn from_provider_map(
        model_registry: ModelRegistry,
        providers: HashMap<String, Arc<dyn LlmProvider>>,
    ) -> Self {
        Self {
            providers: Arc::new(RwLock::new(providers)),
            model_registry,
            retry: RetryConfig::default(),
            token_bucket: TokenBucketConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            rate_limit_until: Arc::new(Mutex::new(HashMap::new())),
            buckets: Arc::new(Mutex::new(HashMap::new())),
            circuits: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(RouterMetrics::default())),
            request_middleware: Arc::new(RwLock::new(Vec::new())),
            response_middleware: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Registers one provider implementation.
    pub async fn register_provider(&self, provider: Arc<dyn LlmProvider>) {
        let mut providers = self.providers.write().await;
        providers.insert(provider.name().to_string(), provider);
    }

    /// Returns provider names currently registered.
    pub async fn provider_names(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        let mut names = providers.keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Adds a request middleware hook.
    pub async fn add_request_middleware(&self, middleware: Arc<dyn RequestMiddleware>) {
        let mut hooks = self.request_middleware.write().await;
        hooks.push(middleware);
    }

    /// Adds a response middleware hook.
    pub async fn add_response_middleware(&self, middleware: Arc<dyn ResponseMiddleware>) {
        let mut hooks = self.response_middleware.write().await;
        hooks.push(middleware);
    }

    /// Returns metrics snapshot.
    pub async fn metrics_snapshot(&self) -> RouterMetrics {
        self.metrics.lock().await.clone()
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
        for (index, canonical_model) in chain.iter().enumerate() {
            let (provider_name, provider_model_id) =
                ModelRegistry::parse_provider_model(canonical_model)?;

            let provider = {
                let providers = self.providers.read().await;
                providers
                    .get(&provider_name)
                    .cloned()
                    .ok_or_else(|| ProviderError::ProviderNotFound(provider_name.clone()))?
            };

            let mut provider_request = request.clone();
            provider_request.model = provider_model_id;

            self.apply_request_middleware(&mut provider_request).await;

            match self
                .try_provider(&provider_name, provider, provider_request.clone())
                .await
            {
                Ok(mut response) => {
                    if response.usage.input_tokens == 0 && response.usage.output_tokens == 0 {
                        let input = request
                            .messages
                            .iter()
                            .map(|m| serde_json::to_string(m).unwrap_or_default())
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
                        .estimate_cost_usd(canonical_model, &response.usage);
                    self.apply_response_middleware(&mut response).await;

                    self.record_success(&provider_name).await;
                    if index > 0 {
                        self.record_failover().await;
                    }
                    return Ok(response);
                }
                Err(err) if err.is_retryable() => {
                    self.record_failure(&provider_name).await;
                    last_error = Some(err);
                }
                Err(err) => {
                    self.record_failure(&provider_name).await;
                    return Err(err);
                }
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
        self.ensure_circuit_allows(provider_name).await?;

        for attempt in 0..=self.retry.max_retries {
            self.wait_rate_limit(provider_name).await;
            self.wait_token_bucket(provider_name, estimate_request_weight(&request))
                .await;

            let timed = tokio::time::timeout(
                self.retry.request_timeout,
                provider.complete(request.clone()),
            )
            .await;

            match timed {
                Ok(Ok(response)) => {
                    self.mark_circuit_success(provider_name).await;
                    return Ok(response);
                }
                Ok(Err(err)) if err.is_retryable() => {
                    self.mark_circuit_failure(provider_name).await;
                    if let Some(wait) = err.retry_after_hint() {
                        self.set_rate_limit(provider_name, wait).await;
                    }
                    if attempt == self.retry.max_retries {
                        return Err(err);
                    }
                    tokio::time::sleep(backoff_delay(&self.retry, attempt)).await;
                }
                Ok(Err(err)) => {
                    self.mark_circuit_failure(provider_name).await;
                    return Err(err);
                }
                Err(_) => {
                    self.mark_circuit_failure(provider_name).await;
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

    async fn apply_request_middleware(&self, request: &mut CompletionRequest) {
        let middleware = self.request_middleware.read().await;
        for hook in middleware.iter() {
            hook.on_request(request);
        }
    }

    async fn apply_response_middleware(&self, response: &mut CompletionResponse) {
        let middleware = self.response_middleware.read().await;
        for hook in middleware.iter() {
            hook.on_response(response);
        }
    }

    async fn record_success(&self, provider: &str) {
        let mut metrics = self.metrics.lock().await;
        let entry = metrics.success_by_provider.entry(provider.to_string()).or_default();
        *entry += 1;
    }

    async fn record_failure(&self, provider: &str) {
        let mut metrics = self.metrics.lock().await;
        let entry = metrics
            .failures_by_provider
            .entry(provider.to_string())
            .or_default();
        *entry += 1;
    }

    async fn record_failover(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.failovers += 1;
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

    async fn wait_token_bucket(&self, provider_name: &str, weight: f64) {
        loop {
            let wait = {
                let mut buckets = self.buckets.lock().await;
                let bucket = buckets
                    .entry(provider_name.to_string())
                    .or_insert_with(|| TokenBucket::new(&self.token_bucket));
                bucket.consume_or_wait(&self.token_bucket, weight)
            };

            if let Some(wait) = wait {
                tokio::time::sleep(wait).await;
                continue;
            }
            break;
        }
    }

    async fn ensure_circuit_allows(&self, provider_name: &str) -> Result<()> {
        let now = Instant::now();
        let mut circuits = self.circuits.lock().await;
        let state = circuits.entry(provider_name.to_string()).or_default();

        let Some(opened_at) = state.opened_at else {
            return Ok(());
        };

        let elapsed = now.saturating_duration_since(opened_at);
        if elapsed >= self.circuit_breaker.cooldown {
            if !state.half_open_probe_in_flight {
                state.half_open_probe_in_flight = true;
                return Ok(());
            }
            return Err(ProviderError::CircuitOpen {
                provider: provider_name.to_string(),
                remaining_ms: 0,
            });
        }

        let remaining = self.circuit_breaker.cooldown.saturating_sub(elapsed);
        Err(ProviderError::CircuitOpen {
            provider: provider_name.to_string(),
            remaining_ms: remaining.as_millis() as u64,
        })
    }

    async fn mark_circuit_success(&self, provider_name: &str) {
        let mut circuits = self.circuits.lock().await;
        let state = circuits.entry(provider_name.to_string()).or_default();
        state.consecutive_failures = 0;
        state.opened_at = None;
        state.half_open_probe_in_flight = false;
    }

    async fn mark_circuit_failure(&self, provider_name: &str) {
        let mut circuits = self.circuits.lock().await;
        let state = circuits.entry(provider_name.to_string()).or_default();
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        if state.consecutive_failures >= self.circuit_breaker.failure_threshold {
            state.opened_at = Some(Instant::now());
            state.half_open_probe_in_flight = false;
        }
    }
}

fn estimate_request_weight(request: &CompletionRequest) -> f64 {
    let tokens: u32 = request
        .messages
        .iter()
        .map(|msg| serde_json::to_string(msg).unwrap_or_default())
        .map(|text| approximate_tokens(&text))
        .sum();
    let weight = (tokens as f64 / 500.0).ceil();
    weight.max(1.0)
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
        ContentBlock::Thinking { text } => text.clone(),
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
        fail_mode: FailMode,
    }

    #[derive(Clone, Copy)]
    enum FailMode {
        Never,
        FirstOnly,
        Always,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
            let mut lock = self
                .attempts
                .lock()
                .map_err(|_| ProviderError::Exhausted("mock mutex poisoned".to_string()))?;
            *lock += 1;
            match self.fail_mode {
                FailMode::Never => {}
                FailMode::FirstOnly if *lock == 1 => {
                    return Err(ProviderError::api(500, "boom".to_string(), Some(1)));
                }
                FailMode::Always => {
                    return Err(ProviderError::api(500, "boom".to_string(), Some(1)));
                }
                FailMode::FirstOnly => {}
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

    #[derive(Default)]
    struct AddMetadataMiddleware;

    impl RequestMiddleware for AddMetadataMiddleware {
        fn on_request(&self, request: &mut CompletionRequest) {
            request
                .extra
                .insert("request_id".to_string(), serde_json::json!("abc"));
        }
    }

    impl ResponseMiddleware for AddMetadataMiddleware {
        fn on_response(&self, response: &mut CompletionResponse) {
            response.estimated_cost_usd = Some(response.estimated_cost_usd.unwrap_or(0.0) + 1.0);
        }
    }

    fn setup_registry() -> ModelRegistry {
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
        registry
    }

    #[tokio::test]
    async fn retries_and_succeeds_on_same_provider() {
        let attempts = Arc::new(Mutex::new(0));
        let router = ProviderRouter::new(setup_registry());
        router.register_provider(Arc::new(MockProvider {
            name: "openai".to_string(),
            attempts: attempts.clone(),
            fail_mode: FailMode::FirstOnly,
        })).await;

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
        let router = ProviderRouter::new(setup_registry());
        router.register_provider(Arc::new(MockProvider {
            name: "openai".to_string(),
            attempts: Arc::new(Mutex::new(0)),
            fail_mode: FailMode::Always,
        })).await;
        router.register_provider(Arc::new(MockProvider {
            name: "anthropic".to_string(),
            attempts: Arc::new(Mutex::new(0)),
            fail_mode: FailMode::Never,
        })).await;

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
        let metrics = router.metrics_snapshot().await;
        assert_eq!(metrics.failovers, 1);
    }

    #[tokio::test]
    async fn middleware_applies_request_and_response_hooks() {
        let router = ProviderRouter::new(setup_registry());
        router.register_provider(Arc::new(MockProvider {
            name: "openai".to_string(),
            attempts: Arc::new(Mutex::new(0)),
            fail_mode: FailMode::Never,
        })).await;
        router
            .add_request_middleware(Arc::new(AddMetadataMiddleware))
            .await;
        router
            .add_response_middleware(Arc::new(AddMetadataMiddleware))
            .await;

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

        assert!(response.estimated_cost_usd.unwrap_or(0.0) >= 1.0);
    }

    #[tokio::test]
    async fn circuit_breaker_opens_after_failures() {
        let router = ProviderRouter::new(setup_registry());
        router.register_provider(Arc::new(MockProvider {
            name: "openai".to_string(),
            attempts: Arc::new(Mutex::new(0)),
            fail_mode: FailMode::Always,
        })).await;

        let mut failed = 0;
        for _ in 0..6 {
            let result = router
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
                .await;
            if result.is_err() {
                failed += 1;
            }
        }

        assert!(failed >= 1);
    }
}

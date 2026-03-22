//! Shared test helpers for integration tests.

use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use magicmerlin_agent::{AgentError, ToolExecutionResult, ToolExecutor};
use magicmerlin_providers::providers::{LlmProvider, ProviderStream};
use magicmerlin_providers::types::{
    CompletionRequest, CompletionResponse, ContentBlock, StopReason, ToolCall, Usage,
};
use magicmerlin_providers::ProviderError;

/// Find a free TCP port on localhost.
pub fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind to random port");
    listener.local_addr().expect("local addr").port()
}

/// Returns the path to a cargo binary in the workspace target directory.
pub fn cargo_bin(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // tests/
    path.pop(); // workspace root
    path.push("target");
    path.push("debug");
    path.push(name);
    path
}

// ---------------------------------------------------------------------------
// Mock LLM Provider
// ---------------------------------------------------------------------------

/// A mock LLM provider that returns a single text response on every call.
#[derive(Clone)]
pub struct SingleShotProvider {
    pub response_text: String,
}

impl SingleShotProvider {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            response_text: text.into(),
        }
    }
}

#[async_trait]
impl LlmProvider for SingleShotProvider {
    async fn complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        Ok(CompletionResponse {
            id: "mock-1".to_string(),
            model: "mock".to_string(),
            content: vec![ContentBlock::Text {
                text: self.response_text.clone(),
            }],
            tool_calls: Vec::new(),
            usage: Usage::default(),
            stop_reason: StopReason::EndTurn,
            estimated_cost_usd: None,
        })
    }

    async fn complete_stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<ProviderStream, ProviderError> {
        Ok(Vec::new())
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

/// A two-step mock provider: first returns a tool call, then returns final text.
#[derive(Clone)]
pub struct TwoStepProvider {
    pub calls: Arc<Mutex<u32>>,
    pub final_text: String,
}

impl TwoStepProvider {
    pub fn new(final_text: impl Into<String>) -> Self {
        Self {
            calls: Arc::new(Mutex::new(0)),
            final_text: final_text.into(),
        }
    }
}

#[async_trait]
impl LlmProvider for TwoStepProvider {
    async fn complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let mut lock = self.calls.lock().expect("lock");
        *lock += 1;
        if *lock == 1 {
            return Ok(CompletionResponse {
                id: "step-1".to_string(),
                model: "mock".to_string(),
                content: vec![ContentBlock::Text {
                    text: "need tool".to_string(),
                }],
                tool_calls: vec![ToolCall {
                    id: "t1".to_string(),
                    name: "exec".to_string(),
                    arguments: serde_json::json!({"cmd": "echo hello"}),
                }],
                usage: Usage::default(),
                stop_reason: StopReason::ToolUse,
                estimated_cost_usd: None,
            });
        }

        Ok(CompletionResponse {
            id: "step-2".to_string(),
            model: "mock".to_string(),
            content: vec![ContentBlock::Text {
                text: self.final_text.clone(),
            }],
            tool_calls: Vec::new(),
            usage: Usage::default(),
            stop_reason: StopReason::EndTurn,
            estimated_cost_usd: None,
        })
    }

    async fn complete_stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<ProviderStream, ProviderError> {
        Ok(Vec::new())
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Mock Tool Executor
// ---------------------------------------------------------------------------

/// A mock tool executor that always returns success with a fixed output.
pub struct MockTools;

#[async_trait]
impl ToolExecutor for MockTools {
    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolExecutionResult, AgentError> {
        Ok(ToolExecutionResult::ok(
            tool_call.id.clone(),
            "tool output".to_string(),
        ))
    }
}

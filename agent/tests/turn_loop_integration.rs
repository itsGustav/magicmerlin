use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use magicmerlin_agent::{
    AgentEngine, AgentEngineConfig, SessionKey, SessionManager, ToolExecutionResult, ToolExecutor,
};
use magicmerlin_providers::model_registry::{ModelCapabilities, ModelDefinition, ModelRegistry};
use magicmerlin_providers::providers::{LlmProvider, ProviderStream};
use magicmerlin_providers::types::{
    CompletionRequest, CompletionResponse, ContentBlock, StopReason, ToolCall, Usage,
};
use magicmerlin_providers::{ProviderError, ProviderRouter};
use magicmerlin_storage::Storage;

#[derive(Clone)]
struct TwoStepProvider {
    calls: Arc<Mutex<u32>>,
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
                id: "1".to_string(),
                model: "gpt-5.2".to_string(),
                content: vec![ContentBlock::Text {
                    text: "need tool".to_string(),
                }],
                tool_calls: vec![ToolCall {
                    id: "t1".to_string(),
                    name: "exec".to_string(),
                    arguments: serde_json::json!({"cmd":"echo hi"}),
                }],
                usage: Usage::default(),
                stop_reason: StopReason::ToolUse,
                estimated_cost_usd: None,
            });
        }

        Ok(CompletionResponse {
            id: "2".to_string(),
            model: "gpt-5.2".to_string(),
            content: vec![ContentBlock::Text {
                text: "final answer".to_string(),
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

struct MockTools;

#[async_trait]
impl ToolExecutor for MockTools {
    async fn execute_tools(
        &self,
        tool_calls: &[ToolCall],
    ) -> Result<Vec<ToolExecutionResult>, magicmerlin_agent::AgentError> {
        Ok(tool_calls
            .iter()
            .map(|c| ToolExecutionResult {
                tool_call_id: c.id.clone(),
                content: "tool output".to_string(),
            })
            .collect())
    }
}

#[tokio::test]
async fn runs_turn_with_tool_round_trip() {
    let mut models = ModelRegistry::default();
    models.upsert_model(ModelDefinition {
        provider: "openai".to_string(),
        model_id: "gpt-5.2".to_string(),
        context_window: 128_000,
        max_tokens: 8_192,
        input_cost_per_mtok: 0.0,
        output_cost_per_mtok: 0.0,
        capabilities: ModelCapabilities::default(),
    });

    let mut router = ProviderRouter::new(models);
    router.register_provider(Arc::new(TwoStepProvider {
        calls: Arc::new(Mutex::new(0)),
    }));

    let temp = tempfile::tempdir().expect("tmp");
    let storage = Storage::new(temp.path().join("db.sqlite")).expect("storage");
    let sessions =
        SessionManager::new(storage, temp.path().join("sessions"), temp.path()).expect("sessions");

    let mut session = sessions
        .load_or_create(SessionKey::agent_main("merlin"), "merlin")
        .expect("session");

    let engine = AgentEngine::new(
        Arc::new(router),
        sessions,
        AgentEngineConfig {
            model: "openai/gpt-5.2".to_string(),
            workspace_dir: temp.path().to_path_buf(),
            agent_dir: temp.path().to_path_buf(),
            ..AgentEngineConfig::default()
        },
    );

    let reply = engine
        .run_turn(&mut session, "hello", &MockTools)
        .await
        .expect("run");
    assert_eq!(reply.text, "final answer");
}

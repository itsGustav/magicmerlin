//! Core agent turn loop and tool-call orchestration.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use magicmerlin_providers::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, Message,
    MessageContent, Role, ToolCall,
};
use serde_json::json;

use crate::error::{AgentError, Result};
use crate::session::SessionRecord;
use crate::system_prompt::{
    InboundContext, PromptRuntimeMetadata, SystemPromptAssembler, ToolSchemaDescriptor,
};
use crate::SessionManager;

/// Agent engine configuration.
#[derive(Debug, Clone)]
pub struct AgentEngineConfig {
    /// Primary model identifier.
    pub model: String,
    /// Fallback model chain.
    pub fallbacks: Vec<String>,
    /// Context window target.
    pub context_window: u64,
    /// Context utilization threshold triggering compaction.
    pub compact_threshold_pct: u64,
    /// Maximum tool-call rounds per turn.
    pub max_tool_rounds: usize,
    /// Maximum total agent rounds per run.
    pub max_turns: usize,
    /// Max token budget for one run.
    pub token_budget: u64,
    /// Max tool timeout.
    pub tool_timeout: Duration,
    /// Agent name.
    pub agent_name: String,
    /// Agent directory path.
    pub agent_dir: PathBuf,
    /// Workspace directory path.
    pub workspace_dir: PathBuf,
    /// Channel name.
    pub channel: String,
    /// Timezone name.
    pub timezone: String,
    /// Whether provider streaming should be used when available.
    pub stream_responses: bool,
}

impl Default for AgentEngineConfig {
    fn default() -> Self {
        Self {
            model: "openai/gpt-5.2".to_string(),
            fallbacks: vec!["anthropic/claude-sonnet-4-6".to_string()],
            context_window: 128_000,
            compact_threshold_pct: 85,
            max_tool_rounds: 8,
            max_turns: 12,
            token_budget: 120_000,
            tool_timeout: Duration::from_secs(30),
            agent_name: "merlin".to_string(),
            agent_dir: PathBuf::from("."),
            workspace_dir: PathBuf::from("."),
            channel: "terminal".to_string(),
            timezone: "UTC".to_string(),
            stream_responses: true,
        }
    }
}

/// Tool execution response.
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    /// Tool call id.
    pub tool_call_id: String,
    /// Tool output content.
    pub content: String,
    /// Whether execution failed.
    pub is_error: bool,
}

impl ToolExecutionResult {
    /// Creates success tool result.
    pub fn ok(tool_call_id: String, content: String) -> Self {
        Self {
            tool_call_id,
            content,
            is_error: false,
        }
    }

    /// Creates failure tool result.
    pub fn err(tool_call_id: String, error: String) -> Self {
        Self {
            tool_call_id,
            content: error,
            is_error: true,
        }
    }
}

/// Tool execution abstraction consumed by agent loop.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Executes one tool call.
    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolExecutionResult>;

    /// Executes multiple calls concurrently if implementation supports it.
    async fn execute_tools_parallel(&self, tool_calls: &[ToolCall]) -> Result<Vec<ToolExecutionResult>> {
        let mut results = Vec::new();
        for call in tool_calls {
            results.push(self.execute_tool(call).await?);
        }
        Ok(results)
    }
}

/// Abort signal for in-progress turn.
#[derive(Debug, Clone, Default)]
pub struct AbortSignal {
    cancelled: Arc<AtomicBool>,
}

impl AbortSignal {
    /// Creates a new signal.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Triggers cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Final agent reply payload.
#[derive(Debug, Clone)]
pub struct AgentReply {
    /// Plain-text assistant response.
    pub text: String,
    /// Number of rounds executed.
    pub rounds: usize,
    /// Token estimate used in this turn.
    pub token_estimate: u64,
}

/// Turn-loop runtime over provider router and session manager.
#[derive(Clone)]
pub struct AgentEngine {
    router: Arc<magicmerlin_providers::ProviderRouter>,
    sessions: SessionManager,
    config: AgentEngineConfig,
}

impl AgentEngine {
    /// Creates a new agent engine.
    pub fn new(
        router: Arc<magicmerlin_providers::ProviderRouter>,
        sessions: SessionManager,
        config: AgentEngineConfig,
    ) -> Self {
        Self {
            router,
            sessions,
            config,
        }
    }

    /// Runs one complete turn, including recursive tool execution rounds.
    pub async fn run_turn(
        &self,
        session: &mut SessionRecord,
        user_message: &str,
        tools: &dyn ToolExecutor,
    ) -> Result<AgentReply> {
        self.run_turn_with_options(
            session,
            user_message,
            tools,
            &InboundContext::default(),
            &[],
            None,
        )
        .await
    }

    /// Runs one turn with explicit context and optional abort signal.
    pub async fn run_turn_with_options(
        &self,
        session: &mut SessionRecord,
        user_message: &str,
        tools: &dyn ToolExecutor,
        inbound_context: &InboundContext,
        tool_schemas: &[ToolSchemaDescriptor],
        abort_signal: Option<&AbortSignal>,
    ) -> Result<AgentReply> {
        self.check_abort(abort_signal)?;

        // Pre-turn compaction check with context % logging
        let ctx_pct = self.sessions.estimate_context_percent(session, self.config.context_window);
        if let Some(compaction) = self.sessions.compact_if_needed(
            session,
            self.config.context_window,
            self.config.compact_threshold_pct,
        )? {
            tracing::info!(
                "Context at {:.0}%, compacted: {} msgs → {}, {} tokens → {}",
                ctx_pct * 100.0,
                compaction.messages_before,
                compaction.messages_after,
                compaction.tokens_before,
                compaction.tokens_after,
            );
        }

        self.sessions
            .append_message(session, json!({"role":"user","content":user_message}))?;

        let assembler = SystemPromptAssembler::new(&self.config.workspace_dir, &self.config.agent_dir, 8_000);
        let system_prompt = assembler.assemble_with_context(
            &PromptRuntimeMetadata::now(
                self.config.model.clone(),
                self.config.channel.clone(),
                self.config.timezone.clone(),
                self.config.agent_name.clone(),
            ),
            inbound_context,
            tool_schemas,
        )?;

        let transcript_values = session.transcript.read(0, None)?;
        let mut messages = vec![Message {
            role: Role::System,
            content: MessageContent::Text(system_prompt),
        }];
        messages.extend(transcript_values.iter().filter_map(value_to_message));

        let mut rounds = 0_usize;
        let mut tool_rounds = 0_usize;
        let mut consumed_tokens = estimate_messages_tokens(&messages);

        loop {
            self.check_abort(abort_signal)?;
            rounds = rounds.saturating_add(1);
            if rounds > self.config.max_turns {
                return Err(AgentError::InvalidState(
                    "agent loop exceeded max turns".to_string(),
                ));
            }

            if consumed_tokens > self.config.token_budget {
                return Err(AgentError::InvalidState(format!(
                    "turn exceeded token budget: {} > {}",
                    consumed_tokens, self.config.token_budget
                )));
            }

            let response = self
                .request_completion(messages.clone())
                .await?;

            let assistant_text = response
                .content
                .iter()
                .map(content_to_text)
                .collect::<Vec<_>>()
                .join("\n");
            consumed_tokens = consumed_tokens
                .saturating_add(estimate_response_tokens(&response) as u64)
                .saturating_add(approximate_tokens(&assistant_text) as u64);

            self.sessions.append_message(
                session,
                json!({
                    "role":"assistant",
                    "content": assistant_text,
                    "tool_calls": response.tool_calls,
                }),
            )?;

            messages.push(Message {
                role: Role::Assistant,
                content: MessageContent::Text(assistant_text.clone()),
            });

            if response.tool_calls.is_empty() {
                return Ok(AgentReply {
                    text: assistant_text,
                    rounds,
                    token_estimate: consumed_tokens,
                });
            }

            tool_rounds = tool_rounds.saturating_add(1);
            if tool_rounds > self.config.max_tool_rounds {
                return Err(AgentError::InvalidState(
                    "tool loop exceeded max rounds".to_string(),
                ));
            }

            let tool_results = self
                .execute_tools_with_timeout(tools, &response.tool_calls, abort_signal)
                .await?;

            for result in tool_results {
                self.sessions.append_message(
                    session,
                    json!({
                        "role":"tool",
                        "tool_call_id":result.tool_call_id,
                        "content":result.content,
                        "is_error": result.is_error,
                    }),
                )?;

                messages.push(Message {
                    role: Role::Tool,
                    content: MessageContent::ToolResult(magicmerlin_providers::ToolResultContent {
                        tool_call_id: result.tool_call_id,
                        content: result.content,
                    }),
                });
            }
        }
    }

    async fn request_completion(&self, messages: Vec<Message>) -> Result<CompletionResponse> {
        let mut extra = HashMap::new();
        extra.insert("agent_name".to_string(), json!(self.config.agent_name));

        let request = CompletionRequest {
            model: self.config.model.clone(),
            messages,
            tools: None,
            temperature: None,
            max_tokens: None,
            stream: self.config.stream_responses,
            extra,
        };

        self.router
            .complete_with_failover(request, &self.config.fallbacks)
            .await
            .map_err(AgentError::from)
    }

    async fn execute_tools_with_timeout(
        &self,
        tools: &dyn ToolExecutor,
        tool_calls: &[ToolCall],
        abort_signal: Option<&AbortSignal>,
    ) -> Result<Vec<ToolExecutionResult>> {
        self.check_abort(abort_signal)?;

        let execution = tools.execute_tools_parallel(tool_calls);
        let timed = tokio::time::timeout(self.config.tool_timeout, execution).await;

        self.check_abort(abort_signal)?;

        match timed {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(err)) => Err(err),
            Err(_) => {
                let timeout_results = tool_calls
                    .iter()
                    .map(|call| {
                        ToolExecutionResult::err(
                            call.id.clone(),
                            format!("tool timeout after {:?}", self.config.tool_timeout),
                        )
                    })
                    .collect::<Vec<_>>();
                Ok(timeout_results)
            }
        }
    }

    fn check_abort(&self, abort_signal: Option<&AbortSignal>) -> Result<()> {
        if abort_signal.map(|signal| signal.is_cancelled()).unwrap_or(false) {
            return Err(AgentError::Cancelled("agent turn aborted".to_string()));
        }
        Ok(())
    }
}

fn value_to_message(value: &serde_json::Value) -> Option<Message> {
    let role = match value.get("role")?.as_str()? {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        "system" => Role::System,
        _ => return None,
    };
    let content = value
        .get("content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    Some(Message {
        role,
        content: MessageContent::Text(content),
    })
}

fn content_to_text(content: &ContentBlock) -> String {
    match content {
        ContentBlock::Text { text } => text.clone(),
        ContentBlock::Json { value } => value.to_string(),
        ContentBlock::Thinking { text } => text.clone(),
    }
}

fn estimate_messages_tokens(messages: &[Message]) -> u64 {
    messages
        .iter()
        .map(|msg| serde_json::to_string(msg).unwrap_or_default())
        .map(|text| approximate_tokens(&text) as u64)
        .sum()
}

fn estimate_response_tokens(response: &CompletionResponse) -> u32 {
    response
        .content
        .iter()
        .map(content_to_text)
        .map(|text| approximate_tokens(&text))
        .sum()
}

//! Core agent turn loop and tool-call orchestration.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use magicmerlin_providers::types::{
    CompletionRequest, ContentBlock, Message, MessageContent, Role, ToolCall,
};
use serde_json::json;

use crate::error::{AgentError, Result};
use crate::session::SessionRecord;
use crate::system_prompt::{PromptRuntimeMetadata, SystemPromptAssembler};
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
}

impl Default for AgentEngineConfig {
    fn default() -> Self {
        Self {
            model: "openai/gpt-5.2".to_string(),
            fallbacks: vec!["anthropic/claude-sonnet-4-6".to_string()],
            context_window: 128_000,
            compact_threshold_pct: 85,
            max_tool_rounds: 8,
            agent_name: "merlin".to_string(),
            agent_dir: PathBuf::from("."),
            workspace_dir: PathBuf::from("."),
            channel: "terminal".to_string(),
            timezone: "UTC".to_string(),
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
}

/// Tool execution abstraction consumed by agent loop.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Executes tool calls and returns tool results.
    async fn execute_tools(&self, tool_calls: &[ToolCall]) -> Result<Vec<ToolExecutionResult>>;
}

/// Final agent reply payload.
#[derive(Debug, Clone)]
pub struct AgentReply {
    /// Plain-text assistant response.
    pub text: String,
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
        self.sessions.compact_if_needed(
            session,
            self.config.context_window,
            self.config.compact_threshold_pct,
        )?;

        self.sessions
            .append_message(session, json!({"role":"user","content":user_message}))?;

        let assembler =
            SystemPromptAssembler::new(&self.config.workspace_dir, &self.config.agent_dir, 8_000);
        let system_prompt = assembler.assemble(&PromptRuntimeMetadata {
            model: self.config.model.clone(),
            channel: self.config.channel.clone(),
            timezone: self.config.timezone.clone(),
        })?;

        let transcript_values = session.transcript.read(0, None)?;
        let mut messages = vec![Message {
            role: Role::System,
            content: MessageContent::Text(system_prompt),
        }];
        messages.extend(transcript_values.iter().filter_map(value_to_message));

        let mut rounds = 0_usize;
        loop {
            let response = self
                .router
                .complete_with_failover(
                    CompletionRequest {
                        model: self.config.model.clone(),
                        messages: messages.clone(),
                        tools: None,
                        temperature: None,
                        max_tokens: None,
                        stream: false,
                        extra: std::collections::HashMap::new(),
                    },
                    &self.config.fallbacks,
                )
                .await?;

            let assistant_text = response
                .content
                .iter()
                .map(content_to_text)
                .collect::<Vec<_>>()
                .join("\n");

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
                });
            }

            rounds += 1;
            if rounds > self.config.max_tool_rounds {
                return Err(AgentError::InvalidState(
                    "tool loop exceeded max rounds".to_string(),
                ));
            }

            let tool_results = tools.execute_tools(&response.tool_calls).await?;
            for result in tool_results {
                self.sessions.append_message(
                    session,
                    json!({"role":"tool","tool_call_id":result.tool_call_id,"content":result.content}),
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
    }
}

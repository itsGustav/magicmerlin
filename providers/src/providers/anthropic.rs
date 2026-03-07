//! Anthropic Messages API provider.

use async_trait::async_trait;
use reqwest::header::{HeaderName, CONTENT_TYPE};
use serde_json::{json, Value};

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, MessageContent, Role,
    StopReason, StreamChunk, ToolCall, Usage,
};

/// Anthropic provider using `v1/messages`.
#[derive(Clone, Debug)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    auth: AuthProfiles,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
        }
    }

    fn endpoint(&self) -> &str {
        "https://api.anthropic.com/v1/messages"
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let (_, value) = self
            .auth
            .header_for_provider("anthropic", &self.client)
            .await?;

        let mut body = json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "messages": request
                .messages
                .iter()
                .filter(|m| !matches!(m.role, Role::System))
                .map(|m| json!({"role": role_name(m.role), "content": content_text(&m.content)}))
                .collect::<Vec<_>>(),
            "stream": request.stream,
        });

        if let Some(system) = request
            .messages
            .iter()
            .find(|m| matches!(m.role, Role::System))
            .map(|m| content_text(&m.content))
        {
            body["system"] = json!(system);
        }
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(tools) = request.tools {
            body["tools"] = json!(tools
                .into_iter()
                .map(|t| json!({"name": t.name, "description": t.description, "input_schema": t.parameters}))
                .collect::<Vec<_>>());
        }

        if let Some(map) = body.as_object_mut() {
            for (key, value) in request.extra {
                map.insert(key, value);
            }
        }

        let response = self
            .client
            .post(self.endpoint())
            .header(HeaderName::from_static("x-api-key"), value)
            .header(HeaderName::from_static("anthropic-version"), "2023-06-01")
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_else(|_| String::new());
            if status == 429 {
                self.auth.rotate_key("anthropic").await;
            }
            return Err(ProviderError::Api { status, body });
        }

        let raw = response.json::<Value>().await?;
        let mut content = Vec::new();
        let mut tool_calls = Vec::new();
        if let Some(blocks) = raw.get("content").and_then(Value::as_array) {
            for block in blocks {
                match block
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            content.push(ContentBlock::Text {
                                text: text.to_string(),
                            });
                        }
                    }
                    "tool_use" => {
                        tool_calls.push(ToolCall {
                            id: block
                                .get("id")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            name: block
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            arguments: block.get("input").cloned().unwrap_or_else(|| json!({})),
                        });
                    }
                    _ => {}
                }
            }
        }

        let usage = Usage {
            input_tokens: raw
                .get("usage")
                .and_then(|u| u.get("input_tokens"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or_else(|| approximate_tokens(&raw.to_string())),
            output_tokens: raw
                .get("usage")
                .and_then(|u| u.get("output_tokens"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0),
            cache_read: raw
                .get("usage")
                .and_then(|u| u.get("cache_read_input_tokens"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0),
            cache_write: raw
                .get("usage")
                .and_then(|u| u.get("cache_creation_input_tokens"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0),
        };

        Ok(CompletionResponse {
            id: raw
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            model: raw
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            content,
            tool_calls,
            usage,
            stop_reason: match raw
                .get("stop_reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "tool_use" => StopReason::ToolUse,
                "max_tokens" => StopReason::MaxTokens,
                "end_turn" => StopReason::EndTurn,
                _ => StopReason::Unknown,
            },
            estimated_cost_usd: None,
        })
    }

    async fn complete_stream(&self, mut request: CompletionRequest) -> Result<ProviderStream> {
        request.stream = true;
        let response = self.complete(request).await?;
        let chunk = StreamChunk {
            delta_content: response.content,
            tool_calls: response.tool_calls,
            usage: Some(response.usage),
            done: true,
        };
        Ok(vec![Ok(chunk)])
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn supports_model(&self, model_id: &str) -> bool {
        model_id.starts_with("claude")
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "user",
        Role::System => "user",
    }
}

fn content_text(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(s) => s.clone(),
        MessageContent::MultiPart(parts) => parts
            .iter()
            .filter_map(|part| match part {
                crate::types::ContentPart::Text { text } => Some(text.as_str()),
                crate::types::ContentPart::ImageUrl { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MessageContent::ToolUse(call) => call.arguments.to_string(),
        MessageContent::ToolResult(result) => result.content.clone(),
    }
}

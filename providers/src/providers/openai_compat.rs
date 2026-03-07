//! Shared implementation for OpenAI-compatible providers.

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, ContentPart, Message,
    MessageContent, Role, StopReason, StreamChunk, ToolCall, Usage,
};

/// OpenAI-compatible provider configuration.
#[derive(Clone, Debug)]
pub struct OpenAiCompatProvider {
    provider_name: String,
    base_url: String,
    auth_provider: String,
    client: reqwest::Client,
    auth: AuthProfiles,
}

impl OpenAiCompatProvider {
    /// Creates a new OpenAI-compatible provider.
    pub fn new(
        provider_name: impl Into<String>,
        base_url: impl Into<String>,
        auth_provider: impl Into<String>,
        auth: AuthProfiles,
    ) -> Self {
        Self {
            provider_name: provider_name.into(),
            base_url: base_url.into(),
            auth_provider: auth_provider.into(),
            client: reqwest::Client::new(),
            auth,
        }
    }

    /// Builds the OpenAI chat completions request body.
    pub fn build_chat_body(request: &CompletionRequest) -> Value {
        let mut body = json!({
            "model": request.model,
            "messages": request.messages.iter().map(message_to_json).collect::<Vec<_>>(),
            "stream": request.stream,
        });

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(tools) = &request.tools {
            body["tools"] = json!(tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.parameters,
                        }
                    })
                })
                .collect::<Vec<_>>());
        }

        if let Some(map) = body.as_object_mut() {
            for (key, value) in &request.extra {
                map.insert(key.clone(), value.clone());
            }
        }

        body
    }

    fn endpoint(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        if base.ends_with("/v1") {
            format!("{base}/chat/completions")
        } else {
            format!("{base}/v1/chat/completions")
        }
    }

    async fn send_completion(&self, request: &CompletionRequest) -> Result<reqwest::Response> {
        let (header_name, header_value) = self
            .auth
            .header_for_provider(&self.auth_provider, &self.client)
            .await?;
        let response = self
            .client
            .post(self.endpoint())
            .header(header_name, header_value)
            .header(CONTENT_TYPE, "application/json")
            .json(&Self::build_chat_body(request))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_else(|_| String::new());
        if status == 429 {
            self.auth.rotate_key(&self.auth_provider).await;
        }
        Err(ProviderError::Api { status, body })
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let response = self.send_completion(&request).await?;
        let raw = response.json::<Value>().await?;

        let choice = raw
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .cloned()
            .unwrap_or_else(|| json!({}));

        let message = choice.get("message").cloned().unwrap_or_else(|| json!({}));
        let text = message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        let mut tool_calls = Vec::new();
        if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let id = call
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let name = call
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let args_raw = call
                    .get("function")
                    .and_then(|f| f.get("arguments"))
                    .and_then(Value::as_str)
                    .unwrap_or("{}");
                let arguments = serde_json::from_str(args_raw).unwrap_or_else(|_| json!({}));
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments,
                });
            }
        }

        let usage = parse_usage(raw.get("usage"));
        let stop_reason = match choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .unwrap_or("stop")
        {
            "tool_calls" => StopReason::ToolUse,
            "length" => StopReason::MaxTokens,
            "content_filter" => StopReason::ContentFilter,
            "stop" => StopReason::StopSequence,
            _ => StopReason::Unknown,
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
                .unwrap_or(&request.model)
                .to_string(),
            content: if text.is_empty() {
                Vec::new()
            } else {
                vec![ContentBlock::Text { text }]
            },
            tool_calls,
            usage,
            stop_reason,
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
        &self.provider_name
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

fn message_to_json(message: &Message) -> Value {
    let role = match message.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };

    let mut obj = json!({ "role": role });
    match &message.content {
        MessageContent::Text(text) => {
            obj["content"] = json!(text);
        }
        MessageContent::MultiPart(parts) => {
            let payload = parts
                .iter()
                .map(|part| match part {
                    ContentPart::Text { text } => json!({"type":"text","text":text}),
                    ContentPart::ImageUrl { url } => {
                        json!({"type":"image_url","image_url":{"url":url}})
                    }
                })
                .collect::<Vec<_>>();
            obj["content"] = Value::Array(payload);
        }
        MessageContent::ToolUse(call) => {
            obj["content"] = Value::Null;
            obj["tool_calls"] = json!([
                {
                    "id": call.id,
                    "type": "function",
                    "function": {
                        "name": call.name,
                        "arguments": call.arguments.to_string(),
                    }
                }
            ]);
        }
        MessageContent::ToolResult(result) => {
            obj["tool_call_id"] = json!(result.tool_call_id);
            obj["content"] = json!(result.content);
        }
    }

    obj
}

fn parse_usage(v: Option<&Value>) -> Usage {
    let usage = v.cloned().unwrap_or_else(|| json!({}));
    let mut parsed = Usage {
        input_tokens: usage
            .get("prompt_tokens")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        output_tokens: usage
            .get("completion_tokens")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        cache_read: usage
            .get("cached_tokens")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        cache_write: 0,
    };

    if parsed.input_tokens == 0 && parsed.output_tokens == 0 {
        parsed.input_tokens = approximate_tokens(usage.to_string().as_str());
    }
    parsed
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::types::{Message, MessageContent, Role, ToolDefinition};

    #[test]
    fn request_formatting_includes_tools() {
        let request = CompletionRequest {
            model: "gpt-5.2".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text("hello".to_string()),
            }],
            tools: Some(vec![ToolDefinition {
                name: "exec".to_string(),
                description: "Run command".to_string(),
                parameters: json!({"type":"object"}),
            }]),
            temperature: Some(0.2),
            max_tokens: Some(100),
            stream: false,
            extra: HashMap::new(),
        };

        let body = OpenAiCompatProvider::build_chat_body(&request);
        assert_eq!(body["model"], json!("gpt-5.2"));
        assert!(body["tools"].is_array());
        assert_eq!(body["temperature"], json!(0.2));
        assert_eq!(body["max_tokens"], json!(100));
    }
}

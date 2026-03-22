//! Anthropic Messages API provider.

use async_trait::async_trait;
use reqwest::header::{HeaderName, CONTENT_TYPE};
use serde_json::{json, Value};

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, ContentPart,
    MessageContent, Role, StopReason, StreamChunk, ToolCall, Usage,
};

/// Anthropic provider using `v1/messages`.
#[derive(Clone, Debug)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    auth: AuthProfiles,
    endpoint: String,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
        }
    }

    /// Creates a provider with custom endpoint (useful for tests/proxies).
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    fn build_body(&self, request: &CompletionRequest) -> Value {
        let mut body = json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "messages": request
                .messages
                .iter()
                .filter(|m| !matches!(m.role, Role::System))
                .map(message_to_anthropic)
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
        if let Some(tools) = &request.tools {
            body["tools"] = json!(tools
                .iter()
                .map(|t| json!({"name": t.name, "description": t.description, "input_schema": t.parameters}))
                .collect::<Vec<_>>());
        }

        if let Some(budget) = request
            .extra
            .get("thinking_budget_tokens")
            .and_then(Value::as_u64)
        {
            body["thinking"] = json!({"type":"enabled", "budget_tokens": budget});
        }

        if request
            .extra
            .get("prompt_cache")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            body["cache_control"] = json!({"type":"ephemeral"});
        }

        if let Some(map) = body.as_object_mut() {
            for (key, value) in &request.extra {
                if key == "thinking_budget_tokens" || key == "prompt_cache" {
                    continue;
                }
                map.insert(key.clone(), value.clone());
            }
        }

        body
    }

    async fn send_messages(&self, request: &CompletionRequest) -> Result<reqwest::Response> {
        let (_, value) = self
            .auth
            .header_for_provider("anthropic", &self.client)
            .await?;

        let response = self
            .client
            .post(&self.endpoint)
            .header(HeaderName::from_static("x-api-key"), value)
            .header(HeaderName::from_static("anthropic-version"), "2023-06-01")
            .header(CONTENT_TYPE, "application/json")
            .json(&self.build_body(request))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        let body = response.text().await.unwrap_or_default();
        if status == 429 {
            self.auth.rotate_key("anthropic").await;
        }

        Err(ProviderError::api(status, body, retry_after))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let response = self.send_messages(&request).await?;
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
                    "thinking" => {
                        if let Some(text) = block.get("thinking").and_then(Value::as_str) {
                            content.push(ContentBlock::Thinking {
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

        let usage = parse_usage(&raw);

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
        let response = self.send_messages(&request).await?;
        let body = response.text().await.unwrap_or_default();

        parse_anthropic_sse(&body)
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn supports_model(&self, model_id: &str) -> bool {
        model_id.starts_with("claude")
    }
}

fn parse_usage(raw: &Value) -> Usage {
    Usage {
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
    }
}

fn parse_anthropic_sse(raw: &str) -> Result<ProviderStream> {
    let mut out = Vec::new();
    let mut current_event = String::new();
    let mut current_data = String::new();

    for line in raw.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            if current_event.is_empty() || current_data.is_empty() {
                current_event.clear();
                current_data.clear();
                continue;
            }

            let parsed = serde_json::from_str::<Value>(&current_data).map_err(|err| {
                ProviderError::StreamProtocol(format!("invalid anthropic stream event JSON: {err}"))
            })?;

            match current_event.as_str() {
                "content_block_delta" => {
                    if let Some(text) = parsed
                        .get("delta")
                        .and_then(|v| v.get("text"))
                        .and_then(Value::as_str)
                    {
                        out.push(Ok(StreamChunk {
                            delta_content: vec![ContentBlock::Text {
                                text: text.to_string(),
                            }],
                            tool_calls: Vec::new(),
                            usage: None,
                            done: false,
                        }));
                    }
                }
                "content_block_start" => {
                    if let Some(block) = parsed.get("content_block") {
                        if block.get("type").and_then(Value::as_str) == Some("tool_use") {
                            let tool_call = ToolCall {
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
                            };
                            out.push(Ok(StreamChunk {
                                delta_content: Vec::new(),
                                tool_calls: vec![tool_call],
                                usage: None,
                                done: false,
                            }));
                        }
                    }
                }
                "message_delta" => {
                    let usage = parsed.get("usage").map(|usage| Usage {
                        input_tokens: usage
                            .get("input_tokens")
                            .and_then(Value::as_u64)
                            .and_then(|n| u32::try_from(n).ok())
                            .unwrap_or(0),
                        output_tokens: usage
                            .get("output_tokens")
                            .and_then(Value::as_u64)
                            .and_then(|n| u32::try_from(n).ok())
                            .unwrap_or(0),
                        cache_read: 0,
                        cache_write: 0,
                    });
                    out.push(Ok(StreamChunk {
                        delta_content: Vec::new(),
                        tool_calls: Vec::new(),
                        usage,
                        done: false,
                    }));
                }
                "message_stop" => {
                    out.push(Ok(StreamChunk {
                        delta_content: Vec::new(),
                        tool_calls: Vec::new(),
                        usage: None,
                        done: true,
                    }));
                }
                _ => {}
            }

            current_event.clear();
            current_data.clear();
            continue;
        }

        if let Some(value) = line.strip_prefix("event:") {
            current_event = value.trim().to_string();
        }
        if let Some(value) = line.strip_prefix("data:") {
            current_data = value.trim().to_string();
        }
    }

    if out.is_empty() {
        return Err(ProviderError::StreamProtocol(
            "no stream chunks parsed".to_string(),
        ));
    }

    Ok(out)
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
                ContentPart::Text { text } => Some(text.as_str()),
                ContentPart::ImageUrl { .. }
                | ContentPart::ImageBase64 { .. }
                | ContentPart::DocumentBase64 { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MessageContent::ToolUse(call) => call.arguments.to_string(),
        MessageContent::ToolResult(result) => result.content.clone(),
    }
}

fn multipart_content(content: &MessageContent) -> Vec<Value> {
    match content {
        MessageContent::Text(text) => vec![json!({"type":"text","text":text})],
        MessageContent::ToolResult(result) => vec![json!({
            "type": "tool_result",
            "tool_use_id": result.tool_call_id,
            "content": result.content,
        })],
        MessageContent::ToolUse(call) => vec![json!({
            "type": "tool_use",
            "id": call.id,
            "name": call.name,
            "input": call.arguments,
        })],
        MessageContent::MultiPart(parts) => parts
            .iter()
            .map(|part| match part {
                ContentPart::Text { text } => json!({"type":"text","text":text}),
                ContentPart::ImageUrl { url } => json!({
                    "type":"image",
                    "source": {"type":"url","url":url}
                }),
                ContentPart::ImageBase64 { mime_type, data } => json!({
                    "type":"image",
                    "source": {"type":"base64","media_type":mime_type,"data":data}
                }),
                ContentPart::DocumentBase64 {
                    mime_type,
                    data,
                    title,
                } => json!({
                    "type": "document",
                    "title": title,
                    "source": {"type":"base64","media_type":mime_type,"data":data}
                }),
            })
            .collect(),
    }
}

fn message_to_anthropic(message: &crate::types::Message) -> Value {
    json!({
        "role": role_name(message.role),
        "content": multipart_content(&message.content),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_system_top_level() {
        let request = CompletionRequest {
            model: "claude-sonnet-4-6".to_string(),
            messages: vec![
                crate::types::Message {
                    role: Role::System,
                    content: MessageContent::Text("you are concise".to_string()),
                },
                crate::types::Message {
                    role: Role::User,
                    content: MessageContent::Text("hello".to_string()),
                },
            ],
            tools: None,
            temperature: None,
            max_tokens: Some(1024),
            stream: false,
            extra: Default::default(),
        };

        let body = AnthropicProvider::new(AuthProfiles::default()).build_body(&request);
        assert_eq!(body["system"], json!("you are concise"));
        assert!(body["messages"].is_array());
    }

    #[test]
    fn parses_stream_events() {
        let raw = concat!(
            "event: content_block_delta\n",
            "data: {\"delta\": {\"text\": \"hi\"}}\n\n",
            "event: message_stop\n",
            "data: {}\n\n"
        );

        let chunks = parse_anthropic_sse(raw).expect("chunks");
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].as_ref().expect("chunk").delta_content.len() == 1);
        assert!(chunks[1].as_ref().expect("chunk").done);
    }

    #[test]
    fn maps_image_and_document_parts() {
        let message = crate::types::Message {
            role: Role::User,
            content: MessageContent::MultiPart(vec![
                ContentPart::ImageBase64 {
                    mime_type: "image/png".to_string(),
                    data: "abc".to_string(),
                },
                ContentPart::DocumentBase64 {
                    mime_type: "application/pdf".to_string(),
                    data: "def".to_string(),
                    title: Some("spec".to_string()),
                },
            ]),
        };

        let mapped = message_to_anthropic(&message);
        assert!(mapped.to_string().contains("base64"));
        assert!(mapped.to_string().contains("document"));
    }
}

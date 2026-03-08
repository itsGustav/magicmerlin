//! Google Gemini provider.

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, ContentPart,
    MessageContent, Role, StopReason, StreamChunk, ToolCall, Usage,
};

/// Google AI provider using `generateContent`.
#[derive(Clone, Debug)]
pub struct GoogleProvider {
    client: reqwest::Client,
    auth: AuthProfiles,
    endpoint_base: String,
}

impl GoogleProvider {
    /// Creates a new Google provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
            endpoint_base: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    /// Creates provider with a custom API base URL.
    pub fn with_endpoint_base(mut self, endpoint_base: impl Into<String>) -> Self {
        self.endpoint_base = endpoint_base.into();
        self
    }

    fn build_generate_body(&self, request: &CompletionRequest) -> Value {
        let mut body = json!({
            "contents": request.messages.iter().map(map_message).collect::<Vec<_>>()
        });

        if let Some(system) = request
            .messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| content_text(&m.content))
        {
            body["system_instruction"] = json!({
                "parts": [{"text": system}],
            });
        }

        if let Some(tools) = &request.tools {
            body["tools"] = json!([{
                "function_declarations": tools.iter().map(|t| {
                    json!({"name": t.name, "description": t.description, "parameters": t.parameters})
                }).collect::<Vec<_>>()
            }]);
        }

        if let Some(temperature) = request.temperature {
            body["generationConfig"] = json!({"temperature": temperature});
        }
        if let Some(max_tokens) = request.max_tokens {
            let mut generation = body
                .get("generationConfig")
                .cloned()
                .unwrap_or_else(|| json!({}));
            generation["maxOutputTokens"] = json!(max_tokens);
            body["generationConfig"] = generation;
        }

        if let Some(safety) = request.extra.get("safety_settings") {
            body["safetySettings"] = safety.clone();
        }

        if let Some(map) = body.as_object_mut() {
            for (key, value) in &request.extra {
                if key == "safety_settings" {
                    continue;
                }
                map.insert(key.clone(), value.clone());
            }
        }

        body
    }

    fn endpoint(&self, model: &str, stream: bool, api_key: &str) -> String {
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        format!(
            "{}/models/{}:{}?key={api_key}",
            self.endpoint_base, model, method
        )
    }

    async fn send_request(&self, request: &CompletionRequest, stream: bool) -> Result<reqwest::Response> {
        let (_, value) = self
            .auth
            .header_for_provider("google", &self.client)
            .await?;
        let api_key = value
            .to_str()
            .map_err(|err| ProviderError::OAuthRefresh {
                provider: "google".to_string(),
                message: err.to_string(),
            })?
            .trim_start_matches("Bearer ")
            .to_string();

        let endpoint = self.endpoint(&request.model, stream, &api_key);
        let response = self
            .client
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .json(&self.build_generate_body(request))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|raw| raw.parse::<u64>().ok());
        let body = response.text().await.unwrap_or_default();
        if status == 429 {
            self.auth.rotate_key("google").await;
        }
        Err(ProviderError::api(status, body, retry_after))
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let response = self.send_request(&request, false).await?;
        let raw = response.json::<Value>().await?;
        parse_generate_response(raw, request.model)
    }

    async fn complete_stream(&self, mut request: CompletionRequest) -> Result<ProviderStream> {
        request.stream = true;
        let response = self.send_request(&request, true).await?;
        let raw = response.text().await.unwrap_or_default();

        parse_stream_json_lines(&raw)
    }

    fn name(&self) -> &str {
        "google"
    }

    fn supports_model(&self, model_id: &str) -> bool {
        model_id.starts_with("gemini")
    }
}

fn parse_generate_response(raw: Value, model: String) -> Result<CompletionResponse> {
    let candidate = raw
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .cloned()
        .unwrap_or_else(|| json!({}));

    let mut content = Vec::new();
    let mut tool_calls = Vec::new();

    if let Some(parts) = candidate
        .get("content")
        .and_then(|c| c.get("parts"))
        .and_then(Value::as_array)
    {
        for part in parts {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                if !text.is_empty() {
                    content.push(ContentBlock::Text {
                        text: text.to_string(),
                    });
                }
            }
            if let Some(fc) = part.get("functionCall") {
                tool_calls.push(ToolCall {
                    id: format!(
                        "call_{}",
                        fc.get("name").and_then(Value::as_str).unwrap_or("tool")
                    ),
                    name: fc
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    arguments: fc.get("args").cloned().unwrap_or_else(|| json!({})),
                });
            }
        }
    }

    Ok(CompletionResponse {
        id: String::new(),
        model,
        content,
        tool_calls,
        usage: Usage {
            input_tokens: raw
                .get("usageMetadata")
                .and_then(|u| u.get("promptTokenCount"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or_else(|| approximate_tokens(&raw.to_string())),
            output_tokens: raw
                .get("usageMetadata")
                .and_then(|u| u.get("candidatesTokenCount"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0),
            cache_read: raw
                .get("usageMetadata")
                .and_then(|u| u.get("cachedContentTokenCount"))
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0),
            cache_write: 0,
        },
        stop_reason: match candidate
            .get("finishReason")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "MAX_TOKENS" => StopReason::MaxTokens,
            "STOP" => StopReason::EndTurn,
            _ => StopReason::Unknown,
        },
        estimated_cost_usd: None,
    })
}

fn parse_stream_json_lines(raw: &str) -> Result<ProviderStream> {
    let mut out = Vec::new();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let line = line.trim();
        if line == "[DONE]" {
            out.push(Ok(StreamChunk {
                delta_content: Vec::new(),
                tool_calls: Vec::new(),
                usage: None,
                done: true,
            }));
            continue;
        }

        let parsed = serde_json::from_str::<Value>(line).map_err(|err| {
            ProviderError::StreamProtocol(format!("invalid Google stream line: {err}"))
        })?;

        let response = parse_generate_response(parsed.clone(), String::new())?;
        out.push(Ok(StreamChunk {
            delta_content: response.content,
            tool_calls: response.tool_calls,
            usage: Some(response.usage),
            done: false,
        }));
    }

    if out.is_empty() {
        return Err(ProviderError::StreamProtocol(
            "empty Google stream".to_string(),
        ));
    }

    if let Some(last) = out.last_mut() {
        if let Ok(last) = last {
            last.done = true;
        }
    }

    Ok(out)
}

fn map_message(message: &crate::types::Message) -> Value {
    let role = match message.role {
        Role::Assistant => "model",
        Role::System => "user",
        Role::User | Role::Tool => "user",
    };

    json!({
        "role": role,
        "parts": map_parts(&message.content),
    })
}

fn map_parts(content: &MessageContent) -> Vec<Value> {
    match content {
        MessageContent::Text(text) => vec![json!({"text": text})],
        MessageContent::ToolUse(call) => vec![json!({
            "functionCall": {
                "name": call.name,
                "args": call.arguments,
            }
        })],
        MessageContent::ToolResult(result) => vec![json!({
            "functionResponse": {
                "name": "tool_result",
                "response": {
                    "tool_call_id": result.tool_call_id,
                    "content": result.content,
                }
            }
        })],
        MessageContent::MultiPart(parts) => parts
            .iter()
            .map(|part| match part {
                ContentPart::Text { text } => json!({"text": text}),
                ContentPart::ImageUrl { url } => json!({"fileData": {"uri": url}}),
                ContentPart::ImageBase64 { mime_type, data } => json!({
                    "inlineData": {"mimeType": mime_type, "data": data}
                }),
                ContentPart::DocumentBase64 {
                    mime_type,
                    data,
                    title,
                } => json!({
                    "inlineData": {"mimeType": mime_type, "data": data},
                    "title": title,
                }),
            })
            .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_body_includes_system_instruction() {
        let req = CompletionRequest {
            model: "gemini-2.5-pro".to_string(),
            messages: vec![
                crate::types::Message {
                    role: Role::System,
                    content: MessageContent::Text("system".to_string()),
                },
                crate::types::Message {
                    role: Role::User,
                    content: MessageContent::Text("user".to_string()),
                },
            ],
            tools: None,
            temperature: Some(0.1),
            max_tokens: Some(50),
            stream: false,
            extra: Default::default(),
        };

        let body = GoogleProvider::new(AuthProfiles::default()).build_generate_body(&req);
        assert!(body.get("system_instruction").is_some());
        assert!(body.get("generationConfig").is_some());
    }

    #[test]
    fn parse_response_extracts_tool_calls() {
        let raw = json!({
            "candidates": [{
                "content": {
                    "parts": [
                        {"text": "answer"},
                        {"functionCall": {"name": "lookup", "args": {"id": 1}}}
                    ]
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 12, "candidatesTokenCount": 34}
        });

        let parsed = parse_generate_response(raw, "gemini".to_string()).expect("parse");
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.content.len(), 1);
    }

    #[test]
    fn stream_json_lines_marks_done() {
        let raw = "{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"hi\"}]}}]}\n";
        let chunks = parse_stream_json_lines(raw).expect("chunks");
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].as_ref().expect("chunk").done);
    }
}

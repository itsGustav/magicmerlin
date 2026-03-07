//! Google Gemini provider.

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{
    approximate_tokens, CompletionRequest, CompletionResponse, ContentBlock, MessageContent,
    StopReason, StreamChunk, ToolCall, Usage,
};

/// Google AI provider using `generateContent`.
#[derive(Clone, Debug)]
pub struct GoogleProvider {
    client: reqwest::Client,
    auth: AuthProfiles,
}

impl GoogleProvider {
    /// Creates a new Google provider.
    pub fn new(auth: AuthProfiles) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
        }
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
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

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={api_key}",
            request.model
        );

        let mut body = json!({
            "contents": request.messages.iter().map(|m| {
                json!({
                    "role": if matches!(m.role, crate::types::Role::Assistant) { "model" } else { "user" },
                    "parts": [{"text": content_text(&m.content)}],
                })
            }).collect::<Vec<_>>()
        });

        if let Some(tools) = request.tools {
            body["tools"] = json!([{
                "function_declarations": tools.into_iter().map(|t| {
                    json!({"name": t.name, "description": t.description, "parameters": t.parameters})
                }).collect::<Vec<_>>()
            }]);
        }

        if let Some(map) = body.as_object_mut() {
            for (key, value) in request.extra {
                map.insert(key, value);
            }
        }

        let response = self
            .client
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_else(|_| String::new());
            if status == 429 {
                self.auth.rotate_key("google").await;
            }
            return Err(ProviderError::Api { status, body });
        }

        let raw = response.json::<Value>().await?;
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
                    content.push(ContentBlock::Text {
                        text: text.to_string(),
                    });
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
            model: request.model,
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
                cache_read: 0,
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
        "google"
    }

    fn supports_model(&self, model_id: &str) -> bool {
        model_id.starts_with("gemini")
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

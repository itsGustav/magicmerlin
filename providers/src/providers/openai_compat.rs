//! Shared implementation for OpenAI-compatible providers.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};

use crate::auth::AuthProfiles;
use crate::error::{ProviderError, Result};
use crate::providers::{LlmProvider, ProviderStream};
use crate::types::{
    approximate_tokens, parse_reasoning_effort, parse_response_format, CompletionRequest,
    CompletionResponse, ContentBlock, ContentPart, Message, MessageContent, ResponseFormatMode,
    Role, StopReason, StreamChunk, ToolCall, Usage,
};

/// OpenAI-compatible provider configuration.
#[derive(Clone, Debug)]
pub struct OpenAiCompatProvider {
    provider_name: String,
    base_url: String,
    auth_provider: String,
    auth_header_name: Option<String>,
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
            auth_header_name: None,
            client: reqwest::Client::new(),
            auth,
        }
    }

    /// Creates provider with custom auth header name override.
    pub fn with_auth_header_name(mut self, header_name: impl Into<String>) -> Self {
        self.auth_header_name = Some(header_name.into());
        self
    }

    /// Returns provider base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
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
            body["parallel_tool_calls"] = json!(true);
        }

        let response_format = parse_response_format(&request.extra);
        match response_format.mode {
            ResponseFormatMode::JsonObject => {
                body["response_format"] = json!({"type":"json_object"});
            }
            ResponseFormatMode::JsonSchema => {
                let name = response_format
                    .schema_name
                    .unwrap_or_else(|| "structured_output".to_string());
                let schema = response_format
                    .schema
                    .unwrap_or_else(|| json!({"type": "object"}));
                body["response_format"] = json!({
                    "type": "json_schema",
                    "json_schema": {
                        "name": name,
                        "strict": response_format.strict.unwrap_or(true),
                        "schema": schema,
                    }
                });
            }
            ResponseFormatMode::None => {}
        }

        if request.model.starts_with('o') {
            if let Some(reasoning_effort) = parse_reasoning_effort(&request.extra) {
                body["reasoning_effort"] = json!(reasoning_effort);
            }
        }

        if let Some(map) = body.as_object_mut() {
            for (key, value) in &request.extra {
                if key == "response_format" || key == "reasoning_effort" {
                    continue;
                }
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
        let (mut header_name, header_value) = self
            .auth
            .header_for_provider(&self.auth_provider, &self.client)
            .await?;

        if let Some(custom) = self.auth_header_name.as_deref() {
            header_name = reqwest::header::HeaderName::from_bytes(custom.as_bytes())
                .map_err(|_| ProviderError::InvalidRequest("invalid auth header".to_string()))?;
        }

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
        let retry_after = parse_retry_after(response.headers());
        let body = response.text().await.unwrap_or_default();
        if status == 429 {
            self.auth.rotate_key(&self.auth_provider).await;
        }
        Err(ProviderError::api(status, body, retry_after))
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
        let mut content = Vec::new();

        if let Some(text) = message.get("content").and_then(Value::as_str) {
            if !text.is_empty() {
                content.push(ContentBlock::Text {
                    text: text.to_string(),
                });
            }
        }

        if let Some(reasoning) = message
            .get("reasoning")
            .and_then(|v| v.get("summary"))
            .and_then(Value::as_str)
        {
            if !reasoning.is_empty() {
                content.push(ContentBlock::Thinking {
                    text: reasoning.to_string(),
                });
            }
        }

        let tool_calls = parse_openai_tool_calls(&message);
        let usage = parse_usage(raw.get("usage"));
        let stop_reason = parse_stop_reason(
            choice
                .get("finish_reason")
                .and_then(Value::as_str)
                .unwrap_or("stop"),
        );

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
            content,
            tool_calls,
            usage,
            stop_reason,
            estimated_cost_usd: None,
        })
    }

    async fn complete_stream(&self, mut request: CompletionRequest) -> Result<ProviderStream> {
        request.stream = true;
        let response = self.send_completion(&request).await?;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();

        let body = response.text().await.unwrap_or_default();
        if !content_type.contains("text/event-stream") {
            let raw = serde_json::from_str::<Value>(&body).unwrap_or_else(|_| json!({}));
            let message = raw
                .get("choices")
                .and_then(Value::as_array)
                .and_then(|v| v.first())
                .and_then(|c| c.get("message"))
                .cloned()
                .unwrap_or_else(|| json!({}));

            let chunk = StreamChunk {
                delta_content: message
                    .get("content")
                    .and_then(Value::as_str)
                    .map(|text| {
                        vec![ContentBlock::Text {
                            text: text.to_string(),
                        }]
                    })
                    .unwrap_or_default(),
                tool_calls: parse_openai_tool_calls(&message),
                usage: raw.get("usage").map(|u| parse_usage(Some(u))),
                done: true,
            };
            return Ok(vec![Ok(chunk)]);
        }

        parse_sse_stream_to_chunks(&body)
    }

    fn name(&self) -> &str {
        &self.provider_name
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }
}

fn parse_openai_tool_calls(message: &Value) -> Vec<ToolCall> {
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
    tool_calls
}

fn parse_stop_reason(reason: &str) -> StopReason {
    match reason {
        "tool_calls" | "function_call" => StopReason::ToolUse,
        "length" => StopReason::MaxTokens,
        "content_filter" => StopReason::ContentFilter,
        "stop" => StopReason::EndTurn,
        _ => StopReason::Unknown,
    }
}

fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.trim().parse::<u64>().ok())
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
                    ContentPart::ImageBase64 { mime_type, data } => {
                        json!({"type":"image_url","image_url":{"url":format!("data:{mime_type};base64,{data}")}})
                    }
                    ContentPart::DocumentBase64 {
                        mime_type,
                        data,
                        title,
                    } => json!({
                        "type": "input_file",
                        "input_file": {
                            "filename": title.clone().unwrap_or_else(|| "document.pdf".to_string()),
                            "mime_type": mime_type,
                            "data": data,
                        }
                    }),
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
            .get("prompt_tokens_details")
            .and_then(|v| v.get("cached_tokens"))
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .or_else(|| {
                usage
                    .get("cached_tokens")
                    .and_then(Value::as_u64)
                    .and_then(|n| u32::try_from(n).ok())
            })
            .unwrap_or(0),
        cache_write: usage
            .get("prompt_cache_write_tokens")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
    };

    if parsed.input_tokens == 0 && parsed.output_tokens == 0 {
        parsed.input_tokens = approximate_tokens(usage.to_string().as_str());
    }
    parsed
}

/// One SSE event.
#[derive(Debug, Clone, Default)]
struct SseEvent {
    event: Option<String>,
    data_lines: Vec<String>,
    retry: Option<u64>,
}

impl SseEvent {
    fn data_joined(&self) -> String {
        self.data_lines.join("\n")
    }
}

fn parse_sse_events(raw: &str) -> Vec<SseEvent> {
    let mut events = Vec::new();
    let mut current = SseEvent::default();

    for line in raw.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            if !current.data_lines.is_empty() || current.event.is_some() || current.retry.is_some()
            {
                events.push(current.clone());
                current = SseEvent::default();
            }
            continue;
        }

        if line.starts_with(':') {
            continue;
        }

        if let Some((field, value)) = line.split_once(':') {
            let value = value.trim_start().to_string();
            match field {
                "event" => current.event = Some(value),
                "data" => current.data_lines.push(value),
                "retry" => current.retry = value.parse::<u64>().ok(),
                _ => {}
            }
        }
    }

    if !current.data_lines.is_empty() || current.event.is_some() || current.retry.is_some() {
        events.push(current);
    }
    events
}

fn parse_sse_stream_to_chunks(raw: &str) -> Result<ProviderStream> {
    let events = parse_sse_events(raw);
    let mut out: ProviderStream = Vec::new();
    let mut tool_arg_buffer: HashMap<String, String> = HashMap::new();

    for event in events {
        let data = event.data_joined();
        if data.trim() == "[DONE]" {
            out.push(Ok(StreamChunk {
                delta_content: Vec::new(),
                tool_calls: Vec::new(),
                usage: None,
                done: true,
            }));
            continue;
        }

        let payload = serde_json::from_str::<Value>(&data)
            .map_err(|err| ProviderError::StreamProtocol(format!("invalid SSE JSON: {err}")))?;

        let mut delta_content = Vec::new();
        let mut tool_calls = Vec::new();
        let mut usage = None;

        if let Some(usage_obj) = payload.get("usage") {
            usage = Some(parse_usage(Some(usage_obj)));
        }

        if let Some(choice) = payload
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        {
            if let Some(delta) = choice.get("delta") {
                if let Some(text) = delta.get("content").and_then(Value::as_str) {
                    if !text.is_empty() {
                        delta_content.push(ContentBlock::Text {
                            text: text.to_string(),
                        });
                    }
                }

                if let Some(reasoning) = delta.get("reasoning_content").and_then(Value::as_str) {
                    if !reasoning.is_empty() {
                        delta_content.push(ContentBlock::Thinking {
                            text: reasoning.to_string(),
                        });
                    }
                }

                if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
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
                        let args_chunk = call
                            .get("function")
                            .and_then(|f| f.get("arguments"))
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();

                        let key = if id.is_empty() {
                            format!(
                                "{}:unknown",
                                call.get("index").and_then(Value::as_u64).unwrap_or(0)
                            )
                        } else {
                            id.clone()
                        };
                        let entry = tool_arg_buffer.entry(key.clone()).or_default();
                        entry.push_str(&args_chunk);
                        let parsed_args = serde_json::from_str(entry).unwrap_or_else(|_| json!({}));

                        tool_calls.push(ToolCall {
                            id: if id.is_empty() { key } else { id },
                            name,
                            arguments: parsed_args,
                        });
                    }
                }
            }

            if choice
                .get("finish_reason")
                .and_then(Value::as_str)
                .is_some()
            {
                out.push(Ok(StreamChunk {
                    delta_content,
                    tool_calls,
                    usage,
                    done: true,
                }));
                continue;
            }
        }

        if !delta_content.is_empty() || !tool_calls.is_empty() || usage.is_some() {
            out.push(Ok(StreamChunk {
                delta_content,
                tool_calls,
                usage,
                done: false,
            }));
        }
    }

    if out.is_empty() {
        out.push(Err(ProviderError::StreamProtocol(
            "empty stream payload".to_string(),
        )));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::types::{Message, MessageContent, Role, ToolDefinition};

    #[test]
    fn request_formatting_includes_tools_and_reasoning() {
        let mut extra = HashMap::new();
        extra.insert(
            "reasoning_effort".to_string(),
            Value::String("high".to_string()),
        );
        let request = CompletionRequest {
            model: "o3-mini".to_string(),
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
            extra,
        };

        let body = OpenAiCompatProvider::build_chat_body(&request);
        assert_eq!(body["model"], json!("o3-mini"));
        assert!(body["tools"].is_array());
        assert_eq!(body["temperature"], json!(0.2));
        assert_eq!(body["max_tokens"], json!(100));
        assert_eq!(body["reasoning_effort"], json!("high"));
        assert_eq!(body["parallel_tool_calls"], json!(true));
    }

    #[test]
    fn multipart_maps_base64_image_to_data_url() {
        let message = Message {
            role: Role::User,
            content: MessageContent::MultiPart(vec![
                ContentPart::Text {
                    text: "what is this".to_string(),
                },
                ContentPart::ImageBase64 {
                    mime_type: "image/png".to_string(),
                    data: "abcd==".to_string(),
                },
            ]),
        };

        let payload = message_to_json(&message);
        assert!(payload["content"].is_array());
        let content = payload["content"].as_array().expect("array");
        assert!(content
            .iter()
            .any(|part| part.to_string().contains("data:image/png;base64,abcd==")));
    }

    #[test]
    fn parses_sse_events_basic_fields() {
        let raw = "event: message\ndata: {\"x\":1}\nretry: 500\n\n";
        let events = parse_sse_events(raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("message"));
        assert_eq!(events[0].retry, Some(500));
        assert_eq!(events[0].data_joined(), "{\"x\":1}");
    }

    #[test]
    fn parses_sse_stream_with_tool_call_deltas() {
        let raw = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lo\",\"tool_calls\":[{\"id\":\"call_1\",\"function\":{\"name\":\"exec\",\"arguments\":\"{\\\"cmd\\\":\\\"ls\\\"}\"}}]}}]}\n\n",
            "data: [DONE]\n\n"
        );

        let chunks = parse_sse_stream_to_chunks(raw).expect("chunks");
        assert!(chunks.iter().any(|c| {
            c.as_ref()
                .ok()
                .map(|chunk| {
                    chunk
                        .delta_content
                        .iter()
                        .any(|block| matches!(block, ContentBlock::Text { text } if text == "Hel"))
                })
                .unwrap_or(false)
        }));

        assert!(chunks.iter().any(|c| {
            c.as_ref()
                .ok()
                .map(|chunk| !chunk.tool_calls.is_empty())
                .unwrap_or(false)
        }));

        assert!(chunks
            .iter()
            .any(|c| { c.as_ref().ok().map(|chunk| chunk.done).unwrap_or(false) }));
    }

    #[test]
    fn parse_retry_after_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "retry-after",
            reqwest::header::HeaderValue::from_static("7"),
        );
        assert_eq!(parse_retry_after(&headers), Some(7));
    }
}

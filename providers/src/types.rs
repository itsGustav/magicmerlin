//! Request/response types used across model providers.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A normalized completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier or alias.
    pub model: String,
    /// Chat message history.
    pub messages: Vec<Message>,
    /// Optional tool declarations.
    pub tools: Option<Vec<ToolDefinition>>,
    /// Optional sampling temperature.
    pub temperature: Option<f64>,
    /// Optional output token cap.
    pub max_tokens: Option<u32>,
    /// Whether streaming is requested.
    pub stream: bool,
    /// Provider-specific request extensions.
    #[serde(default)]
    pub extra: HashMap<String, Value>,
}

/// One chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message author role.
    pub role: Role,
    /// Message payload.
    pub content: MessageContent,
}

/// Role for a chat message.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System instruction.
    System,
    /// Human input.
    User,
    /// Assistant output.
    Assistant,
    /// Tool result message.
    Tool,
}

/// Structured content for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MessageContent {
    /// Plain text content.
    Text(String),
    /// Multipart content blocks.
    MultiPart(Vec<ContentPart>),
    /// Tool invocation payload.
    ToolUse(ToolCall),
    /// Tool result payload.
    ToolResult(ToolResultContent),
}

/// One message multipart item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text segment.
    Text { text: String },
    /// Image URL segment.
    ImageUrl { url: String },
    /// Base64 image segment.
    ImageBase64 {
        /// MIME type, for example `image/png`.
        mime_type: String,
        /// Base64 payload (without data URL prefix).
        data: String,
    },
    /// PDF/document payload.
    DocumentBase64 {
        /// MIME type, for example `application/pdf`.
        mime_type: String,
        /// Base64 payload.
        data: String,
        /// Optional display title.
        title: Option<String>,
    },
}

/// Tool declaration with JSON schema parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool function name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON-schema params.
    pub parameters: Value,
}

/// Tool call emitted by model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call id.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// JSON args.
    pub arguments: Value,
}

/// Tool call result content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultContent {
    /// Tool call id.
    pub tool_call_id: String,
    /// Raw result content.
    pub content: String,
}

/// Response content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text output.
    Text { text: String },
    /// Non-text output.
    Json { value: Value },
    /// Reasoning/thinking output emitted by some providers.
    Thinking { text: String },
}

/// Token/caching usage counters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Input token count.
    pub input_tokens: u32,
    /// Output token count.
    pub output_tokens: u32,
    /// Cache-read tokens.
    pub cache_read: u32,
    /// Cache-write tokens.
    pub cache_write: u32,
}

/// Stop reason for generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Completion finished normally.
    EndTurn,
    /// Completion requested tool use.
    ToolUse,
    /// Completion reached max token cap.
    MaxTokens,
    /// Provider content filter.
    ContentFilter,
    /// Explicit stop sequence.
    StopSequence,
    /// Provider error state.
    Error,
    /// Unknown provider-specific reason.
    Unknown,
}

/// Normalized non-stream response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Provider response ID.
    pub id: String,
    /// Fully qualified model name.
    pub model: String,
    /// Output content blocks.
    pub content: Vec<ContentBlock>,
    /// Tool calls emitted by model.
    pub tool_calls: Vec<ToolCall>,
    /// Token/caching usage counters.
    pub usage: Usage,
    /// Stop reason.
    pub stop_reason: StopReason,
    /// Computed request cost in USD.
    pub estimated_cost_usd: Option<f64>,
}

/// One stream chunk emitted by providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Delta content blocks.
    pub delta_content: Vec<ContentBlock>,
    /// Delta tool calls.
    pub tool_calls: Vec<ToolCall>,
    /// Optional usage snapshot.
    pub usage: Option<Usage>,
    /// Whether this chunk ends the stream.
    pub done: bool,
}

/// Supported response-format modes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatMode {
    /// No explicit response format constraints.
    None,
    /// Strict JSON object response.
    JsonObject,
    /// JSON Schema-constrained response.
    JsonSchema,
}

/// Parsed response format from request extras.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseFormat {
    /// Mode for provider mapping.
    pub mode: ResponseFormatMode,
    /// Optional schema object for `json_schema` mode.
    pub schema: Option<Value>,
    /// Optional schema name.
    pub schema_name: Option<String>,
    /// Optional strict toggle.
    pub strict: Option<bool>,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self {
            mode: ResponseFormatMode::None,
            schema: None,
            schema_name: None,
            strict: None,
        }
    }
}

/// Attempts to parse response-format related fields from a request `extra` map.
pub fn parse_response_format(extra: &HashMap<String, Value>) -> ResponseFormat {
    let mut format = ResponseFormat::default();
    let Some(value) = extra.get("response_format") else {
        return format;
    };

    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if kind == "json_object" {
        format.mode = ResponseFormatMode::JsonObject;
        return format;
    }

    if kind != "json_schema" {
        return format;
    }

    format.mode = ResponseFormatMode::JsonSchema;
    let schema_wrapper = value.get("json_schema");
    format.schema = schema_wrapper
        .and_then(|v| v.get("schema"))
        .cloned()
        .or_else(|| value.get("schema").cloned());
    format.schema_name = schema_wrapper
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            value
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
    format.strict = schema_wrapper
        .and_then(|v| v.get("strict"))
        .and_then(Value::as_bool)
        .or_else(|| value.get("strict").and_then(Value::as_bool));

    format
}

/// Extracts optional reasoning effort for OpenAI reasoning models from request extras.
pub fn parse_reasoning_effort(extra: &HashMap<String, Value>) -> Option<String> {
    extra
        .get("reasoning_effort")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Returns approximate token count from plain text using chars/4 heuristic.
pub fn approximate_tokens(text: &str) -> u32 {
    ((text.chars().count() as f64) / 4.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_schema_response_format() {
        let mut extra = HashMap::new();
        extra.insert(
            "response_format".to_string(),
            serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "structured_output",
                    "strict": true,
                    "schema": {"type": "object", "properties": {"answer": {"type": "string"}}}
                }
            }),
        );

        let parsed = parse_response_format(&extra);
        assert_eq!(parsed.mode, ResponseFormatMode::JsonSchema);
        assert_eq!(parsed.schema_name.as_deref(), Some("structured_output"));
        assert_eq!(parsed.strict, Some(true));
        assert!(parsed.schema.is_some());
    }

    #[test]
    fn parses_reasoning_effort() {
        let mut extra = HashMap::new();
        extra.insert("reasoning_effort".to_string(), Value::String("high".to_string()));
        assert_eq!(parse_reasoning_effort(&extra).as_deref(), Some("high"));
    }
}

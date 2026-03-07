//! Tool trait, context, and registry.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::{Result, ToolError};

/// Delivery context for channel-specific message dispatch.
#[derive(Debug, Clone)]
pub struct DeliveryContext {
    /// Delivery channel name.
    pub channel: String,
    /// Channel target identifier.
    pub target: String,
}

/// Tool execution context.
#[derive(Clone)]
pub struct ToolContext {
    /// Agent identity.
    pub agent_name: String,
    /// Workspace root path.
    pub workspace_dir: PathBuf,
    /// State path helpers.
    pub state_paths: magicmerlin_config::StatePaths,
    /// Runtime config snapshot.
    pub config: magicmerlin_config::Config,
    /// Optional delivery metadata.
    pub delivery: Option<DeliveryContext>,
    /// Shared process manager.
    pub process_manager: crate::ProcessManager,
}

/// Tool execution result wrapper.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolResult {
    /// Whether execution succeeded.
    pub ok: bool,
    /// Structured output payload.
    pub value: Value,
    /// Whether output was truncated.
    pub truncated: bool,
}

impl ToolResult {
    /// Creates success result.
    pub fn success(value: Value) -> Self {
        Self {
            ok: true,
            value,
            truncated: false,
        }
    }

    /// Creates error result.
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            value: json!({"error": message.into()}),
            truncated: false,
        }
    }
}

/// Tool contract exposed to agent runtime.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name.
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &str;

    /// JSON schema describing expected params.
    fn schema(&self) -> Value;

    /// Executes the tool.
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
}

/// Registry of tools with permissions and output-size controls.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    deny_list: HashSet<String>,
    /// Max serialized output bytes before truncation.
    pub max_result_bytes: usize,
}

impl ToolRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            deny_list: HashSet::new(),
            max_result_bytes: 64 * 1024,
        }
    }

    /// Registers one tool.
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Denies a tool by name.
    pub fn deny_tool(&mut self, name: &str) {
        self.deny_list.insert(name.to_string());
    }

    /// Returns JSON schema catalog for all tools.
    pub fn schemas(&self) -> Vec<Value> {
        let mut list = self
            .tools
            .values()
            .map(|tool| {
                json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.schema(),
                })
            })
            .collect::<Vec<_>>();
        list.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        list
    }

    /// Executes one tool call by name.
    pub async fn execute(
        &self,
        name: &str,
        params: Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        if self.deny_list.contains(name) {
            return Err(ToolError::PermissionDenied(format!("tool denied: {name}")));
        }

        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::UnknownTool(name.to_string()))?;
        let mut result = tool.execute(params, ctx).await?;

        let serialized = serde_json::to_vec(&result.value)?;
        if serialized.len() > self.max_result_bytes {
            let clipped = String::from_utf8_lossy(&serialized)
                .chars()
                .take(self.max_result_bytes)
                .collect::<String>();
            result.value = json!({"truncated": clipped});
            result.truncated = true;
        }

        Ok(result)
    }

    /// Returns list of registered tool names.
    pub fn names(&self) -> Vec<String> {
        let mut names = self.tools.keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }
}

//! Default tool implementations and registration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use rusqlite::params;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::{Result, ToolError};
use crate::registry::{Tool, ToolContext, ToolRegistry, ToolResult};

/// Registers all default tools (implemented and stubbed).
pub fn register_default_tools(registry: &mut ToolRegistry) {
    registry.register(Arc::new(ExecTool));
    registry.register(Arc::new(ProcessTool));
    registry.register(Arc::new(ReadTool));
    registry.register(Arc::new(WriteTool));
    registry.register(Arc::new(EditTool));
    registry.register(Arc::new(WebSearchTool));
    registry.register(Arc::new(WebFetchTool));
    registry.register(Arc::new(MemorySearchTool));
    registry.register(Arc::new(MemoryGetTool));
    registry.register(Arc::new(SessionStatusTool));
    registry.register(Arc::new(SessionsListTool));
    registry.register(Arc::new(SessionsHistoryTool));

    for name in [
        "sessions_send",
        "sessions_spawn",
        "subagents",
        "agents_list",
        "message",
        "image",
        "pdf",
        "tts",
        "browser",
        "canvas",
        "nodes",
    ] {
        registry.register(Arc::new(StubTool::new(name)));
    }
}

struct ExecTool;

#[async_trait]
impl Tool for ExecTool {
    fn name(&self) -> &str {
        "exec"
    }

    fn description(&self) -> &str {
        "Executes a shell command with optional timeout and background mode."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "cmd": {"type":"string"},
                "cwd": {"type":"string"},
                "timeout_ms": {"type":"integer"},
                "background": {"type":"boolean"},
                "env": {"type":"object", "additionalProperties": {"type":"string"}}
            },
            "required": ["cmd"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let cmd = required_string(&params, "cmd", self.name())?;
        let cwd = params
            .get("cwd")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_else(|| ctx.workspace_dir.clone());
        enforce_workspace_path(&ctx.workspace_dir, &cwd)?;

        let timeout_ms = params
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .unwrap_or(120_000);
        let background = params
            .get("background")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let env = parse_env(params.get("env"));

        if background {
            let id = ctx.process_manager.spawn(&cmd, Some(&cwd), &env).await?;
            return Ok(ToolResult::success(json!({"session_id": id})));
        }

        let mut command = shell_command(&cmd);
        command.current_dir(&cwd).envs(&env);

        let output = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            command.output(),
        )
        .await
        .map_err(|_| ToolError::Execution("command timed out".to_string()))?
        .map_err(|err| ToolError::Execution(err.to_string()))?;

        Ok(ToolResult::success(json!({
            "status": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
        })))
    }
}

struct ProcessTool;

#[async_trait]
impl Tool for ProcessTool {
    fn name(&self) -> &str {
        "process"
    }

    fn description(&self) -> &str {
        "Manages background process sessions."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {"type": "string"},
                "session_id": {"type": "integer"},
                "offset": {"type": "integer"},
                "limit": {"type": "integer"},
                "text": {"type": "string"}
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        match action.as_str() {
            "list" => Ok(ToolResult::success(
                json!({"processes": ctx.process_manager.list().await}),
            )),
            "poll" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let summary = ctx.process_manager.poll(id).await?;
                Ok(ToolResult::success(json!({"process": summary})))
            }
            "log" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
                let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(4000) as usize;
                let log = ctx.process_manager.log(id, offset, limit).await?;
                Ok(ToolResult::success(json!({"log": log})))
            }
            "write" | "submit" | "send-keys" | "paste" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let text = required_string(&params, "text", self.name())?;
                ctx.process_manager.write(id, &text).await?;
                Ok(ToolResult::success(json!({"ok": true})))
            }
            "kill" => {
                let id = required_u64(&params, "session_id", self.name())?;
                ctx.process_manager.kill(id).await?;
                Ok(ToolResult::success(json!({"ok": true})))
            }
            other => Err(ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: format!("unsupported action: {other}"),
            }),
        }
    }
}

struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Reads file content with optional offset and limit."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "path":{"type":"string"},
                "offset":{"type":"integer"},
                "limit":{"type":"integer"}
            },
            "required":["path"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path = resolve_workspace_path(
            &ctx.workspace_dir,
            required_string(&params, "path", self.name())?,
        )?;
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
        let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
        let limit = params
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(32_000) as usize;
        let slice = bytes
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();

        if is_image_path(&path) {
            return Ok(ToolResult::success(json!({
                "path": path,
                "kind": "image",
                "bytes": slice.len()
            })));
        }

        Ok(ToolResult::success(json!({
            "path": path,
            "text": String::from_utf8_lossy(&slice).to_string(),
        })))
    }
}

struct WriteTool;

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Writes text content to file, creating parent directories."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "path":{"type":"string"},
                "content":{"type":"string"}
            },
            "required":["path","content"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path = resolve_workspace_path(
            &ctx.workspace_dir,
            required_string(&params, "path", self.name())?,
        )?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|source| ToolError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }
        let content = required_string(&params, "content", self.name())?;
        tokio::fs::write(&path, content)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
        Ok(ToolResult::success(json!({"ok": true, "path": path})))
    }
}

struct EditTool;

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Replaces exact text in a file."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "path":{"type":"string"},
                "oldText":{"type":"string"},
                "newText":{"type":"string"}
            },
            "required":["path","oldText","newText"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path = resolve_workspace_path(
            &ctx.workspace_dir,
            required_string(&params, "path", self.name())?,
        )?;
        let old_text = required_string(&params, "oldText", self.name())?;
        let new_text = required_string(&params, "newText", self.name())?;

        let body = tokio::fs::read_to_string(&path)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
        if !body.contains(&old_text) {
            return Err(ToolError::Execution("oldText not found".to_string()));
        }
        let updated = body.replacen(&old_text, &new_text, 1);
        tokio::fs::write(&path, updated)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;

        Ok(ToolResult::success(json!({"ok": true, "path": path})))
    }
}

struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Runs Brave Search API query."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "query":{"type":"string"},
                "count":{"type":"integer"},
                "freshness":{"type":"string"},
                "country":{"type":"string"},
                "language":{"type":"string"}
            },
            "required":["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let query = required_string(&params, "query", self.name())?;
        let count = params.get("count").and_then(Value::as_u64).unwrap_or(5);
        let api_key = ctx
            .config
            .tools
            .values
            .get("brave_api_key")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolError::Execution("missing tools.brave_api_key config".to_string())
            })?;

        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", api_key)
            .query(&[("q", query.as_str()), ("count", &count.to_string())])
            .send()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let status = resp.status().as_u16();
        let value = resp
            .json::<Value>()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        Ok(ToolResult::success(
            json!({"status": status, "result": value}),
        ))
    }
}

struct WebFetchTool;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetches URL and returns plain text body."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {"url":{"type":"string"}},
            "required":["url"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let url = required_string(&params, "url", self.name())?;
        let response = reqwest::get(&url)
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        Ok(ToolResult::success(json!({"status": status, "text": body})))
    }
}

struct MemorySearchTool;

#[async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        "Searches MEMORY.md and daily memory files for query text."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {"query":{"type":"string"}},
            "required":["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let query = required_string(&params, "query", self.name())?.to_lowercase();
        let root = ctx.state_paths.state_dir.clone();
        let mut matches = Vec::new();

        let files = collect_memory_files(&root)?;
        for path in files {
            let body = std::fs::read_to_string(&path).map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
            for (idx, line) in body.lines().enumerate() {
                if line.to_lowercase().contains(&query) {
                    matches.push(json!({
                        "path": path,
                        "line": idx + 1,
                        "text": line,
                    }));
                }
            }
        }

        Ok(ToolResult::success(json!({"matches": matches})))
    }
}

struct MemoryGetTool;

#[async_trait]
impl Tool for MemoryGetTool {
    fn name(&self) -> &str {
        "memory_get"
    }

    fn description(&self) -> &str {
        "Reads line range from memory file."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "path":{"type":"string"},
                "start_line":{"type":"integer"},
                "end_line":{"type":"integer"}
            },
            "required":["path"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path_value = required_string(&params, "path", self.name())?;
        let path = if Path::new(&path_value).is_absolute() {
            PathBuf::from(&path_value)
        } else {
            ctx.state_paths.state_dir.join(path_value)
        };

        let body = std::fs::read_to_string(&path).map_err(|source| ToolError::Io {
            path: path.clone(),
            source,
        })?;
        let start = params
            .get("start_line")
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        let end = params
            .get("end_line")
            .and_then(Value::as_u64)
            .unwrap_or(start as u64 + 50) as usize;

        let lines = body
            .lines()
            .enumerate()
            .filter(|(idx, _)| {
                let line_no = idx + 1;
                line_no >= start && line_no <= end
            })
            .map(|(idx, line)| format!("{}:{}", idx + 1, line))
            .collect::<Vec<_>>();

        Ok(ToolResult::success(
            json!({"path": path, "snippet": lines.join("\n")}),
        ))
    }
}

struct SessionStatusTool;

#[async_trait]
impl Tool for SessionStatusTool {
    fn name(&self) -> &str {
        "session_status"
    }

    fn description(&self) -> &str {
        "Returns session status with token/cost fields."
    }

    fn schema(&self) -> Value {
        json!({"type":"object","properties":{"session_id":{"type":"string"}},"required":["session_id"]})
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let session_id = required_string(&params, "session_id", self.name())?;
        let conn =
            magicmerlin_storage::Storage::new(ctx.state_paths.state_dir.join("openclaw.db"))?
                .connection()?;
        let row = conn.query_row(
            "SELECT id, agent, status, started_at, updated_at FROM sessions WHERE id=?1",
            params![session_id],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "agent": row.get::<_, Option<String>>(1)?,
                    "status": row.get::<_, String>(2)?,
                    "started_at": row.get::<_, i64>(3)?,
                    "updated_at": row.get::<_, i64>(4)?,
                }))
            },
        );
        let status = match row {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => json!({"missing": true}),
            Err(err) => return Err(ToolError::Execution(err.to_string())),
        };

        Ok(ToolResult::success(json!({
            "session": status,
            "model": ctx.config.agents.defaults.model,
            "tokens": null,
            "cost_usd": null,
            "context_pct": null
        })))
    }
}

struct SessionsListTool;

#[async_trait]
impl Tool for SessionsListTool {
    fn name(&self) -> &str {
        "sessions_list"
    }

    fn description(&self) -> &str {
        "Lists known sessions from sqlite."
    }

    fn schema(&self) -> Value {
        json!({"type":"object","properties":{"agent":{"type":"string"}}})
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let filter_agent = params
            .get("agent")
            .and_then(Value::as_str)
            .map(str::to_string);
        let storage =
            magicmerlin_storage::Storage::new(ctx.state_paths.state_dir.join("openclaw.db"))?;
        let conn = storage.connection()?;

        let mut query =
            "SELECT id, agent, status, started_at, updated_at FROM sessions".to_string();
        if filter_agent.is_some() {
            query.push_str(" WHERE agent = ?1");
        }
        query.push_str(" ORDER BY updated_at DESC LIMIT 100");

        let mut out = Vec::new();
        if let Some(agent) = filter_agent {
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            let rows = stmt
                .query_map(params![agent], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>(0)?,
                        "agent": row.get::<_, Option<String>>(1)?,
                        "status": row.get::<_, String>(2)?,
                        "started_at": row.get::<_, i64>(3)?,
                        "updated_at": row.get::<_, i64>(4)?,
                    }))
                })
                .map_err(|e| ToolError::Execution(e.to_string()))?;

            for row in rows {
                out.push(row.map_err(|e| ToolError::Execution(e.to_string()))?);
            }
        } else {
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>(0)?,
                        "agent": row.get::<_, Option<String>>(1)?,
                        "status": row.get::<_, String>(2)?,
                        "started_at": row.get::<_, i64>(3)?,
                        "updated_at": row.get::<_, i64>(4)?,
                    }))
                })
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            for row in rows {
                out.push(row.map_err(|e| ToolError::Execution(e.to_string()))?);
            }
        }

        Ok(ToolResult::success(json!({"sessions": out})))
    }
}

struct SessionsHistoryTool;

#[async_trait]
impl Tool for SessionsHistoryTool {
    fn name(&self) -> &str {
        "sessions_history"
    }

    fn description(&self) -> &str {
        "Returns transcript history for a session."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "agent":{"type":"string"},
                "session_id":{"type":"string"},
                "offset":{"type":"integer"},
                "limit":{"type":"integer"}
            },
            "required":["agent","session_id"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let agent = required_string(&params, "agent", self.name())?;
        let session_id = required_string(&params, "session_id", self.name())?;
        let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
        let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize;

        let transcript_path = ctx
            .state_paths
            .sessions_dir
            .join(agent)
            .join(format!("{}.jsonl", session_id.replace(':', "__")));
        let store = magicmerlin_storage::TranscriptStore::new(transcript_path)?;
        let entries = store.read(offset, Some(limit))?;
        Ok(ToolResult::success(json!({"entries": entries})))
    }
}

struct StubTool {
    name: &'static str,
}

impl StubTool {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait]
impl Tool for StubTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Reserved tool surface."
    }

    fn schema(&self) -> Value {
        json!({"type":"object"})
    }

    async fn execute(&self, _params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        Ok(ToolResult::failure(format!(
            "tool {} is not implemented in this phase",
            self.name
        )))
    }
}

fn required_string(params: &Value, key: &str, tool: &str) -> Result<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ToolError::InvalidParams {
            tool: tool.to_string(),
            message: format!("missing string field `{key}`"),
        })
}

fn required_u64(params: &Value, key: &str, tool: &str) -> Result<u64> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| ToolError::InvalidParams {
            tool: tool.to_string(),
            message: format!("missing integer field `{key}`"),
        })
}

fn parse_env(value: Option<&Value>) -> HashMap<String, String> {
    value
        .and_then(Value::as_object)
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}

fn resolve_workspace_path(workspace: &Path, requested: String) -> Result<PathBuf> {
    let path = if Path::new(&requested).is_absolute() {
        PathBuf::from(requested)
    } else {
        workspace.join(requested)
    };
    enforce_workspace_path(workspace, &path)?;
    Ok(path)
}

fn enforce_workspace_path(workspace: &Path, path: &Path) -> Result<()> {
    let workspace = workspace.canonicalize().map_err(|source| ToolError::Io {
        path: workspace.to_path_buf(),
        source,
    })?;

    let canonical = if path.exists() {
        path.canonicalize().map_err(|source| ToolError::Io {
            path: path.to_path_buf(),
            source,
        })?
    } else {
        let parent = path
            .parent()
            .ok_or_else(|| ToolError::PermissionDenied("invalid path".to_string()))?;
        let parent_canonical = parent.canonicalize().map_err(|source| ToolError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        parent_canonical.join(path.file_name().unwrap_or_default())
    };

    if !canonical.starts_with(&workspace) {
        return Err(ToolError::PermissionDenied(format!(
            "path outside workspace: {}",
            path.display()
        )));
    }
    Ok(())
}

fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|x| x.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ext) if ["png", "jpg", "jpeg", "gif", "webp", "bmp"].contains(&ext.as_str())
    )
}

fn collect_memory_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let top = root.join("MEMORY.md");
    if top.exists() {
        files.push(top);
    }

    let memory_dir = root.join("memory");
    if memory_dir.exists() {
        for entry in std::fs::read_dir(&memory_dir).map_err(|source| ToolError::Io {
            path: memory_dir.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| ToolError::Io {
                path: memory_dir.clone(),
                source,
            })?;
            if entry.path().is_file() {
                files.push(entry.path());
            }
        }
    }
    Ok(files)
}

fn shell_command(command: &str) -> Command {
    #[cfg(target_family = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command);
        cmd
    }
    #[cfg(not(target_family = "windows"))]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-lc").arg(command);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn edit_replaces_text() {
        let temp = tempfile::tempdir().expect("tmp");
        let path = temp.path().join("a.txt");
        std::fs::write(&path, "hello old").expect("write");

        let state_paths = magicmerlin_config::StatePaths::new(magicmerlin_config::PathScope::dev())
            .expect("paths");
        let ctx = ToolContext {
            agent_name: "merlin".to_string(),
            workspace_dir: temp.path().to_path_buf(),
            state_paths,
            config: magicmerlin_config::Config::default(),
            delivery: None,
            process_manager: crate::ProcessManager::new(),
        };

        let tool = EditTool;
        tool.execute(
            json!({"path":"a.txt","oldText":"old","newText":"new"}),
            &ctx,
        )
        .await
        .expect("exec");

        let body = std::fs::read_to_string(path).expect("read");
        assert_eq!(body, "hello new");
    }

    #[tokio::test]
    async fn registry_contains_required_tools() {
        let mut registry = ToolRegistry::new();
        register_default_tools(&mut registry);
        let names = registry.names();
        for required in ["exec", "process", "read", "write", "edit", "memory_get"] {
            assert!(names.contains(&required.to_string()));
        }
    }

    #[test]
    fn collects_memory_files() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(temp.path().join("MEMORY.md"), "x").expect("write");
        std::fs::create_dir_all(temp.path().join("memory")).expect("mkdir");
        std::fs::write(temp.path().join("memory/2026-03-06.md"), "x").expect("write");

        let files = collect_memory_files(temp.path()).expect("collect");
        assert_eq!(files.len(), 2);
    }
}

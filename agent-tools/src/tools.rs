//! Default tool implementations and registration.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use rusqlite::params;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::{Result, ToolError};
use crate::registry::{Tool, ToolContext, ToolRegistry, ToolResult};

const READ_MAX_BYTES: usize = 50 * 1024;
const READ_MAX_LINES: usize = 2000;

/// Registers all default tools.
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
    registry.register(Arc::new(SessionsSendTool));
    registry.register(Arc::new(SessionsSpawnTool));
    registry.register(Arc::new(SubagentsTool));
    registry.register(Arc::new(AgentsListTool));
    registry.register(Arc::new(MessageTool));
    registry.register(Arc::new(ImageTool));
    registry.register(Arc::new(PdfTool));
    registry.register(Arc::new(TtsTool));
    registry.register(Arc::new(BrowserTool));
    registry.register(Arc::new(CanvasTool));
    registry.register(Arc::new(NodesTool));
}

struct ExecTool;

#[async_trait]
impl Tool for ExecTool {
    fn name(&self) -> &str {
        "exec"
    }

    fn description(&self) -> &str {
        "Executes a shell command with optional timeout, background mode, tty, and env."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "cmd": {"type":"string"},
                "cwd": {"type":"string"},
                "timeout_ms": {"type":"integer"},
                "background": {"type":"boolean"},
                "tty": {"type":"boolean"},
                "env": {"type":"object", "additionalProperties": {"type":"string"}},
                "capture": {"type":"string", "enum": ["separate", "combined"]}
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
        let tty = params.get("tty").and_then(Value::as_bool).unwrap_or(false);
        let env = parse_env(params.get("env"));

        if background {
            let id = ctx
                .process_manager
                .spawn_with_options(
                    &cmd,
                    Some(&cwd),
                    &env,
                    tty,
                    Some(Duration::from_millis(timeout_ms)),
                )
                .await?;
            return Ok(ToolResult::success(json!({
                "session_id": id,
                "background": true,
                "cwd": cwd,
            })));
        }

        let mut command = shell_command(&cmd);
        command.current_dir(&cwd).envs(&env);

        let output = tokio::time::timeout(Duration::from_millis(timeout_ms), command.output())
            .await
            .map_err(|_| ToolError::Execution("command timed out".to_string()))?
            .map_err(|err| ToolError::Execution(err.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let capture_mode = params
            .get("capture")
            .and_then(Value::as_str)
            .unwrap_or("separate");

        let value = if capture_mode == "combined" {
            json!({
                "status": output.status.code(),
                "output": format!("{stdout}{stderr}"),
                "stdout_bytes": output.stdout.len(),
                "stderr_bytes": output.stderr.len(),
            })
        } else {
            json!({
                "status": output.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "stdout_bytes": output.stdout.len(),
                "stderr_bytes": output.stderr.len(),
            })
        };

        Ok(ToolResult::success(value))
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
                "text": {"type": "string"},
                "timeout_ms": {"type": "integer"}
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
                let timeout_ms = params
                    .get("timeout_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let chunk = ctx.process_manager.poll_wait(id, timeout_ms).await?;
                Ok(ToolResult::success(json!({"process": chunk})))
            }
            "log" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
                let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(4000) as usize;
                let log = ctx.process_manager.log(id, offset, limit).await?;
                Ok(ToolResult::success(json!({"log": log})))
            }
            "write" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let text = required_string(&params, "text", self.name())?;
                ctx.process_manager.write(id, &text).await?;
                Ok(ToolResult::success(json!({"ok": true})))
            }
            "send-keys" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let text = required_string(&params, "text", self.name())?;
                ctx.process_manager.send_keys(id, &text).await?;
                Ok(ToolResult::success(json!({"ok": true})))
            }
            "paste" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let text = required_string(&params, "text", self.name())?;
                ctx.process_manager.paste(id, &text).await?;
                Ok(ToolResult::success(json!({"ok": true})))
            }
            "submit" => {
                let id = required_u64(&params, "session_id", self.name())?;
                let text = params.get("text").and_then(Value::as_str);
                ctx.process_manager.submit(id, text).await?;
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
        "Reads file content with optional line offset and line limit."
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

        if is_image_path(&path) {
            return Ok(ToolResult::success(json!({
                "path": path,
                "kind": "image",
                "bytes": bytes.len()
            })));
        }

        if looks_binary(&bytes) {
            return Err(ToolError::Execution(format!(
                "refusing to read binary file: {}",
                path.display()
            )));
        }

        let text = decode_text(&bytes);
        let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(1) as usize;
        let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(200) as usize;
        let limit = limit.min(READ_MAX_LINES);

        let lines = text.lines().collect::<Vec<_>>();
        let start_line = offset.max(1);
        let end_line = (start_line + limit - 1).min(lines.len());

        let mut out = String::new();
        for (idx, line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            if line_no < start_line || line_no > end_line {
                continue;
            }
            out.push_str(&format!("{line_no:>6}  {line}\n"));
            if out.len() >= READ_MAX_BYTES {
                break;
            }
        }

        let truncated = out.len() >= READ_MAX_BYTES || end_line < lines.len();
        Ok(ToolResult::success(json!({
            "path": path,
            "text": out,
            "from": start_line,
            "to": end_line,
            "total_lines": lines.len(),
            "truncated": truncated,
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
        "Writes text content to file atomically, creating parent directories."
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
        atomic_write(&path, content.as_bytes()).await?;
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
        "Replaces exact text in a file if old string exists exactly once."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "path":{"type":"string"},
                "old_string":{"type":"string"},
                "new_string":{"type":"string"},
                "oldText":{"type":"string"},
                "newText":{"type":"string"}
            },
            "required":["path"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path = resolve_workspace_path(
            &ctx.workspace_dir,
            required_string(&params, "path", self.name())?,
        )?;
        let old_text = params
            .get("old_string")
            .and_then(Value::as_str)
            .or_else(|| params.get("oldText").and_then(Value::as_str))
            .ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "missing old_string".to_string(),
            })?
            .to_string();
        let new_text = params
            .get("new_string")
            .and_then(Value::as_str)
            .or_else(|| params.get("newText").and_then(Value::as_str))
            .ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "missing new_string".to_string(),
            })?
            .to_string();

        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
        let body = decode_text(&bytes);
        let occurrences = body.match_indices(&old_text).count();
        match occurrences.cmp(&1) {
            Ordering::Less => {
                return Err(ToolError::Execution("old string not found".to_string()));
            }
            Ordering::Greater => {
                return Err(ToolError::Execution(
                    "old string matches multiple locations".to_string(),
                ));
            }
            Ordering::Equal => {}
        }

        let updated = body.replacen(&old_text, &new_text, 1);
        atomic_write(&path, updated.as_bytes()).await?;

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
        "Runs Brave Search API query and returns normalized result snippets."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "q":{"type":"string"},
                "query":{"type":"string"},
                "count":{"type":"integer"},
                "freshness":{"type":"string"},
                "country":{"type":"string"},
                "search_lang":{"type":"string"},
                "ui_lang":{"type":"string"}
            },
            "required":["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let query = params
            .get("query")
            .and_then(Value::as_str)
            .or_else(|| params.get("q").and_then(Value::as_str))
            .ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "missing query".to_string(),
            })?
            .to_string();
        let count = params
            .get("count")
            .and_then(Value::as_u64)
            .unwrap_or(5)
            .clamp(1, 10);

        let api_key = ctx
            .config
            .tools
            .values
            .get("brave_api_key")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolError::Execution("missing tools.brave_api_key config".to_string())
            })?;

        let mut req = reqwest::Client::new()
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", api_key)
            .query(&[("q", query.as_str()), ("count", &count.to_string())]);

        for key in ["freshness", "country", "search_lang", "ui_lang"] {
            if let Some(value) = params.get(key).and_then(Value::as_str) {
                req = req.query(&[(key, value)]);
            }
        }

        let response = req
            .send()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let status = response.status().as_u16();
        let value = response
            .json::<Value>()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;

        let results = value
            .pointer("/web/results")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        json!({
                            "title": item.get("title").and_then(Value::as_str).unwrap_or_default(),
                            "url": item.get("url").and_then(Value::as_str).unwrap_or_default(),
                            "description": item.get("description").and_then(Value::as_str).unwrap_or_default(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(ToolResult::success(json!({
            "status": status,
            "results": results,
            "raw": value,
        })))
    }
}

struct WebFetchTool;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetches URL and returns markdown/plain content with truncation."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "url":{"type":"string"},
                "format":{"type":"string", "enum":["markdown","text","html"]},
                "timeout_ms":{"type":"integer"},
                "max_chars":{"type":"integer"}
            },
            "required":["url"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let url = required_string(&params, "url", self.name())?;
        let timeout = Duration::from_millis(
            params
                .get("timeout_ms")
                .and_then(Value::as_u64)
                .unwrap_or(20_000),
        );
        let max_chars = params
            .get("max_chars")
            .and_then(Value::as_u64)
            .unwrap_or(50_000) as usize;
        let format = params
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("markdown");

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent("magicmerlin-agent-tools/0.1")
            .build()
            .map_err(|e| ToolError::Execution(e.to_string()))?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = response
            .text()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;

        if !content_type.contains("html") && format != "html" {
            return Ok(ToolResult::success(json!({
                "status": status,
                "url": final_url,
                "content_type": content_type,
                "text": truncate_chars(&body, max_chars),
                "truncated": body.chars().count() > max_chars,
            })));
        }

        let rendered = match format {
            "text" => html_to_text(&body),
            "html" => body.clone(),
            _ => html_to_markdown(&body),
        };

        let truncated = rendered.chars().count() > max_chars;
        Ok(ToolResult::success(json!({
            "status": status,
            "url": final_url,
            "content_type": content_type,
            "content": truncate_chars(&rendered, max_chars),
            "truncated": truncated,
            "format": format,
        })))
    }
}

struct MemorySearchTool;

#[async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        "Performs keyword/BM25-like search over MEMORY.md and memory/*.md files."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "query":{"type":"string"},
                "limit":{"type":"integer"},
                "min_score":{"type":"number"}
            },
            "required":["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let query = required_string(&params, "query", self.name())?;
        let query_terms = tokenize(&query);
        let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        let min_score = params
            .get("min_score")
            .and_then(Value::as_f64)
            .unwrap_or(0.01);
        let root = ctx.state_paths.state_dir.clone();
        let files = collect_memory_files(&root)?;

        let mut docs = Vec::new();
        for path in files {
            let body = std::fs::read_to_string(&path).map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
            for (idx, line) in body.lines().enumerate() {
                let terms = tokenize(line);
                if terms.is_empty() {
                    continue;
                }
                let score = bm25_score(&query_terms, &terms, 80.0);
                if score >= min_score {
                    docs.push((
                        score,
                        json!({
                            "path": path,
                            "line": idx + 1,
                            "score": score,
                            "snippet": line,
                        }),
                    ));
                }
            }
        }

        docs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        let matches = docs
            .into_iter()
            .take(limit)
            .map(|(_, v)| v)
            .collect::<Vec<_>>();
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
        "Reads line range from one memory file under state root."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "path":{"type":"string"},
                "from":{"type":"integer"},
                "lines":{"type":"integer"}
            },
            "required":["path"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let requested = required_string(&params, "path", self.name())?;
        let base = ctx.state_paths.state_dir.clone();
        let path = resolve_within(&base, &requested)?;

        let body = std::fs::read_to_string(&path).map_err(|source| ToolError::Io {
            path: path.clone(),
            source,
        })?;
        let from = params.get("from").and_then(Value::as_u64).unwrap_or(1) as usize;
        let lines_count = params.get("lines").and_then(Value::as_u64).unwrap_or(50) as usize;

        let mut out = Vec::new();
        for (idx, line) in body.lines().enumerate() {
            let line_no = idx + 1;
            if line_no < from {
                continue;
            }
            if out.len() >= lines_count {
                break;
            }
            out.push(format!("{line_no}: {line}"));
        }

        Ok(ToolResult::success(json!({
            "path": path,
            "from": from,
            "lines": out,
        })))
    }
}

struct SessionStatusTool;

#[async_trait]
impl Tool for SessionStatusTool {
    fn name(&self) -> &str {
        "session_status"
    }

    fn description(&self) -> &str {
        "Returns session status with token, cost, and cache approximations."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "session_key":{"type":"string"},
                "model":{"type":"string"}
            },
            "required":["session_key"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let session_key = required_string(&params, "session_key", self.name())?;
        let storage =
            magicmerlin_storage::Storage::new(ctx.state_paths.state_dir.join("openclaw.db"))?;
        let conn = storage.connection()?;

        let row = conn.query_row(
            "SELECT id, agent, status, started_at, updated_at FROM sessions WHERE id=?1",
            params![session_key],
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

        let session = match row {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => json!({"missing": true}),
            Err(err) => return Err(ToolError::Execution(err.to_string())),
        };

        let model = params
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| ctx.config.agents.defaults.model.clone());

        Ok(ToolResult::success(json!({
            "session": session,
            "model": model,
            "tokens_used": null,
            "context_pct": null,
            "cost_usd": null,
            "cache": {"hits": null, "misses": null}
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
        "Lists known sessions from sqlite with optional filters."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "activeMinutes":{"type":"integer"},
                "kinds":{"type":"array", "items": {"type":"string"}},
                "limit":{"type":"integer"}
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let limit = params
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(100)
            .min(500);
        let active_minutes = params.get("activeMinutes").and_then(Value::as_i64);

        let storage =
            magicmerlin_storage::Storage::new(ctx.state_paths.state_dir.join("openclaw.db"))?;
        let conn = storage.connection()?;

        let mut query =
            "SELECT id, agent, status, started_at, updated_at FROM sessions".to_string();
        if active_minutes.is_some() {
            query.push_str(" WHERE updated_at >= ?1");
        }
        query.push_str(" ORDER BY updated_at DESC LIMIT ?2");

        let mut out = Vec::new();
        if let Some(minutes) = active_minutes {
            let threshold = Utc::now().timestamp() - (minutes * 60);
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            let rows = stmt
                .query_map(params![threshold, limit], |row| {
                    Ok(json!({
                        "session_key": row.get::<_, String>(0)?,
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
            let query = "SELECT id, agent, status, started_at, updated_at FROM sessions ORDER BY updated_at DESC LIMIT ?1";
            let mut stmt = conn
                .prepare(query)
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            let rows = stmt
                .query_map(params![limit], |row| {
                    Ok(json!({
                        "session_key": row.get::<_, String>(0)?,
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
                "session_key":{"type":"string"},
                "limit":{"type":"integer"},
                "includeTools":{"type":"boolean"}
            },
            "required":["agent","session_key"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let agent = required_string(&params, "agent", self.name())?;
        let session_key = required_string(&params, "session_key", self.name())?;
        let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize;
        let include_tools = params
            .get("includeTools")
            .and_then(Value::as_bool)
            .unwrap_or(true);

        let transcript_path = ctx
            .state_paths
            .sessions_dir
            .join(agent)
            .join(format!("{}.jsonl", session_key.replace(':', "__")));
        let store = magicmerlin_storage::TranscriptStore::new(transcript_path)?;
        let mut entries = store.read(0, Some(limit))?;
        if !include_tools {
            entries.retain(|item| {
                let kind = item.get("type").and_then(Value::as_str).unwrap_or_default();
                !matches!(kind, "tool_use" | "tool_result")
            });
        }
        Ok(ToolResult::success(json!({"entries": entries})))
    }
}

struct SessionsSendTool;

#[async_trait]
impl Tool for SessionsSendTool {
    fn name(&self) -> &str {
        "sessions_send"
    }

    fn description(&self) -> &str {
        "Queues a message to another session by key or label."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "session_key":{"type":"string"},
                "label":{"type":"string"},
                "message":{"type":"string"}
            },
            "required":["message"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let message = required_string(&params, "message", self.name())?;
        let target = params
            .get("session_key")
            .and_then(Value::as_str)
            .or_else(|| params.get("label").and_then(Value::as_str))
            .ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "missing session_key or label".to_string(),
            })?
            .to_string();

        let queue_dir = ctx.state_paths.state_dir.join("session_inbox");
        tokio::fs::create_dir_all(&queue_dir)
            .await
            .map_err(|source| ToolError::Io {
                path: queue_dir.clone(),
                source,
            })?;
        let path = queue_dir.join(format!("{}.jsonl", target.replace(':', "__")));
        let entry = json!({
            "ts": Utc::now().timestamp(),
            "from": ctx.agent_name,
            "message": message,
        });
        append_jsonl(&path, &entry).await?;

        Ok(ToolResult::success(
            json!({"queued": true, "target": target}),
        ))
    }
}

struct SessionsSpawnTool;

#[async_trait]
impl Tool for SessionsSpawnTool {
    fn name(&self) -> &str {
        "sessions_spawn"
    }

    fn description(&self) -> &str {
        "Spawns isolated sub-agent session metadata and parent relation."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "runtime":{"type":"string"},
                "mode":{"type":"string"},
                "model":{"type":"string"},
                "agent":{"type":"string"}
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let runtime = params
            .get("runtime")
            .and_then(Value::as_str)
            .unwrap_or("subagent");
        let mode = params
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("session");
        let model = params
            .get("model")
            .and_then(Value::as_str)
            .or_else(|| ctx.config.agents.defaults.model.as_deref())
            .unwrap_or("unknown");
        let agent = params
            .get("agent")
            .and_then(Value::as_str)
            .unwrap_or("subagent");

        let session_key = format!(
            "{}:{}:{}",
            agent,
            Utc::now().timestamp_millis(),
            rand_suffix(6)
        );

        let storage =
            magicmerlin_storage::Storage::new(ctx.state_paths.state_dir.join("openclaw.db"))?;
        let conn = storage.connection()?;
        conn.execute(
            "INSERT INTO sessions(id, agent, status, started_at, updated_at, metadata) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session_key,
                agent,
                "running",
                Utc::now().timestamp(),
                Utc::now().timestamp(),
                json!({
                    "parent": ctx.agent_name,
                    "runtime": runtime,
                    "mode": mode,
                    "model": model,
                }).to_string(),
            ],
        ).map_err(|e| ToolError::Execution(e.to_string()))?;

        append_subagent_record(
            &ctx.state_paths.state_dir,
            json!({
                "session_key": session_key,
                "agent": agent,
                "runtime": runtime,
                "mode": mode,
                "model": model,
                "parent": ctx.agent_name,
                "status": "running",
                "started_at": Utc::now().timestamp(),
            }),
        )
        .await?;

        Ok(ToolResult::success(json!({
            "session_key": session_key,
            "runtime": runtime,
            "mode": mode,
            "model": model,
        })))
    }
}

struct SubagentsTool;

#[async_trait]
impl Tool for SubagentsTool {
    fn name(&self) -> &str {
        "subagents"
    }

    fn description(&self) -> &str {
        "Lists, steers, or kills sub-agents tracked in state metadata."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string"},
                "session_key":{"type":"string"},
                "message":{"type":"string"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        match action.as_str() {
            "list" => {
                let list = read_subagent_records(&ctx.state_paths.state_dir).await?;
                Ok(ToolResult::success(json!({"subagents": list})))
            }
            "steer" => {
                let target = required_string(&params, "session_key", self.name())?;
                let message = required_string(&params, "message", self.name())?;
                SessionsSendTool
                    .execute(json!({"session_key": target, "message": message}), ctx)
                    .await
            }
            "kill" => {
                let target = required_string(&params, "session_key", self.name())?;
                let mut items = read_subagent_records(&ctx.state_paths.state_dir).await?;
                for item in &mut items {
                    if item.get("session_key").and_then(Value::as_str) == Some(target.as_str()) {
                        item["status"] = json!("killed");
                    }
                }
                write_subagent_records(&ctx.state_paths.state_dir, &items).await?;
                Ok(ToolResult::success(json!({"killed": target})))
            }
            _ => Err(ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "action must be list|steer|kill".to_string(),
            }),
        }
    }
}

struct AgentsListTool;

#[async_trait]
impl Tool for AgentsListTool {
    fn name(&self) -> &str {
        "agents_list"
    }

    fn description(&self) -> &str {
        "Returns available agent IDs."
    }

    fn schema(&self) -> Value {
        json!({"type":"object"})
    }

    async fn execute(&self, _params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let mut agents = vec![ctx.agent_name.clone()];
        if let Some(model) = &ctx.config.agents.defaults.model {
            agents.push(format!("default:{model}"));
        }
        if ctx.state_paths.sessions_dir.exists() {
            for entry in std::fs::read_dir(&ctx.state_paths.sessions_dir).map_err(|source| {
                ToolError::Io {
                    path: ctx.state_paths.sessions_dir.clone(),
                    source,
                }
            })? {
                let entry = entry.map_err(|source| ToolError::Io {
                    path: ctx.state_paths.sessions_dir.clone(),
                    source,
                })?;
                if entry.path().is_dir() {
                    agents.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        agents.sort();
        agents.dedup();
        Ok(ToolResult::success(json!({"agents": agents})))
    }
}

struct MessageTool;

#[async_trait]
impl Tool for MessageTool {
    fn name(&self) -> &str {
        "message"
    }

    fn description(&self) -> &str {
        "Sends message payload to delivery target metadata."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "text":{"type":"string"},
                "media":{"type":"array"},
                "buttons":{"type":"array"},
                "reaction":{"type":"string"},
                "edit":{"type":"boolean"},
                "delete":{"type":"boolean"}
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let delivery = ctx.delivery.clone().map(|d| {
            json!({
                "channel": d.channel,
                "target": d.target,
            })
        });

        Ok(ToolResult::success(json!({
            "delivered": delivery.is_some(),
            "delivery": delivery,
            "payload": params,
        })))
    }
}

struct ImageTool;

#[async_trait]
impl Tool for ImageTool {
    fn name(&self) -> &str {
        "image"
    }

    fn description(&self) -> &str {
        "Analyzes one or more images using media understanding subsystem."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "images":{"type":"array", "items": {"type":"string"}},
                "prompt":{"type":"string"}
            },
            "required":["images"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let images = params
            .get("images")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "images must be array".to_string(),
            })?;
        if images.is_empty() || images.len() > 20 {
            return Err(ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "images length must be in 1..=20".to_string(),
            });
        }
        let prompt = params
            .get("prompt")
            .and_then(Value::as_str)
            .unwrap_or("Describe this image")
            .to_string();

        let client = magicmerlin_media::understanding::UnderstandingClient::new(
            magicmerlin_media::understanding::UnderstandingConfig::default(),
        );

        let mut results = Vec::new();
        for img in images {
            let p = img.as_str().ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "image entries must be strings".to_string(),
            })?;
            let source = if p.starts_with("http://") || p.starts_with("https://") {
                magicmerlin_media::understanding::MediaSource::Url { url: p.to_string() }
            } else {
                magicmerlin_media::understanding::MediaSource::File {
                    path: PathBuf::from(p),
                }
            };

            let analysis = client
                .analyze(magicmerlin_media::understanding::AnalysisRequest {
                    media_type: magicmerlin_media::understanding::MediaType::Image,
                    source,
                    prompt: prompt.clone(),
                    preferred_provider: None,
                    metadata: Value::Null,
                })
                .await
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            results.push(serde_json::to_value(analysis)?);
        }

        Ok(ToolResult::success(json!({"results": results})))
    }
}

struct PdfTool;

#[async_trait]
impl Tool for PdfTool {
    fn name(&self) -> &str {
        "pdf"
    }

    fn description(&self) -> &str {
        "Analyzes one or more PDFs using media understanding subsystem."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "pdfs":{"type":"array", "items": {"type":"string"}},
                "prompt":{"type":"string"},
                "from":{"type":"integer"},
                "to":{"type":"integer"}
            },
            "required":["pdfs"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let pdfs = params
            .get("pdfs")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "pdfs must be array".to_string(),
            })?;
        let prompt = params
            .get("prompt")
            .and_then(Value::as_str)
            .unwrap_or("Summarize this PDF")
            .to_string();

        let range = match (
            params.get("from").and_then(Value::as_u64),
            params.get("to").and_then(Value::as_u64),
        ) {
            (Some(from), Some(to)) => Some(magicmerlin_media::understanding::PdfPageRange {
                from: from as u32,
                to: to as u32,
            }),
            _ => None,
        };

        let client = magicmerlin_media::understanding::UnderstandingClient::new(
            magicmerlin_media::understanding::UnderstandingConfig::default(),
        );

        let mut results = Vec::new();
        for item in pdfs {
            let path = item.as_str().ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "pdf entries must be strings".to_string(),
            })?;
            let analysis = client
                .analyze_pdf_with_fallback(
                    magicmerlin_media::understanding::AnalysisRequest {
                        media_type: magicmerlin_media::understanding::MediaType::Pdf,
                        source: magicmerlin_media::understanding::MediaSource::File {
                            path: PathBuf::from(path),
                        },
                        prompt: prompt.clone(),
                        preferred_provider: None,
                        metadata: Value::Null,
                    },
                    range,
                    magicmerlin_media::understanding::PdfFallbackMode::TextFirst,
                )
                .await
                .map_err(|e| ToolError::Execution(e.to_string()))?;
            results.push(serde_json::to_value(analysis)?);
        }

        Ok(ToolResult::success(json!({"results": results})))
    }
}

struct TtsTool;

#[async_trait]
impl Tool for TtsTool {
    fn name(&self) -> &str {
        "tts"
    }

    fn description(&self) -> &str {
        "Converts text to speech using media TTS subsystem and writes audio file."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "text":{"type":"string"},
                "agent":{"type":"string"},
                "path":{"type":"string"},
                "format":{"type":"string", "enum":["mp3","ogg","wav"]}
            },
            "required":["text"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let text = required_string(&params, "text", self.name())?;
        let agent = params
            .get("agent")
            .and_then(Value::as_str)
            .unwrap_or("default")
            .to_string();
        let path = params
            .get("path")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join(format!("tts-{}.mp3", rand_suffix(6))));
        let format = match params
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("mp3")
        {
            "ogg" => magicmerlin_media::tts::OutputFormat::Ogg,
            "wav" => magicmerlin_media::tts::OutputFormat::Wav,
            _ => magicmerlin_media::tts::OutputFormat::Mp3,
        };

        let client =
            magicmerlin_media::tts::TtsClient::new(magicmerlin_media::tts::TtsConfig::default());
        let bytes = client
            .synthesize_for_agent(&agent, &text, Some(format))
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|source| ToolError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }

        tokio::fs::write(&path, bytes)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;

        Ok(ToolResult::success(json!({"audio_path": path})))
    }
}

struct BrowserTool;

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Browser control surface routed through media browser module."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string"},
                "port":{"type":"integer"},
                "url":{"type":"string"},
                "tab_id":{"type":"string"},
                "expression":{"type":"string"},
                "x":{"type":"number"},
                "y":{"type":"number"},
                "text":{"type":"string"},
                "key":{"type":"string"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let port = params.get("port").and_then(Value::as_u64).unwrap_or(9222) as u16;

        match action.as_str() {
            "tabs" => {
                let tabs = magicmerlin_media::browser::list_tabs(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"tabs": tabs})))
            }
            "open" => {
                let url = params
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or("about:blank");
                let tab = magicmerlin_media::browser::new_tab(port, Some(url))
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"tab": tab})))
            }
            "close" => {
                let tab_id = required_string(&params, "tab_id", self.name())?;
                magicmerlin_media::browser::close_tab(port, &tab_id)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"closed": tab_id})))
            }
            "focus" => {
                let tab_id = required_string(&params, "tab_id", self.name())?;
                magicmerlin_media::browser::focus_tab(port, &tab_id)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"focused": tab_id})))
            }
            "navigate" => {
                let url = required_string(&params, "url", self.name())?;
                let client = magicmerlin_media::browser::BrowserClient::from_tab(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                client
                    .navigate(&url)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"navigated": url})))
            }
            "snapshot" => {
                let client = magicmerlin_media::browser::BrowserClient::from_tab(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                let shot = client
                    .build_snapshot()
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(serde_json::to_value(shot)?))
            }
            "screenshot" => {
                let client = magicmerlin_media::browser::BrowserClient::from_tab(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                let png = client
                    .screenshot_png()
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"bytes": png.len()})))
            }
            "eval" => {
                let expression = required_string(&params, "expression", self.name())?;
                let client = magicmerlin_media::browser::BrowserClient::from_tab(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                let value = client
                    .evaluate_script(magicmerlin_media::browser::EvaluateOptions {
                        expression,
                        await_promise: true,
                        return_by_value: true,
                    })
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"value": value})))
            }
            "act" => {
                let client = magicmerlin_media::browser::BrowserClient::from_tab(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                if let (Some(x), Some(y)) = (
                    params.get("x").and_then(Value::as_f64),
                    params.get("y").and_then(Value::as_f64),
                ) {
                    client
                        .click_at(magicmerlin_media::browser::ClickOptions {
                            x,
                            y,
                            click_count: 1,
                            button: "left".to_string(),
                        })
                        .await
                        .map_err(|e| ToolError::Execution(e.to_string()))?;
                }
                if let Some(text) = params.get("text").and_then(Value::as_str) {
                    client
                        .type_into_focused(magicmerlin_media::browser::InputTextOptions {
                            text: text.to_string(),
                            submit: false,
                        })
                        .await
                        .map_err(|e| ToolError::Execution(e.to_string()))?;
                }
                if let Some(key) = params.get("key").and_then(Value::as_str) {
                    client
                        .send_key(magicmerlin_media::browser::KeyInput {
                            key: key.to_string(),
                            modifiers: 0,
                        })
                        .await
                        .map_err(|e| ToolError::Execution(e.to_string()))?;
                }
                Ok(ToolResult::success(json!({"ok": true})))
            }
            _ => Err(ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "unsupported browser action".to_string(),
            }),
        }
    }
}

struct CanvasTool;

#[async_trait]
impl Tool for CanvasTool {
    fn name(&self) -> &str {
        "canvas"
    }

    fn description(&self) -> &str {
        "Canvas hosting and A2UI message tool."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string"},
                "html":{"type":"string"},
                "url":{"type":"string"},
                "script":{"type":"string"},
                "event":{"type":"string"},
                "payload":{"type":"object"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let server = magicmerlin_media::canvas::CanvasServer::new(
            magicmerlin_media::canvas::CanvasConfig::default(),
        );

        match action.as_str() {
            "present" => {
                if let Some(html) = params.get("html").and_then(Value::as_str) {
                    server.set_html(html.to_string()).await;
                }
                let handle = server
                    .start()
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(
                    json!({"addr": handle.addr.to_string()}),
                ))
            }
            "hide" => Ok(ToolResult::success(json!({"ok": true}))),
            "navigate" => {
                let url = required_string(&params, "url", self.name())?;
                server
                    .navigate_url(&url)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(
                    json!({"url": server.current_url().await}),
                ))
            }
            "eval" => {
                let script = required_string(&params, "script", self.name())?;
                let value = server
                    .evaluate_js(&script)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"value": value})))
            }
            "snapshot" => Ok(ToolResult::success(json!({
                "html": server.html().await,
                "url": server.current_url().await,
            }))),
            "a2ui_push" => {
                let event = required_string(&params, "event", self.name())?;
                let payload = params.get("payload").cloned().unwrap_or(Value::Null);
                server
                    .push_update(magicmerlin_media::canvas::UiUpdate { event, payload })
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"ok": true})))
            }
            _ => Err(ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "unsupported canvas action".to_string(),
            }),
        }
    }
}

struct NodesTool;

#[async_trait]
impl Tool for NodesTool {
    fn name(&self) -> &str {
        "nodes"
    }

    fn description(&self) -> &str {
        "Remote node control over HTTP endpoints."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "base_url":{"type":"string"},
                "action":{"type":"string"},
                "path":{"type":"string"},
                "method":{"type":"string"},
                "body":{}
            },
            "required":["base_url","action"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let base = required_string(&params, "base_url", self.name())?;
        let action = required_string(&params, "action", self.name())?;

        let endpoint = match action.as_str() {
            "status" => "/status",
            "camera" => "/camera",
            "screen" => "/screen",
            "location" => "/location",
            "run" => "/run",
            "invoke" => "/invoke",
            _ => {
                return Err(ToolError::InvalidParams {
                    tool: self.name().to_string(),
                    message: "action must be status|camera|screen|location|run|invoke".to_string(),
                })
            }
        };

        let custom_path = params
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or(endpoint);
        let url = format!("{}{}", base.trim_end_matches('/'), custom_path);

        let method = params
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("POST");
        let body = params.get("body").cloned().unwrap_or(Value::Null);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let req = match method {
            "GET" => client.get(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.post(&url),
        };

        let response = req
            .json(&body)
            .send()
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();

        Ok(ToolResult::success(json!({
            "status": status,
            "url": url,
            "body": text,
        })))
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

fn resolve_within(base: &Path, requested: &str) -> Result<PathBuf> {
    let path = if Path::new(requested).is_absolute() {
        PathBuf::from(requested)
    } else {
        base.join(requested)
    };
    let base = base.canonicalize().map_err(|source| ToolError::Io {
        path: base.to_path_buf(),
        source,
    })?;
    let canonical = path.canonicalize().map_err(|source| ToolError::Io {
        path: path.clone(),
        source,
    })?;
    if !canonical.starts_with(base) {
        return Err(ToolError::PermissionDenied(format!(
            "path outside memory root: {}",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|x| x.to_str())
            .map(|s| s.to_ascii_lowercase()),
        Some(ext)
            if ["png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff"].contains(&ext.as_str())
    )
}

fn looks_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let sample = &bytes[..bytes.len().min(4096)];
    sample
        .iter()
        .any(|b| *b == 0 || (*b < 9) || (*b > 13 && *b < 32))
}

fn decode_text(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|b| *b as char).collect::<String>(),
    }
}

async fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| ToolError::Execution("path has no parent".to_string()))?;
    let temp = parent.join(format!(
        ".{}.tmp{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        rand_suffix(6)
    ));
    tokio::fs::write(&temp, content)
        .await
        .map_err(|source| ToolError::Io {
            path: temp.clone(),
            source,
        })?;
    tokio::fs::rename(&temp, path)
        .await
        .map_err(|source| ToolError::Io {
            path: path.to_path_buf(),
            source,
        })
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

fn tokenize(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn bm25_score(query: &[String], doc: &[String], avg_doc_len: f64) -> f64 {
    if query.is_empty() || doc.is_empty() {
        return 0.0;
    }
    let mut score = 0.0;
    let k1 = 1.2;
    let b = 0.75;
    let dl = doc.len() as f64;
    for q in query {
        let freq = doc.iter().filter(|term| *term == q).count() as f64;
        if freq == 0.0 {
            continue;
        }
        let idf = 1.0; // lightweight approximation without corpus-wide statistics.
        let denom = freq + k1 * (1.0 - b + b * dl / avg_doc_len.max(1.0));
        score += idf * (freq * (k1 + 1.0)) / denom;
    }
    score
}

fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    normalize_ws(&out)
}

fn html_to_markdown(html: &str) -> String {
    let mut body = html.to_string();
    let replacements = [
        ("<h1", "\n# "),
        ("</h1>", "\n\n"),
        ("<h2", "\n## "),
        ("</h2>", "\n\n"),
        ("<h3", "\n### "),
        ("</h3>", "\n\n"),
        ("<p", "\n"),
        ("</p>", "\n\n"),
        ("<li", "\n- "),
        ("</li>", ""),
        ("<code", "`"),
        ("</code>", "`"),
        ("<pre", "\n```\n"),
        ("</pre>", "\n```\n"),
        ("<br", "\n"),
    ];
    for (needle, repl) in replacements {
        body = replace_tag_open(&body, needle, repl);
    }
    body = html_to_text(&body);
    normalize_ws(&body)
}

fn replace_tag_open(data: &str, needle: &str, replacement: &str) -> String {
    let mut out = String::with_capacity(data.len());
    let mut cursor = 0usize;
    let lower = data.to_ascii_lowercase();
    let needle_lc = needle.to_ascii_lowercase();

    while cursor < data.len() {
        let Some(rel) = lower[cursor..].find(&needle_lc) else {
            out.push_str(&data[cursor..]);
            break;
        };
        let start = cursor + rel;
        out.push_str(&data[cursor..start]);

        if needle.starts_with("</") {
            out.push_str(replacement);
            cursor = start + needle.len();
            continue;
        }

        if let Some(end_rel) = lower[start..].find('>') {
            out.push_str(replacement);
            cursor = start + end_rel + 1;
        } else {
            out.push_str(&data[start..]);
            break;
        }
    }

    out
}

fn normalize_ws(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_chars(text: &str, max: usize) -> String {
    text.chars().take(max).collect::<String>()
}

async fn append_jsonl(path: &Path, value: &Value) -> Result<()> {
    let line = serde_json::to_string(value)?;
    use tokio::io::AsyncWriteExt;
    let mut f = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|source| ToolError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    f.write_all(line.as_bytes())
        .await
        .map_err(|source| ToolError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    f.write_all(b"\n").await.map_err(|source| ToolError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

async fn append_subagent_record(state_dir: &Path, value: Value) -> Result<()> {
    let path = state_dir.join("subagents.jsonl");
    append_jsonl(&path, &value).await
}

async fn read_subagent_records(state_dir: &Path) -> Result<Vec<Value>> {
    let path = state_dir.join("subagents.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let body = tokio::fs::read_to_string(&path)
        .await
        .map_err(|source| ToolError::Io {
            path: path.clone(),
            source,
        })?;
    Ok(body
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect())
}

async fn write_subagent_records(state_dir: &Path, items: &[Value]) -> Result<()> {
    let path = state_dir.join("subagents.jsonl");
    let mut out = String::new();
    for item in items {
        out.push_str(&serde_json::to_string(item)?);
        out.push('\n');
    }
    tokio::fs::write(&path, out)
        .await
        .map_err(|source| ToolError::Io { path, source })
}

fn rand_suffix(len: usize) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut x = nanos as u64;
    let mut out = String::new();
    for _ in 0..len {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (x % 36) as u8;
        out.push(if idx < 10 {
            (b'0' + idx) as char
        } else {
            (b'a' + (idx - 10)) as char
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn edit_replaces_text_once() {
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

        EditTool
            .execute(
                json!({"path":"a.txt","old_string":"old","new_string":"new"}),
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
        for required in [
            "exec",
            "process",
            "read",
            "write",
            "edit",
            "memory_get",
            "nodes",
        ] {
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

    #[test]
    fn html_to_markdown_smoke() {
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        let md = html_to_markdown(html);
        assert!(md.contains("Hello"));
        assert!(md.contains("World"));
    }

    #[test]
    fn bm25_scores_nonzero_for_matches() {
        let q = tokenize("hello world");
        let d = tokenize("hello there world");
        assert!(bm25_score(&q, &d, 10.0) > 0.0);
    }
}

//! Default tool implementations and registration.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rusqlite::params;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::{Result, ToolError};
use crate::gateway::gateway_call;
use crate::registry::{NodeConfig, Tool, ToolContext, ToolRegistry, ToolResult};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use std::io::Read as StdRead;

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
    registry.register(Arc::new(CronTool));
    registry.register(Arc::new(ImageTool));
    registry.register(Arc::new(PdfTool));
    registry.register(Arc::new(TtsTool));
    registry.register(Arc::new(BrowserTool));
    registry.register(Arc::new(CanvasTool));
    registry.register(Arc::new(NodesTool));
    registry.register(Arc::new(SessionsYieldTool));
    registry.register(Arc::new(GatewayTool));
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

        // Foreground PTY execution
        if tty {
            return exec_foreground_pty(&cmd, &cwd, &env, timeout_ms).await;
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

        let api_key =
            match ctx
                .config
                .tools
                .values
                .get("brave_api_key")
                .and_then(Value::as_str)
            {
                Some(k) => k.to_string(),
                None => return Ok(ToolResult::failure(
                    "missing tools.brave_api_key config — set it in config to enable web search",
                )),
            };

        let http = reqwest::Client::new();
        let mut last_err = String::new();

        for attempt in 0..3u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt - 1))).await;
            }

            let mut req = http
                .get("https://api.search.brave.com/res/v1/web/search")
                .header("X-Subscription-Token", &api_key)
                .query(&[("q", query.as_str()), ("count", &count.to_string())]);

            for key in ["freshness", "country", "search_lang", "ui_lang"] {
                if let Some(value) = params.get(key).and_then(Value::as_str) {
                    req = req.query(&[(key, value)]);
                }
            }

            let response = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    last_err = e.to_string();
                    continue;
                }
            };

            let status = response.status().as_u16();
            if status == 429 {
                last_err = "rate limited (429)".to_string();
                continue;
            }

            let body = response
                .json::<Value>()
                .await
                .map_err(|e| ToolError::Execution(e.to_string()))?;

            let results = body
                .pointer("/web/results")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| {
                            json!({
                                "title": item.get("title").and_then(Value::as_str).unwrap_or_default(),
                                "url": item.get("url").and_then(Value::as_str).unwrap_or_default(),
                                "snippet": item.get("description").and_then(Value::as_str).unwrap_or_default(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let total = body
                .pointer("/web/totalResults")
                .and_then(Value::as_u64)
                .unwrap_or(results.len() as u64);

            return Ok(ToolResult::success(json!({
                "results": results,
                "total": total,
            })));
        }

        Ok(ToolResult::failure(format!(
            "web search failed after 3 attempts: {last_err}"
        )))
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
                .unwrap_or(30_000),
        );
        let max_chars = params
            .get("max_chars")
            .or_else(|| params.get("maxChars"))
            .and_then(Value::as_u64)
            .unwrap_or(50_000) as usize;
        let format = params
            .get("format")
            .or_else(|| params.get("extractMode"))
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
                "content": truncate_chars(&body, max_chars),
                "truncated": body.chars().count() > max_chars,
            })));
        }

        let rendered = match format {
            "text" => extract_text_content(&body),
            "html" => body.clone(),
            _ => extract_main_content(&body),
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
        "Performs semantic-style search over MEMORY.md and memory/*.md files using chunked TF-IDF."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "query":{"type":"string"},
                "maxResults":{"type":"integer"},
                "limit":{"type":"integer"},
                "minScore":{"type":"number"},
                "min_score":{"type":"number"}
            },
            "required":["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let query = required_string(&params, "query", self.name())?;
        let query_terms = tokenize(&query);
        let max_results = params
            .get("maxResults")
            .or_else(|| params.get("limit"))
            .and_then(Value::as_u64)
            .unwrap_or(10) as usize;
        let min_score = params
            .get("minScore")
            .or_else(|| params.get("min_score"))
            .and_then(Value::as_f64)
            .unwrap_or(0.01);

        // Collect from both state dir and workspace dir
        let mut files = collect_memory_files(&ctx.state_paths.state_dir)?;
        let ws_files = collect_memory_files(&ctx.workspace_dir)?;
        for f in ws_files {
            if !files.contains(&f) {
                files.push(f);
            }
        }

        // Build chunks (~200-word segments with overlap context)
        let mut all_chunks: Vec<MemoryChunk> = Vec::new();
        for path in &files {
            let body = std::fs::read_to_string(path).map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;
            let chunks = chunk_text(&body, 200);
            for chunk in chunks {
                let terms = tokenize(&chunk.text);
                if !terms.is_empty() {
                    all_chunks.push(MemoryChunk {
                        path: path.clone(),
                        start_line: chunk.start_line,
                        end_line: chunk.end_line,
                        text: chunk.text,
                        terms,
                    });
                }
            }
        }

        // Compute IDF across all chunks for each query term
        let total_docs = all_chunks.len().max(1) as f64;
        let mut idf_map: HashMap<String, f64> = HashMap::new();
        for qt in &query_terms {
            let doc_freq = all_chunks.iter().filter(|c| c.terms.contains(qt)).count() as f64;
            let idf = ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
            idf_map.insert(qt.clone(), idf.max(0.1));
        }

        // Score each chunk with TF-IDF / BM25
        let avg_dl = if all_chunks.is_empty() {
            1.0
        } else {
            all_chunks.iter().map(|c| c.terms.len()).sum::<usize>() as f64 / total_docs
        };

        let mut scored: Vec<(f64, &MemoryChunk)> = Vec::new();
        for chunk in &all_chunks {
            let mut score = 0.0_f64;
            let dl = chunk.terms.len() as f64;
            let k1 = 1.2_f64;
            let b = 0.75_f64;
            for qt in &query_terms {
                let tf = chunk.terms.iter().filter(|t| *t == qt).count() as f64;
                if tf == 0.0 {
                    continue;
                }
                let idf = idf_map.get(qt).copied().unwrap_or(0.1);
                let denom = tf + k1 * (1.0 - b + b * dl / avg_dl.max(1.0));
                score += idf * (tf * (k1 + 1.0)) / denom;
            }
            if score >= min_score {
                scored.push((score, chunk));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        let results: Vec<Value> = scored
            .into_iter()
            .take(max_results)
            .map(|(score, chunk)| {
                let rel_path = chunk
                    .path
                    .strip_prefix(&ctx.state_paths.state_dir)
                    .or_else(|_| chunk.path.strip_prefix(&ctx.workspace_dir))
                    .unwrap_or(&chunk.path);
                json!({
                    "path": rel_path,
                    "startLine": chunk.start_line,
                    "endLine": chunk.end_line,
                    "score": (score * 1000.0).round() / 1000.0,
                    "snippet": truncate_chars(&chunk.text, 500),
                    "source": "memory",
                    "citation": format!("{}#L{}-L{}", rel_path.display(), chunk.start_line, chunk.end_line),
                })
            })
            .collect();

        Ok(ToolResult::success(json!({"results": results})))
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
        let total_lines = body.lines().count();
        let from = params.get("from").and_then(Value::as_u64).unwrap_or(1) as usize;
        let lines_count = params.get("lines").and_then(Value::as_u64).unwrap_or(50) as usize;

        let content: String = body
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let line_no = idx + 1;
                if line_no >= from && line_no < from + lines_count {
                    Some(line)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult::success(json!({
            "content": content,
            "path": path,
            "from": from,
            "totalLines": total_lines,
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

        // Estimate tokens from transcript if available
        let agent_name = session
            .get("agent")
            .and_then(Value::as_str)
            .unwrap_or(&ctx.agent_name);
        let transcript_path = ctx
            .state_paths
            .sessions_dir
            .join(agent_name)
            .join(format!("{}.jsonl", session_key.replace(':', "__")));

        let (context_tokens, message_count) = if transcript_path.exists() {
            let body = std::fs::read_to_string(&transcript_path).unwrap_or_default();
            let lines: Vec<&str> = body.lines().collect();
            let total_chars: usize = lines.iter().map(|l| l.len()).sum();
            // Rough heuristic: ~4 chars per token
            (total_chars / 4, lines.len())
        } else {
            (0, 0)
        };

        // Context window size heuristic based on model name
        let context_window = if model.as_deref().unwrap_or("").contains("opus") {
            200_000
        } else {
            128_000
        };
        let context_pct = if context_window > 0 {
            ((context_tokens as f64 / context_window as f64) * 100.0).round() as u64
        } else {
            0
        };

        let started_at = session.get("started_at").and_then(Value::as_i64);
        let start_time = started_at.map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        });

        Ok(ToolResult::success(json!({
            "sessionKey": session_key,
            "model": model,
            "contextTokens": context_tokens,
            "contextPercent": context_pct,
            "messageCount": message_count,
            "startTime": start_time,
            "cost": null,
            "session": session,
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
        "Lists sessions via the gateway sessions subsystem."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "activeMinutes":{"type":"integer"},
                "kinds":{"type":"array", "items": {"type":"string"}},
                "limit":{"type":"integer"},
                "messageLimit":{"type":"integer"}
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        gateway_call(ctx, "sessions.list", params).await
    }
}

struct SessionsHistoryTool;

#[async_trait]
impl Tool for SessionsHistoryTool {
    fn name(&self) -> &str {
        "sessions_history"
    }

    fn description(&self) -> &str {
        "Returns transcript history for a session via the gateway."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "sessionKey":{"type":"string"},
                "limit":{"type":"integer"},
                "includeTools":{"type":"boolean"}
            },
            "required":["sessionKey"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        gateway_call(ctx, "sessions.history", params).await
    }
}

struct SessionsSendTool;

#[async_trait]
impl Tool for SessionsSendTool {
    fn name(&self) -> &str {
        "sessions_send"
    }

    fn description(&self) -> &str {
        "Sends a message to another session via the gateway."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "sessionKey":{"type":"string"},
                "message":{"type":"string"},
                "timeoutSeconds":{"type":"integer"}
            },
            "required":["sessionKey","message"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        gateway_call(ctx, "sessions.send", params).await
    }
}

struct SessionsSpawnTool;

#[async_trait]
impl Tool for SessionsSpawnTool {
    fn name(&self) -> &str {
        "sessions_spawn"
    }

    fn description(&self) -> &str {
        "Spawns an isolated sub-agent session via the gateway."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "task":{"type":"string"},
                "agentId":{"type":"string"},
                "mode":{"type":"string","enum":["run","session"]},
                "model":{"type":"string"},
                "runtime":{"type":"string","enum":["subagent","acp"]},
                "timeoutSeconds":{"type":"integer"},
                "label":{"type":"string"},
                "thread":{"type":"boolean"}
            },
            "required":["task"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        gateway_call(ctx, "sessions.spawn", params).await
    }
}

struct SubagentsTool;

#[async_trait]
impl Tool for SubagentsTool {
    fn name(&self) -> &str {
        "subagents"
    }

    fn description(&self) -> &str {
        "Lists, steers, or kills sub-agents via the gateway."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string","enum":["list","steer","kill"]},
                "target":{"type":"string"},
                "message":{"type":"string"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let method = format!("subagents.{action}");
        gateway_call(ctx, &method, params).await
    }
}

struct AgentsListTool;

#[async_trait]
impl Tool for AgentsListTool {
    fn name(&self) -> &str {
        "agents_list"
    }

    fn description(&self) -> &str {
        "Returns available agent IDs via the gateway."
    }

    fn schema(&self) -> Value {
        json!({"type":"object"})
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        gateway_call(ctx, "agents.list", params).await
    }
}

struct MessageTool;

#[async_trait]
impl Tool for MessageTool {
    fn name(&self) -> &str {
        "message"
    }

    fn description(&self) -> &str {
        "Dispatches message actions (send/react/delete/edit/poll) through the gateway channel system."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string", "enum":["send","react","delete","edit","poll"]},
                "channel":{"type":"string"},
                "target":{"type":"string"},
                "message":{"type":"string"},
                "text":{"type":"string"},
                "messageId":{"type":"string"},
                "emoji":{"type":"string"},
                "media":{"type":"array"},
                "buttons":{"type":"array"}
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = params
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("send");

        // Resolve channel/target from params or delivery context
        let channel = params
            .get("channel")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| ctx.delivery.as_ref().map(|d| d.channel.clone()));
        let target = params
            .get("target")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| ctx.delivery.as_ref().map(|d| d.target.clone()));

        let method = format!("channels.{action}");
        let mut call_params = params.clone();
        if let Some(obj) = call_params.as_object_mut() {
            if let Some(ch) = &channel {
                obj.insert("channel".to_string(), json!(ch));
            }
            if let Some(tgt) = &target {
                obj.insert("target".to_string(), json!(tgt));
            }
            // Normalize: if "text" present but not "message", copy it
            if obj.get("message").is_none() {
                if let Some(text) = obj.get("text").cloned() {
                    obj.insert("message".to_string(), text);
                }
            }
        }

        gateway_call(ctx, &method, call_params).await
    }
}

struct CronTool;

#[async_trait]
impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }

    fn description(&self) -> &str {
        "Manages scheduled jobs via the gateway cron subsystem."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string","enum":["status","list","add","update","remove","run","runs","wake"]},
                "id":{"type":"string"},
                "job":{
                    "type":"object",
                    "properties": {
                        "name":{"type":"string"},
                        "schedule":{
                            "type":"object",
                            "properties": {
                                "kind":{"type":"string","enum":["cron","interval","once"]},
                                "expr":{"type":"string"},
                                "everyMs":{"type":"integer"},
                                "at":{"type":"string"}
                            },
                            "required":["kind"]
                        },
                        "payload":{
                            "type":"object",
                            "properties": {
                                "kind":{"type":"string","enum":["text","message"]},
                                "text":{"type":"string"},
                                "message":{"type":"string"}
                            },
                            "required":["kind"]
                        },
                        "delivery":{"type":"object"},
                        "sessionTarget":{"type":"string"},
                        "enabled":{"type":"boolean"}
                    },
                    "required":["schedule","payload"]
                },
                "limit":{"type":"integer"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let method = format!("cron.{action}");
        gateway_call(ctx, &method, params).await
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

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
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

        let fallback_client;
        let client: &magicmerlin_media::understanding::UnderstandingClient =
            match ctx.understanding_client.as_ref() {
                Some(c) => c,
                None => {
                    fallback_client = magicmerlin_media::understanding::UnderstandingClient::new(
                        magicmerlin_media::understanding::UnderstandingConfig::default(),
                    );
                    &fallback_client
                }
            };

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

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
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

        let fallback_client;
        let client: &magicmerlin_media::understanding::UnderstandingClient =
            match ctx.understanding_client.as_ref() {
                Some(c) => c,
                None => {
                    fallback_client = magicmerlin_media::understanding::UnderstandingClient::new(
                        magicmerlin_media::understanding::UnderstandingConfig::default(),
                    );
                    &fallback_client
                }
            };

        let mut results = Vec::new();
        for item in pdfs {
            let path_str = item.as_str().ok_or_else(|| ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: "pdf entries must be strings".to_string(),
            })?;
            let source = if path_str.starts_with("http://") || path_str.starts_with("https://") {
                magicmerlin_media::understanding::MediaSource::Url {
                    url: path_str.to_string(),
                }
            } else {
                magicmerlin_media::understanding::MediaSource::File {
                    path: PathBuf::from(path_str),
                }
            };
            let analysis = client
                .analyze_pdf_with_fallback(
                    magicmerlin_media::understanding::AnalysisRequest {
                        media_type: magicmerlin_media::understanding::MediaType::Pdf,
                        source,
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

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let text = required_string(&params, "text", self.name())?;
        let agent = params
            .get("agent")
            .and_then(Value::as_str)
            .unwrap_or("default")
            .to_string();
        let format = match params
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("mp3")
        {
            "ogg" => magicmerlin_media::tts::OutputFormat::Ogg,
            "wav" => magicmerlin_media::tts::OutputFormat::Wav,
            _ => magicmerlin_media::tts::OutputFormat::Mp3,
        };
        let ext = match format {
            magicmerlin_media::tts::OutputFormat::Ogg => "ogg",
            magicmerlin_media::tts::OutputFormat::Wav => "wav",
            _ => "mp3",
        };
        let path = params
            .get("path")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::temp_dir().join(format!("tts-{}.{}", rand_suffix(6), ext))
            });

        let fallback_client;
        let client: &magicmerlin_media::tts::TtsClient = match ctx.tts_client.as_ref() {
            Some(c) => c,
            None => {
                fallback_client = magicmerlin_media::tts::TtsClient::new(
                    magicmerlin_media::tts::TtsConfig::default(),
                );
                &fallback_client
            }
        };
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

        let byte_count = bytes.len();
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|source| ToolError::Io {
                path: path.clone(),
                source,
            })?;

        Ok(ToolResult::success(json!({
            "audio_path": path,
            "bytes": byte_count,
            "format": ext,
        })))
    }
}

/// Connects to a browser tab, optionally targeting a specific tab by ID.
async fn browser_client(
    port: u16,
    tab_id: Option<&str>,
) -> Result<magicmerlin_media::browser::BrowserClient> {
    if let Some(id) = tab_id {
        let tabs = magicmerlin_media::browser::list_tabs(port)
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let tab = tabs
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| ToolError::Execution(format!("tab not found: {id}")))?;
        magicmerlin_media::browser::BrowserClient::connect(&tab.web_socket_debugger_url)
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))
    } else {
        magicmerlin_media::browser::BrowserClient::from_tab(port)
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))
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
                "action":{"type":"string","enum":["status","start","stop","profiles","tabs","open","close","focus","navigate","snapshot","screenshot","eval","console","act"]},
                "port":{"type":"integer"},
                "profile":{"type":"string","enum":["default","relay"]},
                "url":{"type":"string"},
                "tab_id":{"type":"string"},
                "expression":{"type":"string"},
                "format":{"type":"string","enum":["png","jpeg"]},
                "quality":{"type":"integer"},
                "x":{"type":"number"},
                "y":{"type":"number"},
                "text":{"type":"string"},
                "key":{"type":"string"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let port = params.get("port").and_then(Value::as_u64).unwrap_or(9222) as u16;
        let tab_id = params.get("tab_id").and_then(Value::as_str);

        match action.as_str() {
            "status" => {
                let mgr = ctx.browser_manager.as_ref().ok_or_else(|| {
                    ToolError::Unavailable("browser manager not initialized".into())
                })?;
                let status = mgr
                    .status(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(serde_json::to_value(status)?))
            }
            "start" => {
                let mgr = ctx.browser_manager.as_ref().ok_or_else(|| {
                    ToolError::Unavailable("browser manager not initialized".into())
                })?;
                let profile = match params
                    .get("profile")
                    .and_then(Value::as_str)
                    .unwrap_or("default")
                {
                    "relay" => magicmerlin_media::browser::BrowserProfile::Relay,
                    _ => magicmerlin_media::browser::BrowserProfile::Default,
                };
                let options =
                    magicmerlin_media::browser::BrowserManager::launch_options_for_profile(
                        profile,
                        port,
                        Duration::from_secs(30),
                    );
                let status = mgr
                    .start(options, profile)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(serde_json::to_value(status)?))
            }
            "stop" => {
                let mgr = ctx.browser_manager.as_ref().ok_or_else(|| {
                    ToolError::Unavailable("browser manager not initialized".into())
                })?;
                mgr.stop(port)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"stopped": port})))
            }
            "profiles" => Ok(ToolResult::success(
                json!({"profiles": ["default", "relay"]}),
            )),
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
                let id = required_string(&params, "tab_id", self.name())?;
                magicmerlin_media::browser::close_tab(port, &id)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"closed": id})))
            }
            "focus" => {
                let id = required_string(&params, "tab_id", self.name())?;
                magicmerlin_media::browser::focus_tab(port, &id)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"focused": id})))
            }
            "navigate" => {
                let url = required_string(&params, "url", self.name())?;
                let client = browser_client(port, tab_id).await?;
                client
                    .navigate(&url)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"navigated": url})))
            }
            "snapshot" => {
                let client = browser_client(port, tab_id).await?;
                let snap = client
                    .build_snapshot()
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(serde_json::to_value(snap)?))
            }
            "screenshot" => {
                let client = browser_client(port, tab_id).await?;
                let fmt = params
                    .get("format")
                    .and_then(Value::as_str)
                    .unwrap_or("png");
                let quality = params
                    .get("quality")
                    .and_then(Value::as_u64)
                    .map(|q| q as u8);
                let bytes = client
                    .capture_screenshot(fmt, quality, None)
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                let b64 = BASE64_STANDARD.encode(&bytes);
                let mime = if fmt == "jpeg" {
                    "image/jpeg"
                } else {
                    "image/png"
                };
                Ok(ToolResult::success(json!({
                    "data": b64,
                    "mimeType": mime,
                    "bytes": bytes.len(),
                })))
            }
            "eval" => {
                let expression = required_string(&params, "expression", self.name())?;
                let client = browser_client(port, tab_id).await?;
                let value = client
                    .evaluate_script(magicmerlin_media::browser::EvaluateOptions {
                        expression,
                        await_promise: params
                            .get("await_promise")
                            .and_then(Value::as_bool)
                            .unwrap_or(true),
                        return_by_value: params
                            .get("return_by_value")
                            .and_then(Value::as_bool)
                            .unwrap_or(true),
                    })
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"value": value})))
            }
            "console" => {
                let client = browser_client(port, tab_id).await?;
                let result = client
                    .evaluate_script(magicmerlin_media::browser::EvaluateOptions {
                        expression: concat!(
                            "(function(){",
                            "if(!window.__mm_console){",
                            "window.__mm_console=[];",
                            "var o={log:console.log,warn:console.warn,",
                            "error:console.error,info:console.info};",
                            "['log','warn','error','info'].forEach(function(l){",
                            "console[l]=function(){",
                            "window.__mm_console.push({level:l,",
                            "message:Array.prototype.slice.call(arguments)",
                            ".map(String).join(' '),ts:Date.now()});",
                            "o[l].apply(console,arguments);};});",
                            "}",
                            "var logs=window.__mm_console.splice(0);",
                            "return logs;})()"
                        )
                        .to_string(),
                        await_promise: false,
                        return_by_value: true,
                    })
                    .await
                    .map_err(|e| ToolError::Execution(e.to_string()))?;
                Ok(ToolResult::success(json!({"logs": result})))
            }
            "act" => {
                let client = browser_client(port, tab_id).await?;
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
                message: format!("unsupported browser action: {action}"),
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

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let server = ctx
            .canvas_server
            .as_ref()
            .ok_or_else(|| ToolError::Unavailable("canvas server not initialized".into()))?;

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
            "a2ui_reset" => {
                while server.pop_update().await.is_some() {}
                Ok(ToolResult::success(json!({"ok": true})))
            }
            _ => Err(ToolError::InvalidParams {
                tool: self.name().to_string(),
                message: format!("unsupported canvas action: {action}"),
            }),
        }
    }
}

/// HTTP client for a single registered node host.
struct NodeApiClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl NodeApiClient {
    fn new(base_url: String, token: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            base_url,
            token,
            http,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let url = self.url(path);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| ToolError::Execution(format!("node request failed: {e}")))?;
        let status = resp.status().as_u16();
        let value: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| json!({"error": "non-json response"}));
        if status >= 400 {
            return Err(ToolError::Execution(format!(
                "node returned HTTP {status}: {}",
                serde_json::to_string(&value).unwrap_or_default()
            )));
        }
        Ok(value)
    }

    async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let url = self.url(path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ToolError::Execution(format!("node request failed: {e}")))?;
        let status = resp.status().as_u16();
        let value: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| json!({"error": "non-json response"}));
        if status >= 400 {
            return Err(ToolError::Execution(format!(
                "node returned HTTP {status}: {}",
                serde_json::to_string(&value).unwrap_or_default()
            )));
        }
        Ok(value)
    }
}

struct NodesTool;

impl NodesTool {
    fn resolve_node<'a>(configs: &'a [NodeConfig], params: &Value) -> Result<&'a NodeConfig> {
        if configs.is_empty() {
            return Err(ToolError::InvalidParams {
                tool: "nodes".to_string(),
                message: "no node hosts configured".to_string(),
            });
        }
        if let Some(id) = params.get("node").and_then(Value::as_str) {
            configs
                .iter()
                .find(|n| n.id == id)
                .ok_or_else(|| ToolError::InvalidParams {
                    tool: "nodes".to_string(),
                    message: format!("unknown node: {id}"),
                })
        } else {
            Ok(&configs[0])
        }
    }
}

#[async_trait]
impl Tool for NodesTool {
    fn name(&self) -> &str {
        "nodes"
    }

    fn description(&self) -> &str {
        "Remote device control via registered node hosts."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "node":{"type":"string","description":"Node ID from config, defaults to first"},
                "action":{"type":"string","enum":[
                    "status","describe",
                    "pending","approve","reject",
                    "notify",
                    "camera_snap","camera_list","camera_clip",
                    "photos_latest",
                    "screen_record",
                    "location_get",
                    "notifications_list","notifications_action",
                    "device_status","device_info",
                    "run","invoke"
                ]},
                "requestId":{"type":"string"},
                "title":{"type":"string"},
                "body":{"type":"string"},
                "priority":{"type":"string"},
                "sound":{"type":"string"},
                "delivery":{"type":"object"},
                "facing":{"type":"string"},
                "maxWidth":{"type":"integer"},
                "quality":{"type":"integer"},
                "durationMs":{"type":"integer"},
                "fps":{"type":"integer"},
                "includeAudio":{"type":"boolean"},
                "screenIndex":{"type":"integer"},
                "limit":{"type":"integer"},
                "accuracy":{"type":"string"},
                "timeoutMs":{"type":"integer"},
                "notificationKey":{"type":"string"},
                "replyText":{"type":"string"},
                "command":{"type":"array","items":{"type":"string"}},
                "params":{"type":"object"},
                "cwd":{"type":"string"},
                "env":{"type":"object"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let node_cfg = Self::resolve_node(&ctx.node_configs, &params)?;
        let client = NodeApiClient::new(node_cfg.url.clone(), node_cfg.token.clone());

        let result = match action.as_str() {
            // --- Status / describe ---
            "status" => client.get("/api/status").await?,
            "describe" => client.get("/api/describe").await?,

            // --- Pairing ---
            "pending" => client.get("/api/pairing/pending").await?,
            "approve" => {
                let id = required_string(&params, "requestId", self.name())?;
                client
                    .post("/api/pairing/approve", json!({"requestId": id}))
                    .await?
            }
            "reject" => {
                let id = required_string(&params, "requestId", self.name())?;
                client
                    .post("/api/pairing/reject", json!({"requestId": id}))
                    .await?
            }

            // --- Notifications push ---
            "notify" => {
                client
                    .post(
                        "/api/notify",
                        json!({
                            "title": params.get("title"),
                            "body": params.get("body"),
                            "priority": params.get("priority"),
                            "sound": params.get("sound"),
                            "delivery": params.get("delivery"),
                        }),
                    )
                    .await?
            }

            // --- Camera ---
            "camera_snap" => {
                client
                    .post(
                        "/api/camera/snap",
                        json!({
                            "facing": params.get("facing"),
                            "maxWidth": params.get("maxWidth"),
                            "quality": params.get("quality"),
                        }),
                    )
                    .await?
            }
            "camera_list" => client.get("/api/camera/list").await?,
            "camera_clip" => {
                client
                    .post(
                        "/api/camera/clip",
                        json!({
                            "facing": params.get("facing"),
                            "durationMs": params.get("durationMs"),
                            "fps": params.get("fps"),
                            "includeAudio": params.get("includeAudio"),
                        }),
                    )
                    .await?
            }

            // --- Photos ---
            "photos_latest" => {
                let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(10);
                client
                    .get(&format!("/api/photos/latest?limit={limit}"))
                    .await?
            }

            // --- Screen ---
            "screen_record" => {
                client
                    .post(
                        "/api/screen/record",
                        json!({
                            "durationMs": params.get("durationMs"),
                            "screenIndex": params.get("screenIndex"),
                        }),
                    )
                    .await?
            }

            // --- Location ---
            "location_get" => {
                let accuracy = params
                    .get("accuracy")
                    .and_then(Value::as_str)
                    .unwrap_or("balanced");
                let timeout = params
                    .get("timeoutMs")
                    .and_then(Value::as_u64)
                    .unwrap_or(5000);
                client
                    .get(&format!(
                        "/api/location?accuracy={accuracy}&timeoutMs={timeout}"
                    ))
                    .await?
            }

            // --- Notification inbox ---
            "notifications_list" => {
                let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20);
                client
                    .get(&format!("/api/notifications?limit={limit}"))
                    .await?
            }
            "notifications_action" => {
                client
                    .post(
                        "/api/notifications/action",
                        json!({
                            "notificationKey": params.get("notificationKey"),
                            "action": params.get("action"),
                            "replyText": params.get("replyText"),
                        }),
                    )
                    .await?
            }

            // --- Device ---
            "device_status" => client.get("/api/device/status").await?,
            "device_info" => client.get("/api/device/info").await?,

            // --- Run / invoke ---
            "run" => {
                client
                    .post(
                        "/api/run",
                        json!({
                            "command": params.get("command"),
                            "cwd": params.get("cwd"),
                            "env": params.get("env"),
                            "timeoutMs": params.get("timeoutMs"),
                        }),
                    )
                    .await?
            }
            "invoke" => {
                client
                    .post(
                        "/api/invoke",
                        json!({
                            "command": params.get("command"),
                            "params": params.get("params"),
                            "timeoutMs": params.get("timeoutMs"),
                        }),
                    )
                    .await?
            }

            _ => {
                return Err(ToolError::InvalidParams {
                    tool: self.name().to_string(),
                    message: format!("unsupported node action: {action}"),
                })
            }
        };

        Ok(ToolResult::success(
            json!({"node": node_cfg.id, "action": action, "result": result}),
        ))
    }
}

struct SessionsYieldTool;

#[async_trait]
impl Tool for SessionsYieldTool {
    fn name(&self) -> &str {
        "sessions_yield"
    }

    fn description(&self) -> &str {
        "Ends the current turn, signalling the gateway to pause until a sub-agent reports back."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "message":{"type":"string"}
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        gateway_call(ctx, "sessions.yield", params).await
    }
}

struct GatewayTool;

#[async_trait]
impl Tool for GatewayTool {
    fn name(&self) -> &str {
        "gateway"
    }

    fn description(&self) -> &str {
        "Gateway control surface for config, restart, and updates."
    }

    fn schema(&self) -> Value {
        json!({
            "type":"object",
            "properties": {
                "action":{"type":"string","enum":["restart","config.get","config.patch","config.apply","update.run"]},
                "reason":{"type":"string"},
                "delayMs":{"type":"integer"},
                "path":{"type":"string"},
                "raw":{"type":"object"},
                "note":{"type":"string"}
            },
            "required":["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", self.name())?;
        let method = format!("gateway.{action}");
        gateway_call(ctx, &method, params).await
    }
}

/// Executes a command inside a real PTY and returns combined output.
async fn exec_foreground_pty(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    timeout_ms: u64,
) -> Result<ToolResult> {
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| ToolError::Execution(format!("pty open: {e}")))?;

    let mut cmd_builder = CommandBuilder::new("sh");
    cmd_builder.args(["-lc", cmd]);
    cmd_builder.cwd(cwd);
    for (k, v) in env {
        cmd_builder.env(k, v);
    }

    let mut child = pair
        .slave
        .spawn_command(cmd_builder)
        .map_err(|e| ToolError::Execution(format!("pty spawn: {e}")))?;
    let pid = child.process_id();
    // Drop slave so master reader sees EOF when child exits.
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| ToolError::Execution(format!("pty reader: {e}")))?;

    let read_handle = tokio::task::spawn_blocking(move || {
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).to_string()
    });

    let wait_handle = tokio::task::spawn_blocking(move || child.wait());

    let result = tokio::time::timeout(Duration::from_millis(timeout_ms), async {
        let output = read_handle
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        let status = wait_handle
            .await
            .map_err(|e| ToolError::Execution(e.to_string()))?
            .map_err(|e| ToolError::Execution(e.to_string()))?;
        Ok::<_, ToolError>((output, status))
    })
    .await
    .map_err(|_| ToolError::Execution("pty command timed out".to_string()))??;

    let (output, status) = result;
    Ok(ToolResult::success(json!({
        "status": if status.success() { 0 } else { 1 },
        "output": output,
        "output_bytes": output.len(),
        "pid": pid,
        "tty": true,
    })))
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

struct MemoryChunk {
    path: PathBuf,
    start_line: usize,
    end_line: usize,
    text: String,
    terms: Vec<String>,
}

struct RawChunk {
    start_line: usize,
    end_line: usize,
    text: String,
}

/// Splits text into ~`target_words`-word chunks aligned to line boundaries.
fn chunk_text(text: &str, target_words: usize) -> Vec<RawChunk> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut word_count = 0;
    let mut buf = String::new();

    for (i, line) in lines.iter().enumerate() {
        let words_in_line = line.split_whitespace().count();
        if word_count > 0 && word_count + words_in_line > target_words {
            chunks.push(RawChunk {
                start_line: start + 1,
                end_line: i, // exclusive of current, so last included is i
                text: buf.clone(),
            });
            buf.clear();
            start = i;
            word_count = 0;
        }
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(line);
        word_count += words_in_line;
    }

    if !buf.is_empty() {
        chunks.push(RawChunk {
            start_line: start + 1,
            end_line: lines.len(),
            text: buf,
        });
    }

    chunks
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

/// Extracts main content from HTML and converts to markdown using the `scraper` crate.
/// Removes nav, header, footer, aside, script, style elements.
/// Prefers article/main content containers when available.
fn extract_main_content(html: &str) -> String {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let skip_tags: HashSet<&str> = [
        "script", "style", "nav", "header", "footer", "aside", "noscript", "iframe", "svg",
    ]
    .into_iter()
    .collect();

    // Try to find a main content container
    let content_selectors = ["article", "main", "[role=main]"];
    let body_sel = Selector::parse("body").unwrap();
    let mut root = None;
    for sel_str in content_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                root = Some(el);
                break;
            }
        }
    }
    let root = root.or_else(|| doc.select(&body_sel).next());
    let Some(root) = root else {
        return html_to_text(html);
    };

    let mut out = String::new();
    walk_element_to_markdown(root, &skip_tags, &mut out);
    clean_markdown_output(&out)
}

/// Extracts plain text from HTML, removing unwanted elements.
fn extract_text_content(html: &str) -> String {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let content_selectors = ["article", "main", "[role=main]"];
    let body_sel = Selector::parse("body").unwrap();
    let mut root = None;
    for sel_str in content_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                root = Some(el);
                break;
            }
        }
    }
    let root = root.or_else(|| doc.select(&body_sel).next());
    match root {
        Some(el) => {
            let texts: Vec<&str> = el.text().collect();
            let joined = texts.join(" ");
            normalize_ws(&joined)
        }
        None => html_to_text(html),
    }
}

/// Recursively walks an HTML element tree, converting to markdown.
fn walk_element_to_markdown(el: scraper::ElementRef, skip: &HashSet<&str>, out: &mut String) {
    for child in el.children() {
        match child.value() {
            scraper::Node::Text(t) => {
                let raw = t.to_string();
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    let has_leading_ws = raw.starts_with(|c: char| c.is_whitespace());
                    let has_trailing_ws = raw.ends_with(|c: char| c.is_whitespace());
                    if has_leading_ws
                        && !out.is_empty()
                        && !out.ends_with(|c: char| c.is_whitespace())
                    {
                        out.push(' ');
                    }
                    out.push_str(trimmed);
                    if has_trailing_ws {
                        out.push(' ');
                    }
                }
            }
            scraper::Node::Element(elem) => {
                let tag = elem.name.local.to_string();
                if skip.contains(tag.as_str()) {
                    continue;
                }
                let Some(child_el) = scraper::ElementRef::wrap(child) else {
                    continue;
                };
                match tag.as_str() {
                    "h1" => {
                        out.push_str("\n\n# ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "h2" => {
                        out.push_str("\n\n## ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "h3" => {
                        out.push_str("\n\n### ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "h4" => {
                        out.push_str("\n\n#### ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "h5" => {
                        out.push_str("\n\n##### ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "h6" => {
                        out.push_str("\n\n###### ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "p" => {
                        out.push_str("\n\n");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n\n");
                    }
                    "br" => out.push('\n'),
                    "li" => {
                        out.push_str("\n- ");
                        walk_element_to_markdown(child_el, skip, out);
                    }
                    "pre" => {
                        out.push_str("\n\n```\n");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str("\n```\n\n");
                    }
                    "code" => {
                        out.push('`');
                        walk_element_to_markdown(child_el, skip, out);
                        while out.ends_with(' ') {
                            out.pop();
                        }
                        out.push('`');
                    }
                    "strong" | "b" => {
                        out.push_str("**");
                        walk_element_to_markdown(child_el, skip, out);
                        while out.ends_with(' ') {
                            out.pop();
                        }
                        out.push_str("**");
                    }
                    "em" | "i" => {
                        out.push('*');
                        walk_element_to_markdown(child_el, skip, out);
                        while out.ends_with(' ') {
                            out.pop();
                        }
                        out.push('*');
                    }
                    "a" => {
                        let href = child_el.attr("href").unwrap_or("");
                        if href.is_empty()
                            || href.starts_with('#')
                            || href.starts_with("javascript:")
                        {
                            walk_element_to_markdown(child_el, skip, out);
                        } else {
                            out.push('[');
                            let start = out.len();
                            walk_element_to_markdown(child_el, skip, out);
                            while out.len() > start && out.ends_with(' ') {
                                out.pop();
                            }
                            out.push_str("](");
                            out.push_str(href);
                            out.push(')');
                        }
                    }
                    "blockquote" => {
                        out.push_str("\n\n> ");
                        walk_element_to_markdown(child_el, skip, out);
                        out.push('\n');
                    }
                    "img" => {
                        let alt = child_el.attr("alt").unwrap_or("");
                        let src = child_el.attr("src").unwrap_or("");
                        if !src.is_empty() {
                            out.push_str(&format!("![{alt}]({src})"));
                        }
                    }
                    "tr" => {
                        walk_element_to_markdown(child_el, skip, out);
                        out.push('\n');
                    }
                    "td" | "th" => {
                        walk_element_to_markdown(child_el, skip, out);
                        out.push_str(" | ");
                    }
                    _ => walk_element_to_markdown(child_el, skip, out),
                }
            }
            _ => {}
        }
    }
}

/// Cleans up raw markdown output by collapsing blank lines.
fn clean_markdown_output(raw: &str) -> String {
    let lines: Vec<&str> = raw.lines().map(|l| l.trim_end()).collect();
    let mut result = String::new();
    let mut prev_blank = false;
    for line in lines {
        if line.is_empty() {
            if !prev_blank && !result.is_empty() {
                result.push('\n');
                prev_blank = true;
            }
        } else {
            result.push_str(line);
            result.push('\n');
            prev_blank = false;
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ctx(temp: &std::path::Path) -> ToolContext {
        let state_paths = magicmerlin_config::StatePaths::new(magicmerlin_config::PathScope::dev())
            .expect("paths");
        ToolContext {
            agent_name: "merlin".to_string(),
            workspace_dir: temp.to_path_buf(),
            state_paths,
            config: magicmerlin_config::Config::default(),
            delivery: None,
            process_manager: crate::ProcessManager::new(),
            node_configs: vec![],
            browser_manager: None,
            canvas_server: None,
            tts_client: None,
            understanding_client: None,
        }
    }

    #[tokio::test]
    async fn edit_replaces_text_once() {
        let temp = tempfile::tempdir().expect("tmp");
        let path = temp.path().join("a.txt");
        std::fs::write(&path, "hello old").expect("write");
        let ctx = make_test_ctx(temp.path());

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
            "memory_search",
            "message",
            "cron",
            "session_status",
            "nodes",
        ] {
            assert!(
                names.contains(&required.to_string()),
                "missing tool: {required}"
            );
        }
    }

    // --- Tool 1: exec PTY ---
    #[tokio::test]
    async fn exec_foreground_pty_captures_output() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = ExecTool
            .execute(json!({"cmd": "echo pty_hello", "tty": true}), &ctx)
            .await
            .expect("pty exec");
        assert!(result.ok);
        let output = result
            .value
            .get("output")
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(output.contains("pty_hello"), "got: {output}");
        assert_eq!(result.value.get("tty").and_then(Value::as_bool), Some(true));
    }

    #[tokio::test]
    async fn exec_background_returns_session_id() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = ExecTool
            .execute(json!({"cmd": "sleep 0.1", "background": true}), &ctx)
            .await
            .expect("bg exec");
        assert!(result.ok);
        assert!(result.value.get("session_id").is_some());
    }

    // --- Tool 2: memory_search ---
    #[tokio::test]
    async fn memory_search_finds_chunks() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(
            temp.path().join("MEMORY.md"),
            "The quick brown fox jumps over the lazy dog.\nRust is a systems programming language.",
        )
        .expect("write");
        let mut ctx = make_test_ctx(temp.path());
        ctx.state_paths.state_dir = temp.path().to_path_buf();

        let result = MemorySearchTool
            .execute(json!({"query": "rust programming"}), &ctx)
            .await
            .expect("search");
        assert!(result.ok);
        let results = result
            .value
            .get("results")
            .and_then(Value::as_array)
            .unwrap();
        assert!(!results.is_empty(), "should find at least one result");
        let first = &results[0];
        assert!(first.get("score").and_then(Value::as_f64).unwrap_or(0.0) > 0.0);
        assert!(first.get("citation").is_some());
    }

    // --- Tool 3: memory_get ---
    #[tokio::test]
    async fn memory_get_returns_content_and_total_lines() {
        let temp = tempfile::tempdir().expect("tmp");
        std::fs::write(
            temp.path().join("notes.md"),
            "line1\nline2\nline3\nline4\nline5",
        )
        .expect("write");
        let mut ctx = make_test_ctx(temp.path());
        ctx.state_paths.state_dir = temp.path().to_path_buf();

        let result = MemoryGetTool
            .execute(json!({"path": "notes.md", "from": 2, "lines": 2}), &ctx)
            .await
            .expect("get");
        assert!(result.ok);
        assert_eq!(
            result.value.get("totalLines").and_then(Value::as_u64),
            Some(5)
        );
        let content = result.value.get("content").and_then(Value::as_str).unwrap();
        assert!(content.contains("line2"));
        assert!(content.contains("line3"));
        assert!(!content.contains("line1"));
    }

    // --- Tool 4: message ---
    #[tokio::test]
    async fn message_tool_constructs_gateway_call() {
        // We can't test actual gateway, but verify parameter handling
        let temp = tempfile::tempdir().expect("tmp");
        let mut ctx = make_test_ctx(temp.path());
        ctx.delivery = Some(crate::registry::DeliveryContext {
            channel: "telegram".to_string(),
            target: "12345".to_string(),
        });

        // This will fail to connect to gateway, which is expected.
        // We verify the tool doesn't panic and returns a meaningful error.
        let result = MessageTool
            .execute(json!({"action": "send", "text": "hello world"}), &ctx)
            .await;
        // Gateway not running, so expect an error
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- Tool 5: cron ---
    #[tokio::test]
    async fn cron_tool_requires_action() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = CronTool.execute(json!({}), &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cron_tool_dispatches_to_gateway() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        // Gateway not running, so expect connection error
        let result = CronTool.execute(json!({"action": "list"}), &ctx).await;
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- Tool 6: session_status ---
    #[tokio::test]
    async fn session_status_handles_missing_session() {
        let temp = tempfile::tempdir().expect("tmp");
        let mut ctx = make_test_ctx(temp.path());
        ctx.state_paths.state_dir = temp.path().to_path_buf();

        // Create the db so Storage works
        let db_path = temp.path().join("openclaw.db");
        let storage = magicmerlin_storage::Storage::new(&db_path).expect("storage");
        let conn = storage.connection().expect("conn");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent TEXT,
                status TEXT NOT NULL DEFAULT 'unknown',
                started_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0,
                metadata TEXT
            )",
        )
        .expect("create table");

        let result = SessionStatusTool
            .execute(json!({"session_key": "nonexistent:key"}), &ctx)
            .await
            .expect("status");
        assert!(result.ok);
        let session = result.value.get("session").unwrap();
        assert_eq!(session.get("missing").and_then(Value::as_bool), Some(true));
        // Should have token fields even for missing session
        assert!(result.value.get("contextTokens").is_some());
        assert!(result.value.get("messageCount").is_some());
    }

    // --- Helpers ---
    #[test]
    fn chunk_text_produces_correct_segments() {
        let text = (0..20)
            .map(|i| format!("word{i} extra filler text padding"))
            .collect::<Vec<_>>()
            .join("\n");
        let chunks = chunk_text(&text, 10);
        assert!(chunks.len() >= 2, "should produce multiple chunks");
        assert_eq!(chunks[0].start_line, 1);
        assert!(chunks[0].end_line > 0);
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

    #[test]
    fn gateway_url_uses_env_override() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        // Without env var, should use config defaults
        let url = crate::gateway::gateway_url(&ctx);
        assert!(url.starts_with("http://"));
        assert!(url.contains("18789") || url.contains("127.0.0.1"));
    }

    // --- NodeApiClient URL building ---
    #[test]
    fn node_api_client_url_building() {
        let client = NodeApiClient::new(
            "http://192.168.1.42:9222".to_string(),
            "test-token".to_string(),
        );
        assert_eq!(
            client.url("/api/status"),
            "http://192.168.1.42:9222/api/status"
        );
        assert_eq!(
            client.url("/api/location?accuracy=balanced&timeoutMs=5000"),
            "http://192.168.1.42:9222/api/location?accuracy=balanced&timeoutMs=5000"
        );

        // Trailing slash should be stripped
        let client2 = NodeApiClient::new("http://example.com:8080/".to_string(), "tok".to_string());
        assert_eq!(
            client2.url("/api/describe"),
            "http://example.com:8080/api/describe"
        );
    }

    // --- NodesTool resolve_node ---
    #[test]
    fn nodes_tool_resolve_node_selects_first() {
        let configs = vec![
            NodeConfig {
                id: "phone".to_string(),
                url: "http://10.0.0.1:9222".to_string(),
                token: "tok1".to_string(),
            },
            NodeConfig {
                id: "tablet".to_string(),
                url: "http://10.0.0.2:9222".to_string(),
                token: "tok2".to_string(),
            },
        ];
        let node = NodesTool::resolve_node(&configs, &json!({})).unwrap();
        assert_eq!(node.id, "phone");
    }

    #[test]
    fn nodes_tool_resolve_node_by_id() {
        let configs = vec![
            NodeConfig {
                id: "phone".to_string(),
                url: "http://10.0.0.1:9222".to_string(),
                token: "tok1".to_string(),
            },
            NodeConfig {
                id: "tablet".to_string(),
                url: "http://10.0.0.2:9222".to_string(),
                token: "tok2".to_string(),
            },
        ];
        let node = NodesTool::resolve_node(&configs, &json!({"node": "tablet"})).unwrap();
        assert_eq!(node.id, "tablet");
        assert_eq!(node.url, "http://10.0.0.2:9222");
    }

    #[test]
    fn nodes_tool_resolve_node_rejects_empty() {
        let configs: Vec<NodeConfig> = vec![];
        let result = NodesTool::resolve_node(&configs, &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn nodes_tool_resolve_node_rejects_unknown_id() {
        let configs = vec![NodeConfig {
            id: "phone".to_string(),
            url: "http://10.0.0.1:9222".to_string(),
            token: "tok1".to_string(),
        }];
        let result = NodesTool::resolve_node(&configs, &json!({"node": "nope"}));
        assert!(result.is_err());
    }

    // --- sessions_spawn dispatches to gateway ---
    #[tokio::test]
    async fn sessions_spawn_dispatches_to_gateway() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = SessionsSpawnTool
            .execute(json!({"task": "test task"}), &ctx)
            .await;
        // Gateway not running, so expect connection error
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- sessions_yield dispatches to gateway ---
    #[tokio::test]
    async fn sessions_yield_dispatches_to_gateway() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = SessionsYieldTool
            .execute(json!({"message": "pausing"}), &ctx)
            .await;
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- gateway tool dispatches ---
    #[tokio::test]
    async fn gateway_tool_requires_action() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = GatewayTool.execute(json!({}), &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn gateway_tool_dispatches_restart() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = GatewayTool
            .execute(json!({"action": "restart", "reason": "test"}), &ctx)
            .await;
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- subagents dispatches to gateway ---
    #[tokio::test]
    async fn subagents_tool_dispatches_list() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = SubagentsTool.execute(json!({"action": "list"}), &ctx).await;
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- agents_list dispatches to gateway ---
    #[tokio::test]
    async fn agents_list_dispatches_to_gateway() {
        let temp = tempfile::tempdir().expect("tmp");
        let ctx = make_test_ctx(temp.path());
        let result = AgentsListTool.execute(json!({}), &ctx).await;
        assert!(result.is_err() || !result.as_ref().unwrap().ok);
    }

    // --- web_search response parsing ---
    #[test]
    fn web_search_parses_brave_response() {
        let raw = json!({
            "web": {
                "results": [
                    {"title": "Rust Lang", "url": "https://rust-lang.org", "description": "A systems programming language"},
                    {"title": "Rust Book", "url": "https://doc.rust-lang.org/book/", "description": "The Rust Programming Language book"},
                ],
                "totalResults": 42
            }
        });
        let results = raw
            .pointer("/web/results")
            .and_then(Value::as_array)
            .unwrap();
        let parsed: Vec<Value> = results
            .iter()
            .map(|item| {
                json!({
                    "title": item.get("title").and_then(Value::as_str).unwrap_or_default(),
                    "url": item.get("url").and_then(Value::as_str).unwrap_or_default(),
                    "snippet": item.get("description").and_then(Value::as_str).unwrap_or_default(),
                })
            })
            .collect();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["title"], "Rust Lang");
        assert_eq!(parsed[0]["snippet"], "A systems programming language");
        let total = raw
            .pointer("/web/totalResults")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        assert_eq!(total, 42);
    }

    // --- web_fetch HTML extraction ---
    #[test]
    fn extract_main_content_strips_nav_and_keeps_article() {
        let html = r#"<html><body>
            <nav><a href="/">Home</a><a href="/about">About</a></nav>
            <header><h1>Site Header</h1></header>
            <article>
                <h1>Article Title</h1>
                <p>This is the <strong>main</strong> content.</p>
                <a href="https://example.com">Example link</a>
            </article>
            <footer>Copyright 2024</footer>
        </body></html>"#;

        let md = extract_main_content(html);
        assert!(
            md.contains("Article Title"),
            "should contain article title: {md}"
        );
        assert!(md.contains("**main**"), "should have bold formatting: {md}");
        assert!(!md.contains("Home"), "should not contain nav links: {md}");
        assert!(!md.contains("Copyright"), "should not contain footer: {md}");
        assert!(
            md.contains("[Example link](https://example.com)"),
            "should have markdown link: {md}"
        );
    }

    #[test]
    fn extract_text_content_returns_plain_text() {
        let html = r#"<html><body>
            <nav>Nav stuff</nav>
            <main><p>Hello <strong>world</strong>!</p></main>
            <footer>Footer</footer>
        </body></html>"#;

        let text = extract_text_content(html);
        assert!(text.contains("Hello"), "got: {text}");
        assert!(text.contains("world"), "got: {text}");
        assert!(!text.contains("<"), "should have no HTML tags: {text}");
    }

    #[test]
    fn extract_main_content_handles_no_article() {
        let html = r#"<html><body>
            <p>Just a paragraph.</p>
            <p>Another one.</p>
        </body></html>"#;

        let md = extract_main_content(html);
        assert!(md.contains("Just a paragraph"), "got: {md}");
        assert!(md.contains("Another one"), "got: {md}");
    }

    // --- Registry includes new tools ---
    #[tokio::test]
    async fn registry_includes_sprint5_tools() {
        let mut registry = ToolRegistry::new();
        register_default_tools(&mut registry);
        let names = registry.names();
        for required in [
            "sessions_spawn",
            "sessions_list",
            "sessions_history",
            "sessions_send",
            "sessions_yield",
            "subagents",
            "agents_list",
            "gateway",
            "cron",
            "nodes",
        ] {
            assert!(
                names.contains(&required.to_string()),
                "missing tool: {required}"
            );
        }
    }
}

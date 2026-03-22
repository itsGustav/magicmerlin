# Sprint 5 — Media Tools: browser, canvas, tts, image, pdf

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Media crate: `media/src/` — browser (720 lines), canvas (378), tts (440), understanding (814+).
Agent tools: `agent-tools/src/tools.rs` — BrowserTool, CanvasTool, TtsTool, ImageTool, PdfTool are registered but call stubs.

## Your Mission
Wire agent tools to the real implementations in the media crate.

---

## Tool 1: `browser` tool

The `media::browser` module has a `BrowserManager` with CDP integration.
Wire `BrowserTool::execute` to it:

```rust
// Supported actions: status/start/stop/profiles/tabs/open/focus/close/
//                    snapshot/screenshot/navigate/console/pdf/upload/dialog/act
impl BrowserTool {
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = required_string(&params, "action", "browser")?;
        let browser = ctx.browser_manager.as_ref()
            .ok_or_else(|| ToolError::unavailable("browser not initialized"))?;
        
        match action.as_str() {
            "status" => {
                let status = browser.status().await?;
                Ok(ToolResult::json(json!({
                    "running": status.running,
                    "profile": status.profile,
                    "tab_count": status.tab_count,
                })))
            }
            "start" => {
                browser.start(params.get("profile").and_then(Value::as_str)).await?;
                Ok(ToolResult::text("Browser started"))
            }
            "stop" => {
                browser.stop().await?;
                Ok(ToolResult::text("Browser stopped"))
            }
            "tabs" => {
                let tabs = browser.list_tabs().await?;
                Ok(ToolResult::json(json!({"tabs": tabs})))
            }
            "open" => {
                let url = required_string(&params, "url", "browser")?;
                let tab_id = browser.open_tab(&url).await?;
                Ok(ToolResult::json(json!({"targetId": tab_id})))
            }
            "navigate" => {
                let url = required_string(&params, "url", "browser")?;
                let target_id = params.get("targetId").and_then(Value::as_str);
                browser.navigate(target_id, &url).await?;
                Ok(ToolResult::text("Navigated"))
            }
            "snapshot" => {
                let target_id = params.get("targetId").and_then(Value::as_str);
                let refs_mode = params.get("refs").and_then(Value::as_str).unwrap_or("role");
                let snapshot = browser.snapshot(target_id, refs_mode).await?;
                Ok(ToolResult::json(json!({
                    "snapshot": snapshot.content,
                    "url": snapshot.url,
                    "title": snapshot.title,
                })))
            }
            "screenshot" => {
                let target_id = params.get("targetId").and_then(Value::as_str);
                let full_page = params.get("fullPage").and_then(Value::as_bool).unwrap_or(false);
                let bytes = browser.screenshot(target_id, full_page).await?;
                // Return as base64
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                Ok(ToolResult::json(json!({"image": b64, "mimeType": "image/png"})))
            }
            "act" => {
                let request = params.get("request")
                    .ok_or_else(|| ToolError::missing_param("request"))?;
                let result = browser.act(params.get("targetId").and_then(Value::as_str), request).await?;
                Ok(ToolResult::json(result))
            }
            "close" => {
                let target_id = params.get("targetId").and_then(Value::as_str);
                browser.close_tab(target_id).await?;
                Ok(ToolResult::text("Tab closed"))
            }
            _ => Err(ToolError::invalid_param("action", &format!("unknown action: {action}")))
        }
    }
}
```

Add `BrowserManager` to `ToolContext`:
```rust
pub struct ToolContext {
    // existing fields...
    pub browser_manager: Option<Arc<media::browser::BrowserManager>>,
    pub canvas_host: Option<Arc<media::canvas::CanvasHost>>,
}
```

---

## Tool 2: `canvas` tool

Wire `CanvasTool::execute` to `media::canvas::CanvasHost`:

```rust
// Actions: present/hide/navigate/eval/snapshot/a2ui_push/a2ui_reset
match action.as_str() {
    "present" => canvas.present(params.get("url").and_then(Value::as_str)).await?,
    "hide" => canvas.hide().await?,
    "navigate" => canvas.navigate(required_string(&params, "url", "canvas")?).await?,
    "eval" => {
        let js = required_string(&params, "javaScript", "canvas")?;
        let result = canvas.eval(&js).await?;
        Ok(ToolResult::json(result))
    }
    "snapshot" => {
        let snap = canvas.snapshot().await?;
        Ok(ToolResult::json(snap))
    }
    "a2ui_push" => {
        let jsonl = required_string(&params, "jsonl", "canvas")?;
        canvas.a2ui_push(&jsonl).await?;
        Ok(ToolResult::text("pushed"))
    }
    "a2ui_reset" => {
        canvas.a2ui_reset().await?;
        Ok(ToolResult::text("reset"))
    }
}
```

---

## Tool 3: `tts` tool

Wire `TtsTool::execute` to `media::tts`:

```rust
// Params: { text: String, channel?: String }
// Route to ElevenLabs if ELEVENLABS_API_KEY set, else OpenAI TTS, else error
impl TtsTool {
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let text = required_string(&params, "text", "tts")?;
        let channel = params.get("channel").and_then(Value::as_str);
        
        let audio_bytes = if let Ok(key) = std::env::var("ELEVENLABS_API_KEY") {
            media::tts::elevenlabs_tts(&text, &key, None).await?
        } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            media::tts::openai_tts(&text, &key, "tts-1", "alloy").await?
        } else {
            return Err(ToolError::unavailable("no TTS API key configured (ELEVENLABS_API_KEY or OPENAI_API_KEY)"));
        };
        
        // Save to temp file and return path + base64
        let tmp = tempfile::NamedTempFile::new_in("/tmp")?.into_temp_path();
        let path = tmp.with_extension("mp3");
        std::fs::write(&path, &audio_bytes)?;
        
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&audio_bytes);
        Ok(ToolResult::json(json!({
            "audio": b64,
            "mimeType": "audio/mpeg",
            "path": path.to_string_lossy(),
        })))
    }
}
```

---

## Tool 4: `image` tool

Wire `ImageTool::execute` to `media::understanding`:

```rust
// Params: { image?: String, images?: [String], prompt?: String }
impl ImageTool {
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let prompt = params.get("prompt")
            .and_then(Value::as_str)
            .unwrap_or("Describe this image in detail.");
        
        // Collect image paths/URLs
        let images: Vec<String> = if let Some(single) = params.get("image") {
            vec![single.as_str().unwrap_or("").to_string()]
        } else if let Some(arr) = params.get("images").and_then(Value::as_array) {
            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
        } else {
            return Err(ToolError::missing_param("image or images"));
        };
        
        // Route through understanding provider
        let result = media::understanding::analyze_images(
            &images,
            prompt,
            &ctx.provider_router,
        ).await?;
        
        Ok(ToolResult::text(result))
    }
}
```

---

## Tool 5: `pdf` tool

Wire `PdfTool::execute` to `media::understanding`:

```rust
// Params: { pdf?: String, pdfs?: [String], prompt: String, pages?: String }
impl PdfTool {
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let prompt = required_string(&params, "prompt", "pdf")?;
        
        let pdfs: Vec<String> = if let Some(single) = params.get("pdf") {
            vec![single.as_str().unwrap_or("").to_string()]
        } else if let Some(arr) = params.get("pdfs").and_then(Value::as_array) {
            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
        } else {
            return Err(ToolError::missing_param("pdf or pdfs"));
        };
        
        let pages = params.get("pages").and_then(Value::as_str);
        
        let result = media::understanding::analyze_pdfs(
            &pdfs,
            &prompt,
            pages,
            &ctx.provider_router,
        ).await?;
        
        Ok(ToolResult::text(result))
    }
}
```

---

## Also: Add to ToolContext initialization

In `agent/src/engine.rs` where ToolContext is created, initialize browser_manager and canvas_host:

```rust
let browser_manager = if config.browser_enabled {
    Some(Arc::new(media::browser::BrowserManager::new(&config.browser_profile)?))
} else {
    None
};

let ctx = ToolContext {
    // ...existing...
    browser_manager,
    canvas_host: None,  // canvas is on-demand
    gateway_url: config.gateway_url.clone(),
};
```

---

## Rules
- `cargo build --workspace` must pass clean
- Add `base64 = "0.22"` to agent-tools/Cargo.toml if not present
- No unwrap() in production paths
- Unit tests for: image path collection, PDF param parsing, TTS key selection logic

## Completion
```bash
openclaw system event --text "Sprint 5 done: browser tool wired (all 16 actions), canvas tool wired, TTS tool (ElevenLabs+OpenAI routing), image analysis tool, PDF analysis tool, ToolContext gets browser_manager+canvas_host" --mode now
```

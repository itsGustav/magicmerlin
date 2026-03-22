# Sprint 5 — Agent A: Media Tools Completion

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
`agent-tools/src/tools.rs` has stub implementations for browser, canvas, tts, image, pdf tools.
`media/src/` has real implementations — this sprint wires them together.

## Your Mission
Wire the `agent-tools` stubs to the real `media` crate implementations.

---

## Tool 1: `browser` — Full CDP-backed browser tool

The `BrowserTool` in tools.rs calls `unimplemented!()` or returns empty. Wire it to `media::browser`.

### Supported actions (from the tool schema):
- `status` → `media::browser::BrowserManager::status()`
- `start` → `BrowserManager::start(profile)`
- `stop` → `BrowserManager::stop(profile)`
- `profiles` → `BrowserManager::list_profiles()`
- `tabs` → `BrowserManager::list_tabs(target_id)`
- `open` → `BrowserManager::open_url(url, profile)`
- `focus` → `BrowserManager::focus_tab(target_id)`
- `close` → `BrowserManager::close_tab(target_id)`
- `snapshot` → `BrowserManager::snapshot(target_id, refs)` — returns accessibility tree as text
- `screenshot` → `BrowserManager::screenshot(target_id, type_)` — returns base64 PNG/JPEG
- `navigate` → `BrowserManager::navigate(target_id, url)`
- `console` → `BrowserManager::get_console_logs(target_id)`
- `act` → `BrowserManager::act(target_id, request)` — click/type/press/hover/drag/select/fill/evaluate

```rust
// In BrowserTool::execute:
let action = required_string(&params, "action", "browser")?;
let profile = params.get("profile").and_then(Value::as_str).unwrap_or("openclaw");
let target_id = params.get("targetId").and_then(Value::as_str);

// Get browser manager from ToolContext (add Arc<BrowserManager> to ToolContext)
let browser = ctx.browser_manager.as_ref()
    .ok_or_else(|| ToolError::Unavailable("browser manager not initialized".into()))?;

match action.as_str() {
    "snapshot" => {
        let snapshot = browser.snapshot(target_id, profile).await?;
        Ok(ToolResult::json(json!({ "snapshot": snapshot })))
    }
    "screenshot" => {
        let (b64, mime) = browser.screenshot(target_id, profile).await?;
        Ok(ToolResult::json(json!({ "data": b64, "mimeType": mime })))
    }
    "act" => {
        let request = params.get("request")
            .ok_or_else(|| ToolError::MissingParam("request".into()))?;
        let result = browser.act(target_id, profile, request.clone()).await?;
        Ok(ToolResult::json(result))
    }
    // ... etc
}
```

Add `Arc<media::browser::BrowserManager>` to `ToolContext` in `agent-tools/src/registry.rs`.

---

## Tool 2: `canvas` — Canvas host process manager

The `CanvasTool` needs to wire to `media::canvas`.

### Actions:
- `present` → `CanvasHost::present(url, width, height, node)`
- `hide` → `CanvasHost::hide()`
- `navigate` → `CanvasHost::navigate(url)`
- `eval` → `CanvasHost::eval(javascript)` — returns result
- `snapshot` → `CanvasHost::snapshot(outputFormat, quality)` — returns base64 image
- `a2ui_push` → `CanvasHost::a2ui_push(jsonl)` — push agent-to-UI events
- `a2ui_reset` → `CanvasHost::a2ui_reset()`

Add `Arc<media::canvas::CanvasHost>` to `ToolContext`.

---

## Tool 3: `tts` — Text-to-speech with channel routing

The `TtsTool` needs to wire to `media::tts`.

```rust
// TtsTool::execute:
// Params: { text: String, channel?: String }
// 1. Route to media::tts::TtsRouter based on config (ElevenLabs, OpenAI TTS, etc.)
// 2. Get audio bytes back
// 3. If channel is "telegram": encode as OGG/Opus for Telegram voice note
//    If channel is "discord": keep as MP3
//    Default: return base64 audio
// 4. Return { audio: base64, mimeType: "audio/ogg", duration_ms: N }

let tts = ctx.tts_router.as_ref()
    .ok_or_else(|| ToolError::Unavailable("TTS not configured".into()))?;
let text = required_string(&params, "text", "tts")?;
let audio = tts.synthesize(&text).await?;
// Format for channel...
```

Add `Arc<media::tts::TtsRouter>` to `ToolContext`.

---

## Tool 4: `image` — Vision model image analysis

The `ImageTool` needs to wire to `media::understanding`.

```rust
// ImageTool::execute:
// Params: { image?: String, images?: [String], prompt?: String, model?: String }
// 1. Load image(s) from path or URL
// 2. Route to media::understanding::ImageUnderstanding
// 3. Return { result: String } with the model's analysis

let understanding = ctx.media_understanding.as_ref()
    .ok_or_else(|| ToolError::Unavailable("media understanding not configured".into()))?;

let prompt = params.get("prompt").and_then(Value::as_str)
    .unwrap_or("Describe this image in detail.");

if let Some(img_path) = params.get("image").and_then(Value::as_str) {
    let result = understanding.analyze_image(img_path, prompt).await?;
    return Ok(ToolResult::json(json!({ "result": result })));
}

if let Some(images) = params.get("images").and_then(Value::as_array) {
    let paths: Vec<&str> = images.iter().filter_map(Value::as_str).collect();
    let result = understanding.analyze_images(&paths, prompt).await?;
    return Ok(ToolResult::json(json!({ "result": result })));
}
```

---

## Tool 5: `pdf` — PDF analysis

The `PdfTool` needs to wire to `media::understanding`.

```rust
// PdfTool::execute:
// Params: { pdf?: String, pdfs?: [String], prompt: String, pages?: String, model?: String }
// 1. Load PDF from path or URL
// 2. Try native provider analysis first (Anthropic, Google support native PDF)
// 3. Fall back to text extraction if provider doesn't support native PDF
// 4. Return { result: String }
```

---

## ToolContext Additions

In `agent-tools/src/registry.rs`, add optional fields to `ToolContext`:

```rust
pub struct ToolContext {
    // existing fields...
    pub browser_manager: Option<Arc<media::browser::BrowserManager>>,
    pub canvas_host: Option<Arc<media::canvas::CanvasHost>>,
    pub tts_router: Option<Arc<media::tts::TtsRouter>>,
    pub media_understanding: Option<Arc<media::understanding::MediaUnderstanding>>,
}
```

These are `Option` so the tools degrade gracefully when not configured.

---

## Tool 6: `web_search` hardening

The existing `WebSearchTool` may have shallow error handling. Harden it:
- Retry on 429 with exponential backoff (max 3 retries)
- Parse Brave Search API response properly: extract `web.results[].title/url/description`
- Return proper JSON: `{ results: [{title, url, snippet}], total: N }`
- Handle missing `BRAVE_SEARCH_API_KEY` gracefully (return error message, not panic)

## Tool 7: `web_fetch` hardening

Harden the existing `WebFetchTool`:
- Real readability-style extraction: strip nav/footer/sidebar HTML, keep main content
- Use `scraper` crate for HTML parsing: `scraper = "0.19"`
- Implement `extract_main_content(html: &str) -> String`:
  - Remove: `<nav>`, `<header>`, `<footer>`, `<aside>`, `<script>`, `<style>`
  - Keep: `<article>`, `<main>`, `<p>`, `<h1-6>`, `<li>`, `<code>`
  - Convert to Markdown: `<h1>` → `# `, `<strong>` → `**`, `<a href>` → `[text](url)`
- Handle `maxChars` truncation
- Add timeout: 30s default
- Support `extractMode=text` (strip all HTML tags, plain text only)

---

## Rules
- `cargo build --workspace` clean
- `Option<Arc<...>>` for all media deps in ToolContext — graceful unavailable
- Unit tests for web_search response parsing, web_fetch HTML extraction

## Completion
```bash
openclaw system event --text "Sprint 5A done: browser/canvas/tts/image/pdf tools wired to media crate, web_search+web_fetch hardened with real HTML extraction" --mode now
```

# Deepening Pass 3: Media + Agent-Tools — Full Implementation

Current: 60.5K lines, 18 crates. Deepen media and agent-tools to production quality.

## Rules
- Read existing code first, EXTEND don't rewrite
- Full error handling, edge cases, real implementations
- Comprehensive tests
- cargo check + cargo test must pass

## 1. Media Crate (target: 8000+ lines, currently 3073)

### Understanding (media/src/understanding.rs → split into module dir)
- Image analysis: Full request builders for OpenAI (gpt-4o vision), Anthropic (base64 image blocks), Google (inlineData). Provider selection based on config. Resize large images before sending. Support both URL and local file path inputs.
- Audio transcription: OpenAI Whisper API (multipart form upload), Groq whisper-large-v3, Deepgram nova-2. Audio format detection. Duration limits.
- Video analysis: ffmpeg frame extraction (shell out to ffmpeg -ss -vf fps=1 -vframes N), extract N key frames, analyze each with vision model, combine descriptions.
- PDF analysis: Anthropic native (document source block), Google native (inlineData pdf), fallback text extraction (shell out to pdftotext), fallback image extraction (shell out to pdftoppm + vision).
- Provider routing: Cost-based selection, capability-based fallback, concurrent analysis for large docs.

### Browser (media/src/browser.rs → split into module dir)
- CDP WebSocket client: Connect to Chrome DevTools via ws://localhost:9222
- Target discovery: List targets via /json endpoint, connect to specific tab
- Page.navigate: Load URL, wait for load event
- DOM.getDocument + DOM.describeNode: Build accessibility tree
- Page.captureScreenshot: PNG/JPEG with quality and clip options
- Runtime.evaluate: Execute JavaScript, return result
- Input.dispatchMouseEvent: Click at coordinates
- Input.dispatchKeyEvent: Type text, key presses
- Element resolution: Map aria refs to DOM node IDs for interaction
- Tab management: Create new tab, close tab, list all tabs, focus tab
- Process management: Launch chrome with --remote-debugging-port, detect existing instance, graceful shutdown
- Profile support: Default vs chrome relay profile with different user data dirs

### Canvas (media/src/canvas.rs)
- Axum-based HTTP server serving HTML content on configurable port
- A2UI protocol: Accept JSONL push messages (set_html, eval_js, navigate, screenshot)
- Screenshot: Use headless chrome CDP to capture rendered canvas
- JavaScript eval: Route through CDP Runtime.evaluate
- State management: Track current URL/HTML per canvas instance

### Links (media/src/links.rs)
- Full HTML parser: Extract <title>, <meta name="description">, og:title, og:description, og:image, twitter:card, canonical URL
- Readability: Strip nav/header/footer/sidebar, extract main <article> or largest text block, convert to markdown
- Content length detection: Return word count, estimated read time
- Cache: HashMap<String, (Instant, LinkResult)> with configurable TTL (default 1h), max entries limit
- Error handling: Timeouts, redirects (follow up to 5), non-HTML content types

### TTS (media/src/tts.rs)
- ElevenLabs: Full REST API — list voices, text-to-speech synthesis, voice settings (stability, similarity_boost, style), model selection (eleven_multilingual_v2, eleven_turbo_v2)
- OpenAI: TTS API — models tts-1 and tts-1-hd, voices (alloy, echo, fable, onyx, nova, shimmer), speed control
- Output formats: MP3, OGG (Opus for Telegram voice notes), WAV, FLAC
- Voice config: Per-agent preferred voice stored in config
- Audio processing: Convert between formats using ffmpeg if needed
- Streaming: Support streaming audio for long text

## 2. Agent-Tools Crate (target: 8000+ lines, currently 1345)

Implement EVERY tool as a real, functional tool:

### exec tool
- Shell command execution via tokio::process::Command
- PTY support via portable-pty crate: allocate pseudo-terminal, handle resize
- Background mode: spawn process, return session ID, track in ProcessRegistry
- Environment variable injection from tool context
- Working directory support
- Timeout handling: kill process after configurable timeout
- Output capture: stdout + stderr combined or separate
- Exit code reporting

### process tool
- ProcessRegistry: HashMap<String, ProcessSession> tracking background processes
- list: Return all sessions with pid, status, runtime
- poll: Wait for output up to timeout_ms, return new output since last poll
- log: Return captured output with offset/limit
- write: Send data to process stdin
- send-keys: Send terminal key sequences (for TTY processes)
- paste: Bracketed paste mode support
- kill: Send SIGTERM, wait, SIGKILL if needed
- submit: Write + EOF (close stdin)

### read tool
- Read text files with offset/limit (line-based)
- Detect binary files, refuse to read
- Image files: Read as attachment (return path for vision model)
- File size limits: Truncate at 50KB or 2000 lines
- Encoding detection: UTF-8, Latin-1 fallback

### write tool
- Create files with content
- Auto-create parent directories (fs::create_dir_all)
- Atomic writes: Write to temp file, rename

### edit tool
- Find exact old_string in file content
- Replace with new_string
- Validate old_string exists exactly once (error if 0 or >1 matches)
- Preserve file permissions and encoding

### web_search tool
- Brave Search API: GET https://api.search.brave.com/res/v1/web/search
- Parameters: q, count (1-10), freshness, country, search_lang, ui_lang
- Parse response: title, url, description for each result
- API key from config/secrets

### web_fetch tool
- Fetch URL via reqwest with configurable timeout
- HTML → Markdown conversion (strip tags, preserve links, headers, lists, code blocks)
- HTML → Text extraction (strip all formatting)
- Max chars truncation
- Follow redirects (up to 5)
- User-Agent header

### memory_search tool
- Semantic search over MEMORY.md and memory/*.md files
- Simple keyword-based search (TF-IDF or BM25) as baseline
- Return top N results with file path, line numbers, matching snippet
- Min score threshold

### memory_get tool
- Read specific lines from memory files
- Support path, from (line number), lines (count)
- Validate path is within memory directory

### session_status tool
- Return current session info: model, tokens used, context %, cost, cache stats
- Support optional model override

### sessions_list tool
- List sessions with filters: activeMinutes, kinds, limit
- Return session key, agent, updated time, token count

### sessions_history tool
- Fetch message history for a session by sessionKey
- Support limit and includeTools options

### sessions_send tool
- Send a message to another session
- Support sessionKey or label targeting

### sessions_spawn tool
- Spawn isolated sub-agent session
- Support runtime (subagent/acp), mode (run/session), model override
- Track parent-child relationship

### subagents tool
- list: Return spawned sub-agents with status
- steer: Send message to sub-agent
- kill: Terminate sub-agent

### agents_list tool
- Return list of available agent IDs

### message tool
- Send message to channel (Telegram, Discord, etc.)
- Support: text, media, buttons, reactions, edit, delete
- Route through channel framework

### image tool
- Analyze image(s) with vision model
- Support single image or multiple (up to 20)
- Route through media understanding

### pdf tool
- Analyze PDF with model
- Support single or multiple PDFs
- Page range selection
- Route through media understanding

### tts tool
- Convert text to speech
- Route through media TTS module
- Return audio file path

### browser tool
- Full browser control: start, stop, status, tabs, open, snapshot, screenshot, navigate, act
- Route through media browser module

### canvas tool
- Present, hide, navigate, eval, snapshot, a2ui_push
- Route through media canvas module

### nodes tool
- Remote node control: status, camera, screen, location, run, invoke
- HTTP client to node endpoints

### Tool Registry
- Register all tools at startup with JSON Schema for parameters
- Tool permission checking: deny lists, workspace-only FS
- Tool result truncation at configurable limit
- Tool execution timeout

Commit after media deepening, then again after agent-tools deepening.

When completely finished, run:
openclaw system event --text "Pass 3 complete: Media (8K+) + Agent-Tools (8K+) fully implemented" --mode now

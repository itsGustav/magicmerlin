# Phase 4: Media & Intelligence — Build Instructions

Phases 0-3 complete (15 crates, 16.6K lines). Now build media processing, browser automation, canvas, link understanding, and TTS.

## What To Build — new crate: `media`

### 4.1 Media Understanding (media/src/understanding/)
Multi-provider media analysis:
1. **Image analysis**: Route to vision-capable models (OpenAI GPT-4o, Anthropic Claude, Google Gemini, xAI Grok). Accept image path/URL, return text description.
2. **Audio transcription**: Support Whisper (OpenAI API), Deepgram, Groq whisper. Accept audio file path, return transcript.
3. **Video frame extraction**: Use ffmpeg (shell out) to extract key frames, then analyze with vision model.
4. **PDF analysis**: Native PDF support for Anthropic/Google (base64 encode), fallback to text extraction (pdftotext) + image extraction for others.
5. **Provider routing**: Pick cheapest/fastest provider based on config. Automatic fallback on failure.
6. **Shared types**: MediaType enum (Image, Audio, Video, Pdf), AnalysisResult struct.

### 4.2 Browser Automation (media/src/browser/)
Chrome DevTools Protocol integration:
1. **CDP client** using `chromiumoxide` crate or custom WebSocket CDP client
2. **Page operations**: navigate, snapshot (accessibility tree → text), screenshot (PNG/JPEG)
3. **Actions**: click, type, press, hover, drag, select, fill (by element ref)
4. **Tab management**: open, close, focus, list tabs
5. **Profile management**: default profile, chrome relay profile
6. **Process lifecycle**: start/stop browser, connect to existing
7. **Snapshot format**: Aria tree → compact text representation for LLM consumption

### 4.3 Canvas Host (media/src/canvas/)
HTML canvas rendering:
1. **Canvas server**: Serve HTML content on a local port
2. **A2UI protocol**: Agent pushes JSONL instructions to update canvas
3. **Snapshot**: Capture rendered canvas as screenshot
4. **JavaScript evaluation**: Run JS in canvas context, return result
5. **Navigation**: Load URLs in canvas

### 4.4 Link Understanding (media/src/links/)
URL metadata extraction:
1. **Fetch and parse**: Download URL, extract title, description, OG tags
2. **Readability**: Extract main content as markdown (like Mozilla Readability)
3. **Content summarization**: Route to LLM for summary of long content
4. **Caching**: Cache results by URL for 1 hour

### 4.5 TTS — Text-to-Speech (media/src/tts/)
1. **ElevenLabs provider**: REST API for speech synthesis, voice selection
2. **OpenAI provider**: TTS API (tts-1, tts-1-hd models)
3. **Voice configuration**: Preferred voice per agent
4. **Output formats**: MP3, OGG (for Telegram voice notes), WAV
5. **Channel routing**: Auto-format for target channel (Telegram voice note, Discord file, etc.)

## Integration
- Add `media` crate to workspace
- Wire media tools into agent-tools (image, pdf, tts, browser, canvas tools)
- Feature flags: `browser`, `canvas`, `tts`, `media-understanding`
- `cargo check` + `cargo test` must pass clean

## Dependencies
- chromiumoxide = "0.7" (browser/CDP) OR skip dep and shell out to playwright
- reqwest (already available)
- base64 (already available)
- serde + serde_json (already available)
- tokio::process for ffmpeg/pdftotext shell-outs

When completely finished, run:
openclaw system event --text "Phase 4 complete: Media & Intelligence built and tested" --mode now

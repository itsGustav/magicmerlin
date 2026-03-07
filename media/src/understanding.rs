use std::collections::HashMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;

use crate::{MediaError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Pdf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VisionProvider {
    OpenAi,
    Anthropic,
    Google,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MediaSource {
    File { path: PathBuf },
    Url { url: String },
    Base64 { mime_type: String, data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisRequest {
    pub media_type: MediaType,
    pub source: MediaSource,
    pub prompt: String,
    #[serde(default)]
    pub preferred_provider: Option<VisionProvider>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisResult {
    pub media_type: MediaType,
    pub provider: String,
    pub text: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct UnderstandingConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub openai_base_url: String,
    pub anthropic_base_url: String,
    pub google_base_url: String,
    pub openai_model: String,
    pub anthropic_model: String,
    pub google_model: String,
}

impl Default for UnderstandingConfig {
    fn default() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            google_api_key: std::env::var("GOOGLE_API_KEY").ok(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            anthropic_base_url: "https://api.anthropic.com/v1".to_string(),
            google_base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            openai_model: "gpt-4.1-mini".to_string(),
            anthropic_model: "claude-3-7-sonnet-latest".to_string(),
            google_model: "gemini-2.0-flash".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnderstandingClient {
    http: reqwest::Client,
    config: UnderstandingConfig,
}

impl UnderstandingClient {
    pub fn new(config: UnderstandingConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
        }
    }

    pub fn with_client(http: reqwest::Client, config: UnderstandingConfig) -> Self {
        Self { http, config }
    }

    pub async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let provider = self.select_provider(request.media_type, request.preferred_provider)?;
        match request.media_type {
            MediaType::Image => self.analyze_image(request, provider).await,
            MediaType::Audio => self.transcribe_audio(request, provider).await,
            MediaType::Video => self.analyze_video(request, provider).await,
            MediaType::Pdf => self.analyze_pdf(request, provider).await,
        }
    }

    pub fn select_provider(
        &self,
        media_type: MediaType,
        preferred: Option<VisionProvider>,
    ) -> Result<VisionProvider> {
        if let Some(provider) = preferred {
            if self.provider_available(provider) {
                return Ok(provider);
            }
            return Err(MediaError::InvalidInput(format!(
                "preferred provider {:?} requested but no credential is configured",
                provider
            )));
        }

        let order = self.provider_priority(media_type);
        order
            .into_iter()
            .find(|provider| self.provider_available(*provider))
            .ok_or_else(|| {
                MediaError::InvalidInput(
                    "no provider configured; set OPENAI_API_KEY, ANTHROPIC_API_KEY, or GOOGLE_API_KEY"
                        .to_string(),
                )
            })
    }

    fn provider_priority(&self, media_type: MediaType) -> Vec<VisionProvider> {
        match media_type {
            MediaType::Image => vec![
                VisionProvider::OpenAi,
                VisionProvider::Anthropic,
                VisionProvider::Google,
            ],
            MediaType::Audio => vec![VisionProvider::OpenAi],
            MediaType::Video => vec![
                VisionProvider::OpenAi,
                VisionProvider::Anthropic,
                VisionProvider::Google,
            ],
            MediaType::Pdf => vec![
                VisionProvider::Anthropic,
                VisionProvider::Google,
                VisionProvider::Local,
            ],
        }
    }

    fn provider_available(&self, provider: VisionProvider) -> bool {
        match provider {
            VisionProvider::OpenAi => self.config.openai_api_key.is_some(),
            VisionProvider::Anthropic => self.config.anthropic_api_key.is_some(),
            VisionProvider::Google => self.config.google_api_key.is_some(),
            VisionProvider::Local => true,
        }
    }

    async fn analyze_image(
        &self,
        request: AnalysisRequest,
        provider: VisionProvider,
    ) -> Result<AnalysisResult> {
        match provider {
            VisionProvider::OpenAi => self.openai_image(request).await,
            VisionProvider::Anthropic => self.anthropic_image(request).await,
            VisionProvider::Google => self.google_image(request).await,
            VisionProvider::Local => Err(MediaError::InvalidInput(
                "local image analysis is not implemented".to_string(),
            )),
        }
    }

    async fn transcribe_audio(
        &self,
        request: AnalysisRequest,
        provider: VisionProvider,
    ) -> Result<AnalysisResult> {
        if provider != VisionProvider::OpenAi {
            return Err(MediaError::InvalidInput(
                "audio transcription currently routes to OpenAI Whisper only".to_string(),
            ));
        }

        let path = request.source.file_path().ok_or_else(|| {
            MediaError::InvalidInput("audio transcription requires file source".to_string())
        })?;
        let bytes = tokio::fs::read(&path).await?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("audio.bin")
            .to_string();
        let boundary = format!(
            "----mm-boundary-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let mut body = Vec::new();
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\nwhisper-1\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"response_format\"\r\n\r\nverbose_json\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(&bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let response = self
            .http
            .post(format!(
                "{}/audio/transcriptions",
                self.config.openai_base_url
            ))
            .bearer_auth(self.require_api_key(VisionProvider::OpenAi)?)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await?;
        let body = self.expect_success_json(response, "openai whisper").await?;

        let text = body
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        Ok(AnalysisResult {
            media_type: MediaType::Audio,
            provider: "openai-whisper".to_string(),
            text,
            metadata: body,
        })
    }

    async fn analyze_video(
        &self,
        request: AnalysisRequest,
        provider: VisionProvider,
    ) -> Result<AnalysisResult> {
        let path = request.source.file_path().ok_or_else(|| {
            MediaError::InvalidInput("video analysis requires file source".to_string())
        })?;
        let frames = self.extract_video_frames(&path).await?;
        if frames.is_empty() {
            return Err(MediaError::Execution(
                "ffmpeg produced no frames".to_string(),
            ));
        }

        let mut combined = Vec::with_capacity(frames.len());
        for (index, frame_path) in frames.iter().enumerate() {
            let image_req = AnalysisRequest {
                media_type: MediaType::Image,
                source: MediaSource::File {
                    path: frame_path.clone(),
                },
                prompt: format!("{}\n\nFrame #{}", request.prompt, index + 1),
                preferred_provider: Some(provider),
                metadata: request.metadata.clone(),
            };
            let analysis = self.analyze_image(image_req, provider).await?;
            combined.push(analysis.text);
        }

        Ok(AnalysisResult {
            media_type: MediaType::Video,
            provider: format!("{:?}", provider).to_lowercase(),
            text: combined.join("\n\n"),
            metadata: json!({
                "frames": frames,
                "frame_count": combined.len(),
            }),
        })
    }

    async fn analyze_pdf(
        &self,
        request: AnalysisRequest,
        provider: VisionProvider,
    ) -> Result<AnalysisResult> {
        match provider {
            VisionProvider::Anthropic => self.anthropic_pdf(request).await,
            VisionProvider::Google => self.google_pdf(request).await,
            VisionProvider::OpenAi => {
                self.anthropic_pdf(AnalysisRequest {
                    preferred_provider: Some(VisionProvider::Anthropic),
                    ..request
                })
                .await
            }
            VisionProvider::Local => self.pdf_to_text(request).await,
        }
    }

    async fn pdf_to_text(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let path = request.source.file_path().ok_or_else(|| {
            MediaError::InvalidInput("pdf fallback requires file source".to_string())
        })?;
        let output = Command::new("pdftotext")
            .arg("-layout")
            .arg(path.as_os_str())
            .arg("-")
            .output()
            .await
            .map_err(|err| MediaError::Execution(format!("pdftotext invocation failed: {err}")))?;
        if !output.status.success() {
            return Err(MediaError::Execution(format!(
                "pdftotext failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(AnalysisResult {
            media_type: MediaType::Pdf,
            provider: "local-pdftotext".to_string(),
            text,
            metadata: json!({
                "fallback": "pdftotext"
            }),
        })
    }

    async fn extract_video_frames(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let dir = tempfile::Builder::new()
            .prefix("media-frames-")
            .tempdir()
            .map_err(MediaError::Io)?;
        let frame_pattern = dir.path().join("frame-%03d.jpg");

        let output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-i")
            .arg(path)
            .arg("-vf")
            .arg("fps=1")
            .arg("-frames:v")
            .arg("6")
            .arg(frame_pattern.as_os_str())
            .output()
            .await
            .map_err(|err| MediaError::Execution(format!("ffmpeg invocation failed: {err}")))?;

        if !output.status.success() {
            return Err(MediaError::Execution(format!(
                "ffmpeg failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let mut paths = Vec::new();
        let mut entries = tokio::fs::read_dir(dir.path()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            if file_type.is_file() {
                paths.push(entry.path());
            }
        }
        paths.sort();

        // Materialize files to a stable temp area since dir will drop at function end.
        let persistent_dir = std::env::temp_dir().join(format!(
            "media-frames-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        tokio::fs::create_dir_all(&persistent_dir).await?;

        let mut persistent_paths = Vec::new();
        for path in paths {
            let target = persistent_dir.join(
                path.file_name()
                    .ok_or_else(|| MediaError::Execution("frame file name missing".to_string()))?,
            );
            tokio::fs::copy(&path, &target).await?;
            persistent_paths.push(target);
        }

        Ok(persistent_paths)
    }

    async fn openai_image(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let image_url = self.source_to_data_url(&request.source).await?;
        let payload = json!({
            "model": self.config.openai_model,
            "input": [{
                "role": "user",
                "content": [
                    {
                        "type": "input_text",
                        "text": request.prompt,
                    },
                    {
                        "type": "input_image",
                        "image_url": image_url,
                    }
                ]
            }]
        });

        let response = self
            .http
            .post(format!("{}/responses", self.config.openai_base_url))
            .bearer_auth(self.require_api_key(VisionProvider::OpenAi)?)
            .json(&payload)
            .send()
            .await?;
        let body = self.expect_success_json(response, "openai image").await?;

        let text = extract_openai_response_text(&body);
        Ok(AnalysisResult {
            media_type: MediaType::Image,
            provider: "openai".to_string(),
            text,
            metadata: body,
        })
    }

    async fn anthropic_image(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let (media_type, data) = self.source_to_base64(&request.source).await?;
        let payload = json!({
            "model": self.config.anthropic_model,
            "max_tokens": 1000,
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": data
                        }
                    },
                    {
                        "type": "text",
                        "text": request.prompt
                    }
                ]
            }]
        });

        let response = self
            .http
            .post(format!("{}/messages", self.config.anthropic_base_url))
            .header(
                "x-api-key",
                self.require_api_key(VisionProvider::Anthropic)?,
            )
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;
        let body = self
            .expect_success_json(response, "anthropic image")
            .await?;

        let text = body
            .get("content")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find_map(|item| {
                    if item.get("type")?.as_str()? == "text" {
                        item.get("text")?.as_str().map(ToOwned::to_owned)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();
        Ok(AnalysisResult {
            media_type: MediaType::Image,
            provider: "anthropic".to_string(),
            text,
            metadata: body,
        })
    }

    async fn google_image(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let (mime_type, data) = self.source_to_base64(&request.source).await?;
        let payload = json!({
            "contents": [{
                "parts": [
                    {
                        "text": request.prompt
                    },
                    {
                        "inlineData": {
                            "mimeType": mime_type,
                            "data": data
                        }
                    }
                ]
            }]
        });

        let key = self.require_api_key(VisionProvider::Google)?;
        let response = self
            .http
            .post(format!(
                "{}/models/{}:generateContent?key={}",
                self.config.google_base_url, self.config.google_model, key
            ))
            .json(&payload)
            .send()
            .await?;
        let body = self.expect_success_json(response, "google image").await?;

        let text = body
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        Ok(AnalysisResult {
            media_type: MediaType::Image,
            provider: "google".to_string(),
            text,
            metadata: body,
        })
    }

    async fn anthropic_pdf(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let (_, data) = self.source_to_base64(&request.source).await?;
        let payload = json!({
            "model": self.config.anthropic_model,
            "max_tokens": 1000,
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "document",
                        "source": {
                            "type": "base64",
                            "media_type": "application/pdf",
                            "data": data
                        }
                    },
                    {
                        "type": "text",
                        "text": request.prompt
                    }
                ]
            }]
        });

        let response = self
            .http
            .post(format!("{}/messages", self.config.anthropic_base_url))
            .header(
                "x-api-key",
                self.require_api_key(VisionProvider::Anthropic)?,
            )
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;
        let body = self.expect_success_json(response, "anthropic pdf").await?;

        let text = body
            .get("content")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find_map(|item| {
                    if item.get("type")?.as_str()? == "text" {
                        item.get("text")?.as_str().map(ToOwned::to_owned)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();

        Ok(AnalysisResult {
            media_type: MediaType::Pdf,
            provider: "anthropic".to_string(),
            text,
            metadata: body,
        })
    }

    async fn google_pdf(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let (_, data) = self.source_to_base64(&request.source).await?;
        let payload = json!({
            "contents": [{
                "parts": [
                    {
                        "text": request.prompt
                    },
                    {
                        "inlineData": {
                            "mimeType": "application/pdf",
                            "data": data
                        }
                    }
                ]
            }]
        });

        let response = self
            .http
            .post(format!(
                "{}/models/{}:generateContent?key={}",
                self.config.google_base_url,
                self.config.google_model,
                self.require_api_key(VisionProvider::Google)?
            ))
            .json(&payload)
            .send()
            .await?;
        let body = self.expect_success_json(response, "google pdf").await?;

        let text = body
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        Ok(AnalysisResult {
            media_type: MediaType::Pdf,
            provider: "google".to_string(),
            text,
            metadata: body,
        })
    }

    async fn source_to_data_url(&self, source: &MediaSource) -> Result<String> {
        match source {
            MediaSource::Base64 { mime_type, data } => {
                Ok(format!("data:{mime_type};base64,{data}"))
            }
            _ => {
                let (mime, data) = self.source_to_base64(source).await?;
                Ok(format!("data:{mime};base64,{data}"))
            }
        }
    }

    async fn source_to_base64(&self, source: &MediaSource) -> Result<(String, String)> {
        match source {
            MediaSource::Base64 { mime_type, data } => Ok((mime_type.clone(), data.clone())),
            MediaSource::Url { url } => {
                let response = self.http.get(url).send().await?;
                let mime = response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let bytes = response.bytes().await?;
                Ok((
                    mime,
                    base64::engine::general_purpose::STANDARD.encode(bytes),
                ))
            }
            MediaSource::File { path } => {
                let bytes = tokio::fs::read(path).await?;
                let mime = guess_mime_from_path(path);
                Ok((
                    mime,
                    base64::engine::general_purpose::STANDARD.encode(bytes),
                ))
            }
        }
    }

    async fn expect_success_json(
        &self,
        response: reqwest::Response,
        source: &str,
    ) -> Result<Value> {
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(MediaError::Execution(format!(
                "{source} request failed with {status}: {text}"
            )));
        }
        serde_json::from_str(&text).map_err(|err| {
            MediaError::Execution(format!("{source} response parse failed: {err}: {text}"))
        })
    }

    fn require_api_key(&self, provider: VisionProvider) -> Result<&str> {
        let key = match provider {
            VisionProvider::OpenAi => self.config.openai_api_key.as_deref(),
            VisionProvider::Anthropic => self.config.anthropic_api_key.as_deref(),
            VisionProvider::Google => self.config.google_api_key.as_deref(),
            VisionProvider::Local => None,
        };
        key.ok_or_else(|| {
            MediaError::InvalidInput(format!("missing API key for provider {:?}", provider))
        })
    }

    pub fn provider_capabilities(&self) -> HashMap<VisionProvider, Vec<MediaType>> {
        let mut capabilities = HashMap::new();

        if self.config.openai_api_key.is_some() {
            capabilities.insert(
                VisionProvider::OpenAi,
                vec![MediaType::Image, MediaType::Audio, MediaType::Video],
            );
        }
        if self.config.anthropic_api_key.is_some() {
            capabilities.insert(
                VisionProvider::Anthropic,
                vec![MediaType::Image, MediaType::Pdf, MediaType::Video],
            );
        }
        if self.config.google_api_key.is_some() {
            capabilities.insert(
                VisionProvider::Google,
                vec![MediaType::Image, MediaType::Pdf, MediaType::Video],
            );
        }
        capabilities.insert(VisionProvider::Local, vec![MediaType::Pdf]);

        capabilities
    }
}

impl MediaSource {
    pub fn file_path(&self) -> Option<PathBuf> {
        match self {
            MediaSource::File { path } => Some(path.clone()),
            _ => None,
        }
    }
}

fn guess_mime_from_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn extract_openai_response_text(body: &Value) -> String {
    if let Some(text) = body
        .pointer("/output/0/content/0/text")
        .and_then(Value::as_str)
    {
        return text.to_string();
    }

    if let Some(text) = body.get("output_text").and_then(Value::as_str) {
        return text.to_string();
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> UnderstandingConfig {
        UnderstandingConfig {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            openai_base_url: "http://localhost/openai".to_string(),
            anthropic_base_url: "http://localhost/anthropic".to_string(),
            google_base_url: "http://localhost/google".to_string(),
            openai_model: "omni-mini".to_string(),
            anthropic_model: "claude-test".to_string(),
            google_model: "gemini-test".to_string(),
        }
    }

    #[test]
    fn provider_selection_prefers_supported_keys() {
        let mut config = test_config();
        config.google_api_key = Some("g-key".to_string());
        config.openai_api_key = Some("o-key".to_string());

        let client = UnderstandingClient::new(config);
        let provider = client
            .select_provider(MediaType::Image, None)
            .expect("provider should resolve");
        assert_eq!(provider, VisionProvider::OpenAi);
    }

    #[test]
    fn provider_selection_rejects_missing_preferred_key() {
        let client = UnderstandingClient::new(test_config());
        let err = client
            .select_provider(MediaType::Image, Some(VisionProvider::Anthropic))
            .expect_err("missing key should fail");
        assert!(format!("{err}").contains("preferred provider"));
    }

    #[test]
    fn mime_guess_uses_extensions() {
        assert_eq!(guess_mime_from_path(Path::new("x.png")), "image/png");
        assert_eq!(guess_mime_from_path(Path::new("x.PDF")), "application/pdf");
        assert_eq!(
            guess_mime_from_path(Path::new("x.bin")),
            "application/octet-stream"
        );
    }

    #[test]
    fn extract_openai_text_supports_response_shapes() {
        let shape1 = json!({
            "output": [{"content": [{"text": "alpha"}]}]
        });
        assert_eq!(extract_openai_response_text(&shape1), "alpha");

        let shape2 = json!({"output_text": "beta"});
        assert_eq!(extract_openai_response_text(&shape2), "beta");
    }

    #[test]
    fn media_source_file_path_roundtrip() {
        let source = MediaSource::File {
            path: PathBuf::from("/tmp/sample.png"),
        };
        assert_eq!(source.file_path(), Some(PathBuf::from("/tmp/sample.png")));
    }

    #[tokio::test]
    async fn base64_source_to_data_url_uses_inline_data() {
        let client = UnderstandingClient::new(test_config());
        let url = client
            .source_to_data_url(&MediaSource::Base64 {
                mime_type: "image/png".to_string(),
                data: "abc123".to_string(),
            })
            .await
            .expect("data URL should be built");
        assert_eq!(url, "data:image/png;base64,abc123");
    }

    #[test]
    fn provider_capabilities_include_local_pdf() {
        let client = UnderstandingClient::new(test_config());
        let capabilities = client.provider_capabilities();
        let local = capabilities
            .get(&VisionProvider::Local)
            .expect("local provider should always exist");
        assert!(local.contains(&MediaType::Pdf));
    }

    #[test]
    fn request_serialization_is_stable() {
        let request = AnalysisRequest {
            media_type: MediaType::Image,
            source: MediaSource::Url {
                url: "https://example.com/a.png".to_string(),
            },
            prompt: "Summarize the image".to_string(),
            preferred_provider: Some(VisionProvider::Google),
            metadata: json!({"trace_id": "abc"}),
        };

        let encoded = serde_json::to_value(request).expect("json encode");
        assert_eq!(encoded["media_type"], "image");
        assert_eq!(encoded["source"]["kind"], "url");
        assert_eq!(encoded["preferred_provider"], "google");
    }
}

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{AnalysisRequest, AnalysisResult, MediaType, UnderstandingClient};
use crate::{MediaError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioProvider {
    OpenAiWhisper,
    GroqWhisperLargeV3,
    DeepgramNova2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Flac,
    Mp4,
    M4a,
    Webm,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioMetadata {
    pub format: AudioFormat,
    pub mime_type: String,
    pub bytes: u64,
    pub estimated_seconds: f64,
    pub provider: AudioProvider,
}

impl UnderstandingClient {
    pub async fn detect_audio_metadata(
        &self,
        path: &Path,
        provider: AudioProvider,
    ) -> Result<AudioMetadata> {
        let meta = tokio::fs::metadata(path).await?;
        let bytes = meta.len();
        let format = detect_audio_format(path);
        let mime_type = audio_mime(format).to_string();

        // Conservative estimate for provider guardrails when duration is unknown.
        let estimated_seconds = estimate_duration_seconds(bytes, format);

        Ok(AudioMetadata {
            format,
            mime_type,
            bytes,
            estimated_seconds,
            provider,
        })
    }

    pub async fn transcribe_audio_with_fallback(
        &self,
        request: AnalysisRequest,
    ) -> Result<AnalysisResult> {
        let path = request.source.file_path().ok_or_else(|| {
            MediaError::InvalidInput("audio transcription requires file source".to_string())
        })?;

        let providers = [
            AudioProvider::OpenAiWhisper,
            AudioProvider::GroqWhisperLargeV3,
            AudioProvider::DeepgramNova2,
        ];

        let mut last_err: Option<MediaError> = None;
        for provider in providers {
            let meta = self.detect_audio_metadata(&path, provider).await?;
            if meta.estimated_seconds > max_duration_for(provider) {
                last_err = Some(MediaError::InvalidInput(format!(
                    "audio exceeds provider duration limit for {:?}: {:.1}s > {:.1}s",
                    provider,
                    meta.estimated_seconds,
                    max_duration_for(provider)
                )));
                continue;
            }

            let result = match provider {
                AudioProvider::OpenAiWhisper => {
                    self.transcribe_openai(path.as_path(), &request.prompt)
                        .await
                }
                AudioProvider::GroqWhisperLargeV3 => {
                    self.transcribe_groq(path.as_path(), &request.prompt).await
                }
                AudioProvider::DeepgramNova2 => {
                    self.transcribe_deepgram(path.as_path(), &request.prompt)
                        .await
                }
            };

            match result {
                Ok(mut analysis) => {
                    analysis.metadata = json!({
                        "audio": meta,
                        "provider": format!("{:?}", provider),
                    });
                    return Ok(analysis);
                }
                Err(err) => last_err = Some(err),
            }
        }

        Err(last_err.unwrap_or_else(|| {
            MediaError::Execution("audio transcription failed on all providers".to_string())
        }))
    }

    async fn transcribe_openai(&self, path: &Path, prompt: &str) -> Result<AnalysisResult> {
        let bytes = tokio::fs::read(path).await?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("audio.bin")
            .to_string();
        let boundary = format!("----mm-whisper-{}", now_seed());

        let mut body = Vec::new();
        push_multipart_field(&mut body, &boundary, "model", "whisper-1");
        push_multipart_field(&mut body, &boundary, "response_format", "verbose_json");
        if !prompt.trim().is_empty() {
            push_multipart_field(&mut body, &boundary, "prompt", prompt);
        }
        push_multipart_file(
            &mut body,
            &boundary,
            "file",
            &file_name,
            "application/octet-stream",
            &bytes,
        );
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let response =
            self.http
                .post(format!(
                    "{}/audio/transcriptions",
                    self.config.openai_base_url
                ))
                .bearer_auth(self.config.openai_api_key.as_deref().ok_or_else(|| {
                    MediaError::InvalidInput("missing OPENAI_API_KEY".to_string())
                })?)
                .header(
                    reqwest::header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(body)
                .send()
                .await?;

        let payload = read_json_or_error(response, "openai whisper").await?;
        Ok(AnalysisResult {
            media_type: MediaType::Audio,
            provider: "openai-whisper-1".to_string(),
            text: payload
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            metadata: payload,
        })
    }

    async fn transcribe_groq(&self, path: &Path, prompt: &str) -> Result<AnalysisResult> {
        let api_key = self.config.openai_api_key.as_deref().ok_or_else(|| {
            MediaError::InvalidInput("missing GROQ key (reusing OPENAI_API_KEY slot)".to_string())
        })?;
        let bytes = tokio::fs::read(path).await?;
        let boundary = format!("----mm-groq-{}", now_seed());
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("audio.bin");

        let mut body = Vec::new();
        push_multipart_field(&mut body, &boundary, "model", "whisper-large-v3");
        if !prompt.trim().is_empty() {
            push_multipart_field(&mut body, &boundary, "prompt", prompt);
        }
        push_multipart_file(
            &mut body,
            &boundary,
            "file",
            file_name,
            "application/octet-stream",
            &bytes,
        );
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let response = self
            .http
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .bearer_auth(api_key)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await?;

        let payload = read_json_or_error(response, "groq whisper-large-v3").await?;
        Ok(AnalysisResult {
            media_type: MediaType::Audio,
            provider: "groq-whisper-large-v3".to_string(),
            text: payload
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            metadata: payload,
        })
    }

    async fn transcribe_deepgram(&self, path: &Path, prompt: &str) -> Result<AnalysisResult> {
        let api_key = self.config.google_api_key.as_deref().ok_or_else(|| {
            MediaError::InvalidInput(
                "missing DEEPGRAM key (reusing GOOGLE_API_KEY slot)".to_string(),
            )
        })?;
        let bytes = tokio::fs::read(path).await?;
        let format = detect_audio_format(path);
        let mime = audio_mime(format);

        let response = self
            .http
            .post("https://api.deepgram.com/v1/listen?model=nova-2")
            .header("Authorization", format!("Token {api_key}"))
            .header("Content-Type", mime)
            .header("X-Transcript-Prompt", prompt)
            .body(bytes)
            .send()
            .await?;

        let payload = read_json_or_error(response, "deepgram nova-2").await?;
        let text = payload
            .pointer("/results/channels/0/alternatives/0/transcript")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        Ok(AnalysisResult {
            media_type: MediaType::Audio,
            provider: "deepgram-nova-2".to_string(),
            text,
            metadata: payload,
        })
    }
}

fn push_multipart_field(body: &mut Vec<u8>, boundary: &str, key: &str, value: &str) {
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{key}\"\r\n\r\n{value}\r\n"
        )
        .as_bytes(),
    );
}

fn push_multipart_file(
    body: &mut Vec<u8>,
    boundary: &str,
    field_name: &str,
    file_name: &str,
    content_type: &str,
    bytes: &[u8],
) {
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{field_name}\"; filename=\"{file_name}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n");
}

fn max_duration_for(provider: AudioProvider) -> f64 {
    match provider {
        AudioProvider::OpenAiWhisper => 60.0 * 60.0,
        AudioProvider::GroqWhisperLargeV3 => 30.0 * 60.0,
        AudioProvider::DeepgramNova2 => 120.0 * 60.0,
    }
}

fn detect_audio_format(path: &Path) -> AudioFormat {
    match path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "mp3" => AudioFormat::Mp3,
        "wav" => AudioFormat::Wav,
        "ogg" | "opus" => AudioFormat::Ogg,
        "flac" => AudioFormat::Flac,
        "mp4" => AudioFormat::Mp4,
        "m4a" => AudioFormat::M4a,
        "webm" => AudioFormat::Webm,
        _ => AudioFormat::Unknown,
    }
}

fn audio_mime(format: AudioFormat) -> &'static str {
    match format {
        AudioFormat::Mp3 => "audio/mpeg",
        AudioFormat::Wav => "audio/wav",
        AudioFormat::Ogg => "audio/ogg",
        AudioFormat::Flac => "audio/flac",
        AudioFormat::Mp4 => "audio/mp4",
        AudioFormat::M4a => "audio/x-m4a",
        AudioFormat::Webm => "audio/webm",
        AudioFormat::Unknown => "application/octet-stream",
    }
}

fn estimate_duration_seconds(bytes: u64, format: AudioFormat) -> f64 {
    let bits = bytes as f64 * 8.0;
    let bitrate = match format {
        AudioFormat::Wav => 1_411_000.0,
        AudioFormat::Flac => 900_000.0,
        AudioFormat::Ogg => 96_000.0,
        AudioFormat::Mp3 => 128_000.0,
        AudioFormat::Mp4 | AudioFormat::M4a => 128_000.0,
        AudioFormat::Webm => 96_000.0,
        AudioFormat::Unknown => 96_000.0,
    };
    bits / bitrate
}

async fn read_json_or_error(response: reqwest::Response, source: &str) -> Result<Value> {
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(MediaError::Execution(format!(
            "{source} request failed with {status}: {text}"
        )));
    }
    serde_json::from_str(&text)
        .map_err(|e| MediaError::Execution(format!("{source} parse failed: {e}; body={text}")))
}

fn now_seed() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_audio_format_by_extension() {
        assert_eq!(detect_audio_format(Path::new("a.mp3")), AudioFormat::Mp3);
        assert_eq!(detect_audio_format(Path::new("a.WAV")), AudioFormat::Wav);
        assert_eq!(
            detect_audio_format(Path::new("a.unknown")),
            AudioFormat::Unknown
        );
    }

    #[test]
    fn duration_estimate_works() {
        let seconds = estimate_duration_seconds(128_000, AudioFormat::Mp3);
        assert!(seconds > 7.0 && seconds < 9.0);
    }
}

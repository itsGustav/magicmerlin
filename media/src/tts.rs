use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{MediaError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Mp3,
    Ogg,
    Wav,
}

impl OutputFormat {
    pub fn elevenlabs_output_format(self) -> &'static str {
        match self {
            OutputFormat::Mp3 => "mp3_44100_128",
            OutputFormat::Ogg => "ogg_44100_128",
            OutputFormat::Wav => "pcm_44100",
        }
    }

    pub fn openai_audio_format(self) -> &'static str {
        match self {
            OutputFormat::Mp3 => "mp3",
            OutputFormat::Ogg => "ogg",
            OutputFormat::Wav => "wav",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            OutputFormat::Mp3 => "audio/mpeg",
            OutputFormat::Ogg => "audio/ogg",
            OutputFormat::Wav => "audio/wav",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceConfig {
    pub provider: TtsProvider,
    pub voice_id: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default = "default_stability")]
    pub stability: f32,
    #[serde(default = "default_similarity_boost")]
    pub similarity_boost: f32,
    #[serde(default)]
    pub style: f32,
    #[serde(default)]
    pub use_speaker_boost: bool,
}

fn default_speed() -> f32 {
    1.0
}

fn default_stability() -> f32 {
    0.5
}

fn default_similarity_boost() -> f32 {
    0.75
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TtsProvider {
    ElevenLabs,
    OpenAi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElevenLabsVoice {
    pub voice_id: String,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TtsConfig {
    pub openai_api_key: Option<String>,
    pub elevenlabs_api_key: Option<String>,
    pub openai_base_url: String,
    pub elevenlabs_base_url: String,
    pub openai_model: String,
    pub default_output_format: OutputFormat,
    pub agent_voice_configs: HashMap<String, VoiceConfig>,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            elevenlabs_api_key: std::env::var("ELEVENLABS_API_KEY").ok(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            elevenlabs_base_url: "https://api.elevenlabs.io/v1".to_string(),
            openai_model: "gpt-4o-mini-tts".to_string(),
            default_output_format: OutputFormat::Mp3,
            agent_voice_configs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtsClient {
    http: reqwest::Client,
    config: TtsConfig,
}

impl TtsClient {
    pub fn new(config: TtsConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
        }
    }

    pub fn with_client(http: reqwest::Client, config: TtsConfig) -> Self {
        Self { http, config }
    }

    pub fn voice_for_agent(&self, agent_id: &str) -> Option<&VoiceConfig> {
        self.config.agent_voice_configs.get(agent_id)
    }

    pub fn set_voice_for_agent(&mut self, agent_id: impl Into<String>, voice: VoiceConfig) {
        self.config
            .agent_voice_configs
            .insert(agent_id.into(), voice);
    }

    pub async fn list_elevenlabs_voices(&self) -> Result<Vec<ElevenLabsVoice>> {
        let api_key =
            self.config.elevenlabs_api_key.as_deref().ok_or_else(|| {
                MediaError::InvalidInput("missing ELEVENLABS_API_KEY".to_string())
            })?;

        let response = self
            .http
            .get(format!("{}/voices", self.config.elevenlabs_base_url))
            .header("xi-api-key", api_key)
            .send()
            .await?;

        let body = read_json_or_error(response, "list elevenlabs voices").await?;
        let voices = body
            .get("voices")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        Some(ElevenLabsVoice {
                            voice_id: item.get("voice_id")?.as_str()?.to_string(),
                            name: item
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown")
                                .to_string(),
                            category: item
                                .get("category")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(voices)
    }

    pub async fn synthesize_for_agent(
        &self,
        agent_id: &str,
        text: &str,
        format: Option<OutputFormat>,
    ) -> Result<Vec<u8>> {
        let voice = self.voice_for_agent(agent_id).ok_or_else(|| {
            MediaError::InvalidInput(format!("no voice configured for agent '{agent_id}'"))
        })?;

        match voice.provider {
            TtsProvider::ElevenLabs => {
                self.synthesize_elevenlabs(
                    voice,
                    text,
                    format.unwrap_or(self.config.default_output_format),
                )
                .await
            }
            TtsProvider::OpenAi => {
                self.synthesize_openai(
                    voice,
                    text,
                    format.unwrap_or(self.config.default_output_format),
                )
                .await
            }
        }
    }

    pub async fn synthesize_elevenlabs(
        &self,
        voice: &VoiceConfig,
        text: &str,
        format: OutputFormat,
    ) -> Result<Vec<u8>> {
        let api_key =
            self.config.elevenlabs_api_key.as_deref().ok_or_else(|| {
                MediaError::InvalidInput("missing ELEVENLABS_API_KEY".to_string())
            })?;

        let payload = json!({
            "text": text,
            "model_id": "eleven_multilingual_v2",
            "voice_settings": {
                "stability": clamp(voice.stability, 0.0, 1.0),
                "similarity_boost": clamp(voice.similarity_boost, 0.0, 1.0),
                "style": clamp(voice.style, 0.0, 1.0),
                "use_speaker_boost": voice.use_speaker_boost
            }
        });

        let response = self
            .http
            .post(format!(
                "{}/text-to-speech/{}/stream?output_format={}",
                self.config.elevenlabs_base_url,
                voice.voice_id,
                format.elevenlabs_output_format(),
            ))
            .header("xi-api-key", api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(MediaError::Execution(format!(
                "elevenlabs synth failed with {status}: {text}"
            )));
        }

        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(MediaError::Http)
    }

    pub async fn synthesize_openai(
        &self,
        voice: &VoiceConfig,
        text: &str,
        format: OutputFormat,
    ) -> Result<Vec<u8>> {
        let api_key = self
            .config
            .openai_api_key
            .as_deref()
            .ok_or_else(|| MediaError::InvalidInput("missing OPENAI_API_KEY".to_string()))?;

        let payload = json!({
            "model": self.config.openai_model,
            "voice": voice.voice_id,
            "input": text,
            "format": format.openai_audio_format(),
            "speed": clamp(voice.speed, 0.25, 4.0),
        });

        let response = self
            .http
            .post(format!("{}/audio/speech", self.config.openai_base_url))
            .bearer_auth(api_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(MediaError::Execution(format!(
                "openai tts failed with {status}: {text}"
            )));
        }

        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(MediaError::Http)
    }

    pub fn list_agent_voices(&self) -> Vec<(String, VoiceConfig)> {
        self.config
            .agent_voice_configs
            .iter()
            .map(|(agent, voice)| (agent.clone(), voice.clone()))
            .collect()
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

async fn read_json_or_error(response: reqwest::Response, op: &str) -> Result<Value> {
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(MediaError::Execution(format!(
            "{op} failed with {status}: {text}"
        )));
    }
    serde_json::from_str(&text)
        .map_err(|err| MediaError::Execution(format!("{op} parse failed: {err}: {text}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_maps_to_provider_specific_values() {
        assert_eq!(OutputFormat::Mp3.openai_audio_format(), "mp3");
        assert_eq!(OutputFormat::Wav.elevenlabs_output_format(), "pcm_44100");
        assert_eq!(OutputFormat::Ogg.content_type(), "audio/ogg");
    }

    #[test]
    fn clamp_behaves_within_range() {
        assert_eq!(clamp(0.2, 0.0, 1.0), 0.2);
        assert_eq!(clamp(-2.0, 0.0, 1.0), 0.0);
        assert_eq!(clamp(2.0, 0.0, 1.0), 1.0);
    }

    #[test]
    fn voice_assignment_round_trip() {
        let mut client = TtsClient::new(TtsConfig::default());
        client.set_voice_for_agent(
            "agent-1",
            VoiceConfig {
                provider: TtsProvider::OpenAi,
                voice_id: "alloy".to_string(),
                speed: 1.25,
                stability: 0.5,
                similarity_boost: 0.8,
                style: 0.2,
                use_speaker_boost: false,
            },
        );

        let voice = client
            .voice_for_agent("agent-1")
            .expect("voice should be available");
        assert_eq!(voice.voice_id, "alloy");
        assert_eq!(voice.provider, TtsProvider::OpenAi);
    }

    #[tokio::test]
    async fn synth_for_agent_requires_mapping() {
        let client = TtsClient::new(TtsConfig::default());
        let err = client
            .synthesize_for_agent("missing", "hello", None)
            .await
            .expect_err("missing voice should error");
        assert!(format!("{err}").contains("no voice configured"));
    }

    #[tokio::test]
    async fn elevenlabs_synthesis_requires_key() {
        let client = TtsClient::new(TtsConfig {
            elevenlabs_api_key: None,
            ..TtsConfig::default()
        });
        let voice = VoiceConfig {
            provider: TtsProvider::ElevenLabs,
            voice_id: "voice-id".to_string(),
            speed: 1.0,
            stability: 0.5,
            similarity_boost: 0.5,
            style: 0.0,
            use_speaker_boost: false,
        };
        let result = client
            .synthesize_elevenlabs(&voice, "hello", OutputFormat::Mp3)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn openai_synthesis_requires_key() {
        let client = TtsClient::new(TtsConfig {
            openai_api_key: None,
            ..TtsConfig::default()
        });
        let voice = VoiceConfig {
            provider: TtsProvider::OpenAi,
            voice_id: "alloy".to_string(),
            speed: 1.0,
            stability: 0.5,
            similarity_boost: 0.5,
            style: 0.0,
            use_speaker_boost: false,
        };
        let result = client
            .synthesize_openai(&voice, "hello", OutputFormat::Mp3)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn list_agent_voices_returns_entries() {
        let mut config = TtsConfig::default();
        config.agent_voice_configs.insert(
            "agent-a".to_string(),
            VoiceConfig {
                provider: TtsProvider::OpenAi,
                voice_id: "alloy".to_string(),
                speed: 1.0,
                stability: 0.5,
                similarity_boost: 0.5,
                style: 0.0,
                use_speaker_boost: false,
            },
        );
        let client = TtsClient::new(config);
        let listing = client.list_agent_voices();
        assert_eq!(listing.len(), 1);
    }
}

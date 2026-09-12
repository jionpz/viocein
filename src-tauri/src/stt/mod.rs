pub mod apple_speech;
pub mod capabilities;
pub mod config;
pub mod whisper_compat;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

use whisper_compat::{WhisperCompatConfig, WhisperCompatProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub api_key: String,
    pub language: Option<String>,
    pub smart_format: bool,
    pub sample_rate: u32,
    pub resource_id: Option<String>,
    pub operation_id: Option<String>,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            language: None,
            smart_format: true,
            sample_rate: 16000,
            resource_id: None,
            operation_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TranscriptEvent {
    Partial { text: String },
    Final { text: String, confidence: f32 },
    SpeechStarted,
    SpeechEnded,
    Error { message: String },
}

#[async_trait]
pub trait SttProvider: Send + Sync {
    async fn connect(&mut self, config: &SttConfig) -> Result<(), AppError>;
    async fn send_audio(&mut self, chunk: &[u8]) -> Result<(), AppError>;
    async fn recv_transcript(&mut self) -> Result<Option<TranscriptEvent>, AppError>;
    /// Disconnect and optionally return a final transcript (for file-based providers).
    async fn disconnect(&mut self) -> Result<Option<String>, AppError>;
    fn recording_limit_override_seconds(&self) -> Option<u32> {
        None
    }
    fn recording_limit_override_explanation_key(&self) -> Option<&'static str> {
        None
    }
    fn name(&self) -> &str;
}

pub fn create_provider(
    provider_name: &str,
    custom_whisper_config: Option<WhisperCompatConfig>,
    client: Option<reqwest::Client>,
) -> Result<Box<dyn SttProvider>, AppError> {
    match provider_name {
        apple_speech::APPLE_SPEECH_PROVIDER => {
            Ok(Box::new(apple_speech::AppleSpeechProvider::new()))
        }
        config::CUSTOM_WHISPER_PROVIDER => {
            let wc = custom_whisper_config.ok_or_else(|| {
                AppError::Config("Local / Custom Whisper is missing base URL or model".to_string())
            })?;
            // Defence in depth: refuse to construct a provider whose endpoint is
            // not loopback, even if the config was tampered with on disk.
            crate::egress::validate_loopback_url(&wc.endpoint).map_err(AppError::Config)?;
            Ok(match client {
                Some(ref c) => Box::new(WhisperCompatProvider::with_client(wc, c.clone())),
                None => Box::new(WhisperCompatProvider::new(wc)),
            })
        }
        name => Err(AppError::Config(format!(
            "Unknown or disabled STT provider: {name}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_whisper_requires_explicit_config() {
        let result = create_provider(config::CUSTOM_WHISPER_PROVIDER, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn custom_whisper_uses_explicit_config() {
        let cfg = config::build_custom_whisper_config(
            "http://localhost:8000/v1",
            "Systran/faster-whisper-large-v3",
        )
        .unwrap();

        let provider = create_provider(config::CUSTOM_WHISPER_PROVIDER, Some(cfg), None).unwrap();
        assert_eq!(provider.name(), config::CUSTOM_WHISPER_PROVIDER);
    }

    #[test]
    fn creates_apple_speech_builtin_local_provider() {
        let provider = create_provider("apple-speech", None, None).unwrap();
        assert_eq!(provider.name(), "Apple Speech");
    }

    #[test]
    fn unknown_stt_provider_returns_error() {
        let result = create_provider("not-a-provider", None, None);
        assert!(result.is_err());
    }
}

//! Recording-limit capability registry for the internal local-only build.
//!
//! Only on-device STT providers are supported, so the registry is a small
//! static table. Remote/managed capability negotiation has been removed.

use crate::storage::AppConfig;
use serde::{Deserialize, Serialize};

pub const CAPABILITY_REGISTRY_VERSION: u32 = 1;
pub const CLIENT_FILE_BUFFER_BYTES: u64 = 24 * 1024 * 1024;
const MIN_CUSTOM_SECONDS: u32 = 30;
const FALLBACK_SECONDS: u32 = 30;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingLimitMode {
    #[default]
    Auto,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SttTransport {
    FileUpload,
    LocalBuffered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingLimitSource {
    Provider,
    ClientBuffer,
    UnknownUpstream,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SttRecordingCapability {
    pub registry_version: u32,
    pub provider_id: String,
    pub transport: SttTransport,
    pub recommended_max_seconds: u32,
    pub hard_max_seconds: u32,
    pub max_upload_bytes: Option<u64>,
    pub source: RecordingLimitSource,
    pub explanation_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRecordingLimit {
    pub capability: SttRecordingCapability,
    pub mode: RecordingLimitMode,
    pub requested_seconds: u32,
    pub effective_max_seconds: u32,
}

fn capability(
    provider_id: &str,
    transport: SttTransport,
    recommended_max_seconds: u32,
    hard_max_seconds: u32,
    max_upload_bytes: Option<u64>,
    source: RecordingLimitSource,
    explanation_key: &str,
) -> SttRecordingCapability {
    SttRecordingCapability {
        registry_version: CAPABILITY_REGISTRY_VERSION,
        provider_id: provider_id.to_string(),
        transport,
        recommended_max_seconds,
        hard_max_seconds,
        max_upload_bytes,
        source,
        explanation_key: explanation_key.to_string(),
    }
}

/// Static capability for a local STT provider. Unknown providers fall back to a
/// conservative 30-second limit instead of inheriting a remote provider policy.
fn static_provider_capability(provider_id: &str) -> SttRecordingCapability {
    match provider_id {
        crate::stt::config::APPLE_SPEECH_PROVIDER => capability(
            provider_id,
            SttTransport::LocalBuffered,
            60,
            60,
            None,
            RecordingLimitSource::Provider,
            "recordingLimits.reasons.appleSpeech",
        ),
        crate::stt::config::CUSTOM_WHISPER_PROVIDER => capability(
            provider_id,
            SttTransport::FileUpload,
            120,
            720,
            Some(CLIENT_FILE_BUFFER_BYTES),
            RecordingLimitSource::UnknownUpstream,
            "recordingLimits.reasons.unknownUpstream",
        ),
        _ => capability(
            provider_id,
            SttTransport::FileUpload,
            FALLBACK_SECONDS,
            FALLBACK_SECONDS,
            Some(CLIENT_FILE_BUFFER_BYTES),
            RecordingLimitSource::UnknownUpstream,
            "recordingLimits.reasons.unknownProvider",
        ),
    }
}

pub fn resolve_recording_limit(
    config: &AppConfig,
    _managed_state: Option<&()>,
    _now_unix_seconds: i64,
) -> ResolvedRecordingLimit {
    let capability = static_provider_capability(&config.stt_provider);
    let requested_seconds = match config.recording_limit_mode {
        RecordingLimitMode::Auto => capability.recommended_max_seconds,
        RecordingLimitMode::Custom => config.custom_recording_limit_seconds,
    };
    let effective_max_seconds = match config.recording_limit_mode {
        RecordingLimitMode::Auto => requested_seconds.min(capability.hard_max_seconds),
        RecordingLimitMode::Custom => requested_seconds
            .max(MIN_CUSTOM_SECONDS)
            .min(capability.hard_max_seconds),
    };

    ResolvedRecordingLimit {
        capability,
        mode: config.recording_limit_mode,
        requested_seconds,
        effective_max_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::AppConfig;

    fn config(provider: &str, mode: RecordingLimitMode, custom_seconds: u32) -> AppConfig {
        AppConfig {
            stt_provider: provider.to_string(),
            recording_limit_mode: mode,
            custom_recording_limit_seconds: custom_seconds,
            ..AppConfig::default()
        }
    }

    #[test]
    fn local_provider_capabilities_cover_the_supported_matrix() {
        let apple = resolve_recording_limit(
            &config("apple-speech", RecordingLimitMode::Auto, 600),
            None,
            0,
        );
        assert_eq!(apple.capability.transport, SttTransport::LocalBuffered);
        assert_eq!(apple.capability.recommended_max_seconds, 60);
        assert_eq!(apple.capability.hard_max_seconds, 60);
        assert_eq!(apple.capability.source, RecordingLimitSource::Provider);
        assert_eq!(apple.effective_max_seconds, 60);

        let whisper = resolve_recording_limit(
            &config("custom-whisper", RecordingLimitMode::Auto, 600),
            None,
            0,
        );
        assert_eq!(whisper.capability.transport, SttTransport::FileUpload);
        assert_eq!(whisper.capability.recommended_max_seconds, 120);
        assert_eq!(whisper.capability.hard_max_seconds, 720);
        assert_eq!(
            whisper.capability.source,
            RecordingLimitSource::UnknownUpstream
        );
        assert_eq!(whisper.capability.max_upload_bytes, Some(24 * 1024 * 1024));
        assert_eq!(whisper.effective_max_seconds, 120);
    }

    #[test]
    fn custom_mode_clamps_to_the_safe_range() {
        let too_low = resolve_recording_limit(
            &config("custom-whisper", RecordingLimitMode::Custom, 1),
            None,
            0,
        );
        let too_high = resolve_recording_limit(
            &config("custom-whisper", RecordingLimitMode::Custom, 9_999),
            None,
            0,
        );

        assert_eq!(too_low.effective_max_seconds, 30);
        assert_eq!(too_high.effective_max_seconds, 720);
    }

    #[test]
    fn unknown_or_remote_provider_uses_a_conservative_fallback() {
        for provider in ["future-provider", "cloud", "deepgram", "groq-whisper"] {
            let resolved =
                resolve_recording_limit(&config(provider, RecordingLimitMode::Auto, 600), None, 0);

            assert_eq!(resolved.capability.provider_id, provider);
            assert_eq!(resolved.capability.transport, SttTransport::FileUpload);
            assert_eq!(
                resolved.capability.source,
                RecordingLimitSource::UnknownUpstream
            );
            assert_eq!(resolved.capability.hard_max_seconds, 30);
            assert_eq!(resolved.effective_max_seconds, 30);
        }
    }
}

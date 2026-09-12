/// Shared STT provider configuration constants for the internal local-only build.
///
/// The internal edition only supports speech recognition that stays on the
/// machine: Apple's on-device Speech framework, or a Whisper-compatible server
/// running on loopback. Remote STT providers are intentionally not compiled in.
use super::whisper_compat::WhisperCompatConfig;

pub const APPLE_SPEECH_PROVIDER: &str = "apple-speech";
pub const CUSTOM_WHISPER_PROVIDER: &str = "custom-whisper";
pub const CUSTOM_WHISPER_PRESET_SPEACHES: &str = "speaches";
pub const CUSTOM_WHISPER_PRESET_CUSTOM: &str = "custom";
pub const DEFAULT_CUSTOM_WHISPER_BASE_URL: &str = "http://localhost:8000/v1";
pub const DEFAULT_CUSTOM_WHISPER_MODEL: &str = "Systran/faster-whisper-large-v3";

/// Normalize a local Whisper-compatible server URL and enforce loopback only.
pub fn normalize_custom_whisper_endpoint(base_url: &str) -> Result<String, String> {
    let mut parsed = crate::egress::parse_loopback_base_url(base_url)?;

    let normalized_path = parsed.path().trim_end_matches('/').to_string();
    if normalized_path.ends_with("/audio/transcriptions") {
        parsed.set_path(&normalized_path);
    } else {
        parsed.set_path(&format!("{normalized_path}/audio/transcriptions"));
    }

    Ok(parsed.to_string())
}

pub fn build_custom_whisper_config(
    base_url: &str,
    model: &str,
) -> Result<WhisperCompatConfig, String> {
    let model = model.trim();
    if model.is_empty() {
        return Err("Model is required for Local / Custom Whisper".to_string());
    }

    Ok(WhisperCompatConfig {
        provider_name: CUSTOM_WHISPER_PROVIDER.to_string(),
        endpoint: normalize_custom_whisper_endpoint(base_url)?,
        model: model.to_string(),
        extra_fields: vec![],
        api_key_required: false,
    })
}

pub fn stt_provider_requires_api_key(provider: &str) -> bool {
    !matches!(provider, CUSTOM_WHISPER_PROVIDER | APPLE_SPEECH_PROVIDER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_providers_do_not_require_api_keys() {
        assert!(!stt_provider_requires_api_key(APPLE_SPEECH_PROVIDER));
        assert!(!stt_provider_requires_api_key(CUSTOM_WHISPER_PROVIDER));
        assert!(stt_provider_requires_api_key("deepgram"));
    }

    #[test]
    fn custom_whisper_endpoint_is_appended() {
        let endpoint = normalize_custom_whisper_endpoint("http://localhost:8000/v1").unwrap();
        assert_eq!(endpoint, "http://localhost:8000/v1/audio/transcriptions");
    }

    #[test]
    fn custom_whisper_full_endpoint_is_preserved() {
        let endpoint =
            normalize_custom_whisper_endpoint("http://localhost:8000/v1/audio/transcriptions")
                .unwrap();
        assert_eq!(endpoint, "http://localhost:8000/v1/audio/transcriptions");
    }

    #[test]
    fn custom_whisper_query_is_preserved() {
        let endpoint = normalize_custom_whisper_endpoint(
            "http://127.0.0.1:8000/v1/audio/transcriptions?api-version=2026-01-01",
        )
        .unwrap();
        assert_eq!(
            endpoint,
            "http://127.0.0.1:8000/v1/audio/transcriptions?api-version=2026-01-01"
        );
    }

    #[test]
    fn custom_whisper_rejects_remote_and_credentialed_hosts() {
        let remote = normalize_custom_whisper_endpoint("https://api.openai.com/v1").unwrap_err();
        assert!(remote.contains("remote hosts are blocked"));

        let disguised =
            normalize_custom_whisper_endpoint("https://localhost.evil.com/v1").unwrap_err();
        assert!(disguised.contains("remote hosts are blocked"));

        let credentials =
            normalize_custom_whisper_endpoint("http://user:secret@localhost:8000/v1").unwrap_err();
        assert!(credentials.contains("credentials"));
    }

    #[test]
    fn custom_whisper_rejects_empty_and_non_http_urls() {
        assert!(normalize_custom_whisper_endpoint("   ").is_err());
        assert!(normalize_custom_whisper_endpoint("file:///tmp/server").is_err());
    }

    #[test]
    fn custom_whisper_requires_model() {
        let error = build_custom_whisper_config("http://localhost:8000/v1", "  ").unwrap_err();
        assert!(error.contains("Model is required"));
    }

    #[test]
    fn custom_whisper_builds_loopback_config() {
        let cfg = build_custom_whisper_config(
            "http://127.0.0.1:8000/v1",
            "Systran/faster-whisper-large-v3",
        )
        .unwrap();
        assert_eq!(cfg.provider_name, CUSTOM_WHISPER_PROVIDER);
        assert_eq!(
            cfg.endpoint,
            "http://127.0.0.1:8000/v1/audio/transcriptions"
        );
        assert!(!cfg.api_key_required);
    }
}

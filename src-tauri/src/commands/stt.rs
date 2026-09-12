use crate::credentials::{resolve_config_secret, SystemCredentialVault};
use crate::storage;
use crate::stt;

/// The internal edition only permits on-device STT providers.
///
/// Keep this check in Rust, not only in the renderer: hidden UI is not a
/// security boundary and Tauri commands are still callable from the webview.
fn ensure_local_stt_provider(provider: &str) -> Result<(), String> {
    match provider {
        stt::config::CUSTOM_WHISPER_PROVIDER | stt::config::APPLE_SPEECH_PROVIDER => Ok(()),
        _ => Err(format!(
            "Remote STT provider is disabled in the internal local-only build: {provider}"
        )),
    }
}

#[tauri::command]
pub async fn get_stt_recording_capability(
    state: tauri::State<'_, storage::ConfigManager>,
    provider: String,
    mode: stt::capabilities::RecordingLimitMode,
    custom_seconds: u32,
) -> Result<stt::capabilities::ResolvedRecordingLimit, String> {
    ensure_local_stt_provider(&provider)?;
    let mut config = state.load().await.map_err(|error| error.to_string())?;
    config.stt_provider = provider;
    config.recording_limit_mode = mode;
    config.custom_recording_limit_seconds = custom_seconds;
    Ok(stt::capabilities::resolve_recording_limit(
        &config,
        None,
        chrono::Utc::now().timestamp(),
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SttProviderDiagnosticIssue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SttProviderDiagnostics {
    pub provider: String,
    pub kind: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub requires_api_key: bool,
    pub api_key_configured: bool,
    pub ready: bool,
    pub issues: Vec<SttProviderDiagnosticIssue>,
}

fn diagnostic_issue(code: &str, message: impl Into<String>) -> SttProviderDiagnosticIssue {
    SttProviderDiagnosticIssue {
        code: code.to_string(),
        message: message.into(),
    }
}

fn disabled_provider_diagnostics(provider: &str) -> SttProviderDiagnostics {
    SttProviderDiagnostics {
        provider: provider.to_string(),
        kind: "disabledRemote".to_string(),
        endpoint: None,
        model: None,
        requires_api_key: false,
        api_key_configured: false,
        ready: false,
        issues: vec![diagnostic_issue(
            "provider_disabled",
            "Remote STT providers are disabled in the internal local-only build",
        )],
    }
}

fn build_apple_speech_diagnostics(
    provider: &str,
    availability: stt::apple_speech::AppleSpeechAvailability,
) -> SttProviderDiagnostics {
    SttProviderDiagnostics {
        provider: provider.to_string(),
        kind: "builtinLocal".to_string(),
        endpoint: None,
        model: Some(
            availability
                .locale
                .as_ref()
                .map(|locale| format!("Apple Speech ({locale})"))
                .unwrap_or_else(|| "Apple Speech".to_string()),
        ),
        requires_api_key: false,
        api_key_configured: false,
        ready: availability.ready,
        issues: match (availability.issue_code, availability.issue_message) {
            (Some(code), Some(message)) => vec![diagnostic_issue(&code, message)],
            (Some(code), None) => vec![diagnostic_issue(&code, code.clone())],
            _ => Vec::new(),
        },
    }
}

fn build_stt_provider_diagnostics(
    provider: &str,
    api_key: &str,
    custom_base_url: Option<&str>,
    custom_model: Option<&str>,
) -> SttProviderDiagnostics {
    match provider {
        "" => SttProviderDiagnostics {
            provider: provider.to_string(),
            kind: "unknown".to_string(),
            endpoint: None,
            model: None,
            requires_api_key: false,
            api_key_configured: false,
            ready: false,
            issues: vec![diagnostic_issue(
                "missing_provider",
                "No STT provider selected",
            )],
        },
        stt::config::APPLE_SPEECH_PROVIDER => build_apple_speech_diagnostics(
            provider,
            stt::apple_speech::apple_speech_availability(None),
        ),
        stt::config::CUSTOM_WHISPER_PROVIDER => {
            let api_key_configured = !api_key.trim().is_empty();
            match stt::config::build_custom_whisper_config(
                custom_base_url.unwrap_or_default(),
                custom_model.unwrap_or_default(),
            ) {
                Ok(cfg) => SttProviderDiagnostics {
                    provider: provider.to_string(),
                    kind: "localCompatible".to_string(),
                    endpoint: Some(cfg.endpoint),
                    model: Some(cfg.model),
                    requires_api_key: cfg.api_key_required,
                    api_key_configured,
                    ready: true,
                    issues: Vec::new(),
                },
                Err(err) => SttProviderDiagnostics {
                    provider: provider.to_string(),
                    kind: "localCompatible".to_string(),
                    endpoint: None,
                    model: custom_model
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToString::to_string),
                    requires_api_key: false,
                    api_key_configured,
                    ready: false,
                    issues: vec![diagnostic_issue("invalid_custom_whisper_config", err)],
                },
            }
        }
        _ => disabled_provider_diagnostics(provider),
    }
}

#[tauri::command]
pub fn get_stt_provider_diagnostics(
    api_key: String,
    provider: String,
    custom_base_url: Option<String>,
    custom_model: Option<String>,
    _provider_region: Option<String>,
) -> Result<SttProviderDiagnostics, String> {
    let resolved_api_key = if provider == stt::config::CUSTOM_WHISPER_PROVIDER {
        resolve_config_secret(&api_key, "stt", &provider, &SystemCredentialVault)
            .map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    Ok(build_stt_provider_diagnostics(
        &provider,
        &resolved_api_key,
        custom_base_url.as_deref(),
        custom_model.as_deref(),
    ))
}

fn resolve_whisper_test_config(
    provider: &str,
    custom_base_url: Option<String>,
    custom_model: Option<String>,
) -> Result<stt::whisper_compat::WhisperCompatConfig, String> {
    if provider != stt::config::CUSTOM_WHISPER_PROVIDER {
        return Err(format!(
            "Remote STT provider is disabled in the internal local-only build: {provider}"
        ));
    }

    stt::config::build_custom_whisper_config(
        custom_base_url.as_deref().unwrap_or_default(),
        custom_model.as_deref().unwrap_or_default(),
    )
}

async fn test_custom_whisper(
    client: &reqwest::Client,
    api_key: &str,
    custom_base_url: Option<String>,
    custom_model: Option<String>,
) -> Result<u32, String> {
    let cfg = resolve_whisper_test_config(
        stt::config::CUSTOM_WHISPER_PROVIDER,
        custom_base_url,
        custom_model,
    )?;

    let silent_pcm = vec![0u8; 3200]; // 0.1s at 16kHz 16-bit mono
    let wav = stt::whisper_compat::WhisperCompatProvider::build_wav(&silent_pcm, 16000);
    let file_part = reqwest::multipart::Part::bytes(wav)
        .file_name("test.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;
    let mut form = reqwest::multipart::Form::new()
        .text("model", cfg.model.clone())
        .part("file", file_part);

    for (key, value) in &cfg.extra_fields {
        form = form.text(key.clone(), value.clone());
    }

    let started = std::time::Instant::now();
    let mut request = client
        .post(&cfg.endpoint)
        .multipart(form)
        .timeout(std::time::Duration::from_secs(15));
    if !api_key.trim().is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    Ok(started.elapsed().as_millis() as u32)
}

#[tauri::command]
// Keep explicit IPC fields so older desktop callers remain wire-compatible.
#[allow(clippy::too_many_arguments)]
pub async fn test_stt_connection(
    api_key: String,
    provider: String,
    custom_base_url: Option<String>,
    custom_model: Option<String>,
    _volcengine_resource_id: Option<String>,
    _provider_region: Option<String>,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<bool, String> {
    if provider.is_empty() {
        return Ok(false);
    }
    ensure_local_stt_provider(&provider)?;

    match provider.as_str() {
        stt::config::APPLE_SPEECH_PROVIDER => {
            Ok(stt::apple_speech::is_available_on_current_platform())
        }
        stt::config::CUSTOM_WHISPER_PROVIDER => {
            let api_key = resolve_config_secret(&api_key, "stt", &provider, &SystemCredentialVault)
                .map_err(|e| e.to_string())?;
            test_custom_whisper(&client, &api_key, custom_base_url, custom_model)
                .await
                .map(|_| true)
        }
        _ => Err(format!(
            "Remote STT provider is disabled in the internal local-only build: {provider}"
        )),
    }
}

#[tauri::command]
// Keep explicit IPC fields so older desktop callers remain wire-compatible.
#[allow(clippy::too_many_arguments)]
pub async fn bench_stt_connection(
    api_key: String,
    provider: String,
    custom_base_url: Option<String>,
    custom_model: Option<String>,
    _volcengine_resource_id: Option<String>,
    _provider_region: Option<String>,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<u32, String> {
    if provider.is_empty() {
        return Err("No provider specified".to_string());
    }
    ensure_local_stt_provider(&provider)?;

    match provider.as_str() {
        stt::config::APPLE_SPEECH_PROVIDER => {
            if stt::apple_speech::is_available_on_current_platform() {
                Ok(0)
            } else {
                Err("Apple Speech is only available on macOS".to_string())
            }
        }
        stt::config::CUSTOM_WHISPER_PROVIDER => {
            let api_key = resolve_config_secret(&api_key, "stt", &provider, &SystemCredentialVault)
                .map_err(|e| e.to_string())?;
            test_custom_whisper(&client, &api_key, custom_base_url, custom_model).await
        }
        _ => Err(format!(
            "Remote STT provider is disabled in the internal local-only build: {provider}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_custom_whisper_test_config() {
        let cfg = resolve_whisper_test_config(
            stt::config::CUSTOM_WHISPER_PROVIDER,
            Some("http://localhost:8000/v1".to_string()),
            Some("Systran/faster-whisper-large-v3".to_string()),
        )
        .unwrap();
        assert_eq!(
            cfg.endpoint,
            "http://localhost:8000/v1/audio/transcriptions"
        );
        assert!(!cfg.api_key_required);
    }

    #[test]
    fn custom_whisper_test_config_requires_model() {
        let err = resolve_whisper_test_config(
            stt::config::CUSTOM_WHISPER_PROVIDER,
            Some("http://localhost:8000/v1".to_string()),
            Some(" ".to_string()),
        )
        .unwrap_err();
        assert!(err.contains("Model is required"));
    }

    #[test]
    fn custom_whisper_diagnostics_exposes_local_endpoint() {
        let diagnostics = build_stt_provider_diagnostics(
            stt::config::CUSTOM_WHISPER_PROVIDER,
            "",
            Some("http://localhost:8000/v1"),
            Some("Systran/faster-whisper-large-v3"),
        );

        assert_eq!(diagnostics.provider, stt::config::CUSTOM_WHISPER_PROVIDER);
        assert_eq!(diagnostics.kind, "localCompatible");
        assert_eq!(
            diagnostics.endpoint.as_deref(),
            Some("http://localhost:8000/v1/audio/transcriptions")
        );
        assert_eq!(
            diagnostics.model.as_deref(),
            Some("Systran/faster-whisper-large-v3")
        );
        assert!(!diagnostics.requires_api_key);
        assert!(diagnostics.ready);
        assert!(diagnostics.issues.is_empty());
    }

    #[test]
    fn custom_whisper_diagnostics_reports_invalid_config() {
        let diagnostics = build_stt_provider_diagnostics(
            stt::config::CUSTOM_WHISPER_PROVIDER,
            "",
            Some("file:///tmp/server"),
            Some(" "),
        );

        assert_eq!(diagnostics.kind, "localCompatible");
        assert!(!diagnostics.ready);
        assert_eq!(diagnostics.issues.len(), 1);
        assert_eq!(diagnostics.issues[0].code, "invalid_custom_whisper_config");
    }

    #[test]
    fn remote_stt_diagnostics_report_disabled_provider_without_endpoint() {
        let diagnostics = build_stt_provider_diagnostics(
            "deepgram",
            "",
            Some("https://api.deepgram.com/v1"),
            Some("nova-3"),
        );

        assert_eq!(diagnostics.kind, "disabledRemote");
        assert_eq!(diagnostics.endpoint, None);
        assert_eq!(diagnostics.model, None);
        assert!(!diagnostics.ready);
        assert_eq!(diagnostics.issues[0].code, "provider_disabled");
    }

    #[test]
    fn apple_speech_diagnostics_are_platform_gated_builtin_local() {
        let diagnostics = build_stt_provider_diagnostics("apple-speech", "", None, None);

        assert_eq!(diagnostics.provider, "apple-speech");
        assert_eq!(diagnostics.kind, "builtinLocal");
        assert!(!diagnostics.requires_api_key);
        assert!(!diagnostics.api_key_configured);
        assert_eq!(diagnostics.endpoint, None);
        assert_eq!(diagnostics.model.as_deref(), Some("Apple Speech"));

        #[cfg(target_os = "macos")]
        {
            if diagnostics.ready {
                assert!(diagnostics.issues.is_empty());
            } else {
                assert!(!diagnostics.issues.is_empty());
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            assert!(!diagnostics.ready);
            assert_eq!(diagnostics.issues[0].code, "unsupported_platform");
        }
    }

    #[test]
    fn apple_speech_diagnostics_reports_authorization_issue() {
        let diagnostics = build_apple_speech_diagnostics(
            "apple-speech",
            stt::apple_speech::AppleSpeechAvailability::from_parts(
                true,
                stt::apple_speech::AppleSpeechAuthorizationStatus::Denied,
                Some("en-US".to_string()),
                None,
            ),
        );

        assert!(!diagnostics.ready);
        assert_eq!(diagnostics.kind, "builtinLocal");
        assert_eq!(diagnostics.issues.len(), 1);
        assert_eq!(diagnostics.issues[0].code, "speech_permission_denied");
    }
}

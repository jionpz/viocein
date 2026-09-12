//! OpenAI-compatible wire protocol helpers.
//!
//! The internal build only talks to an OpenAI-compatible company gateway or a
//! local Ollama endpoint, so the Anthropic / native-provider special cases have
//! been removed.

use reqwest::RequestBuilder;
use serde_json::{json, Value};
use std::time::Duration;

/// Kept as a single-variant enum so call sites stay explicit about the wire
/// protocol they speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmApiKind {
    OpenAiCompatible,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct StreamEvent {
    pub text: Option<String>,
    pub reasoning: Option<String>,
    pub error: Option<String>,
    pub done: bool,
}

pub fn detect_api_kind(_provider: &str, _base_url: &str) -> LlmApiKind {
    LlmApiKind::OpenAiCompatible
}

fn parse_http_url(base_url: &str) -> Result<url::Url, String> {
    let mut url = url::Url::parse(base_url.trim())
        .map_err(|error| format!("Invalid LLM base URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("LLM base URL must use http or https scheme".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("LLM base URL must not include credentials".to_string());
    }
    if url.fragment().is_some() {
        return Err("LLM base URL must not include a fragment".to_string());
    }
    url.set_fragment(None);
    Ok(url)
}

fn replace_or_append_path(url: &mut url::Url, current_suffix: &str, target_suffix: &str) {
    let path = url.path().trim_end_matches('/');
    let root = path.strip_suffix(current_suffix).unwrap_or(path);
    url.set_path(&format!("{root}{target_suffix}"));
}

pub fn chat_endpoint(_provider: &str, base_url: &str) -> Result<String, String> {
    let mut url = parse_http_url(base_url)?;
    let path = url.path().trim_end_matches('/');
    if !path.ends_with("/chat/completions") {
        url.set_path(&format!("{path}/chat/completions"));
    }
    Ok(url.to_string())
}

pub fn models_endpoint(_provider: &str, base_url: &str) -> Result<String, String> {
    let mut url = parse_http_url(base_url)?;
    replace_or_append_path(&mut url, "/chat/completions", "/models");
    Ok(url.to_string())
}

fn is_reasoning_model_without_sampling_controls(model: &str) -> bool {
    let model = model.trim().to_ascii_lowercase();
    model.starts_with("gpt-5")
        || model == "o1"
        || model.starts_with("o1-")
        || model == "o3"
        || model.starts_with("o3-")
        || model == "o4"
        || model.starts_with("o4-")
}

pub fn request_timeout(_provider: &str, _base_url: &str, model: &str) -> Duration {
    if is_reasoning_model_without_sampling_controls(model) {
        Duration::from_secs(60)
    } else {
        Duration::from_secs(30)
    }
}

pub fn build_chat_body(
    _provider: &str,
    _base_url: &str,
    model: &str,
    messages: Vec<Value>,
    max_tokens: u32,
    temperature: f64,
    stream: bool,
) -> Value {
    let mut body = json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "stream": stream
    });
    let object = body.as_object_mut().expect("chat body is an object");
    if !is_reasoning_model_without_sampling_controls(model) {
        object.insert("temperature".to_string(), json!(temperature));
    }
    body
}

pub fn apply_auth_headers(
    request: RequestBuilder,
    provider: &str,
    _base_url: &str,
    api_key: &str,
) -> RequestBuilder {
    let api_key = api_key.trim();
    if super::provider_requires_api_key(provider) || !api_key.is_empty() {
        request.header("Authorization", format!("Bearer {api_key}"))
    } else {
        request
    }
}

pub fn response_text(_kind: LlmApiKind, body: &Value) -> String {
    let message = &body["choices"][0]["message"];
    message["content"]
        .as_str()
        .filter(|content| !content.is_empty())
        .or_else(|| message["reasoning_content"].as_str())
        .unwrap_or("")
        .to_string()
}

pub fn parse_stream_event(_kind: LlmApiKind, body: &Value) -> StreamEvent {
    if body["error"].is_object() {
        return StreamEvent {
            error: body["error"]["message"].as_str().map(str::to_string),
            ..StreamEvent::default()
        };
    }
    let delta = &body["choices"][0]["delta"];
    StreamEvent {
        text: delta["content"].as_str().map(str::to_string),
        reasoning: delta["reasoning_content"].as_str().map(str::to_string),
        ..StreamEvent::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages() -> Vec<Value> {
        vec![
            json!({"role": "system", "content": "Be concise."}),
            json!({"role": "user", "content": "Hello"}),
        ]
    }

    #[test]
    fn chat_endpoint_appends_completions_once() {
        assert_eq!(
            chat_endpoint("company", "https://llm.example.internal/v1").unwrap(),
            "https://llm.example.internal/v1/chat/completions"
        );
        assert_eq!(
            chat_endpoint(
                "company",
                "https://llm.example.internal/v1/chat/completions"
            )
            .unwrap(),
            "https://llm.example.internal/v1/chat/completions"
        );
        assert_eq!(
            models_endpoint(
                "company",
                "https://llm.example.internal/v1/chat/completions"
            )
            .unwrap(),
            "https://llm.example.internal/v1/models"
        );
    }

    #[test]
    fn chat_endpoint_rejects_non_http_and_credentialed_urls() {
        assert!(chat_endpoint("company", "file:///tmp/socket").is_err());
        assert!(chat_endpoint("company", "https://user:pass@llm.example.internal/v1").is_err());
    }

    #[test]
    fn chat_body_sends_openai_compatible_fields() {
        let body = build_chat_body(
            "company",
            "https://llm.example.internal/v1",
            "company-model",
            messages(),
            4096,
            0.3,
            false,
        );

        assert_eq!(body["model"], "company-model");
        assert_eq!(body["max_tokens"], 4096);
        assert_eq!(body["temperature"], 0.3);
        assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn reasoning_models_omit_temperature_and_get_a_longer_timeout() {
        let body = build_chat_body(
            "company",
            "https://llm.example.internal/v1",
            "gpt-5",
            messages(),
            4096,
            0.3,
            false,
        );

        assert!(body.get("temperature").is_none());
        assert_eq!(
            request_timeout("company", "https://llm.example.internal/v1", "gpt-5"),
            Duration::from_secs(60)
        );
        assert_eq!(
            request_timeout("company", "https://llm.example.internal/v1", "company-chat"),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn auth_header_is_sent_only_when_a_key_is_present_or_required() {
        let with_key = apply_auth_headers(
            reqwest::Client::new().post("https://llm.example.internal/v1/chat/completions"),
            "company",
            "https://llm.example.internal/v1",
            "company-key",
        )
        .build()
        .unwrap();
        assert_eq!(with_key.headers()["Authorization"], "Bearer company-key");

        let keyless_ollama = apply_auth_headers(
            reqwest::Client::new().post("http://127.0.0.1:11434/v1/chat/completions"),
            "ollama",
            "http://127.0.0.1:11434/v1",
            "",
        )
        .build()
        .unwrap();
        assert!(keyless_ollama.headers().get("Authorization").is_none());
    }

    #[test]
    fn response_and_stream_parsers_read_openai_shapes() {
        let response = json!({
            "choices": [{"message": {"content": "Hello world"}}]
        });
        assert_eq!(
            response_text(LlmApiKind::OpenAiCompatible, &response),
            "Hello world"
        );

        let event = parse_stream_event(
            LlmApiKind::OpenAiCompatible,
            &json!({"choices": [{"delta": {"content": "Hello"}}]}),
        );
        assert_eq!(event.text.as_deref(), Some("Hello"));
        assert!(!event.done);

        let error = parse_stream_event(
            LlmApiKind::OpenAiCompatible,
            &json!({"error": {"message": "bad request"}}),
        );
        assert_eq!(error.error.as_deref(), Some("bad request"));
    }
}

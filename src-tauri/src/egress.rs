//! Central network egress policy.
//!
//! The internal edition is local-first. Only two kinds of network destinations
//! are permitted:
//!
//! 1. The company OpenAI-compatible LLM gateway the operator configures in
//!    Settings. The configured origin plus base path become that provider's
//!    allowlist.
//! 2. Loopback (`localhost` / `127.0.0.0/8` / `[::1]`) for a local STT server.
//!
//! No third-party endpoint is compiled into the binary. Anything that does not
//! match one of the rules above is rejected before the request leaves the
//! process. This module is intentionally independent of the UI: hiding a
//! provider in the frontend is not a security control, refusing the request in
//! Rust is.

use url::Url;

/// Build an HTTP client that never follows redirects.
///
/// URL validation only sees the URL the application constructs. Following a
/// server redirect would perform a second request to an unvalidated
/// destination, so the network policy must stop at that boundary as well.
pub fn no_redirect_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
}

pub fn no_redirect_client() -> reqwest::Client {
    no_redirect_client_builder()
        .build()
        .expect("Failed to create HTTP client")
}

fn is_loopback_host(host: &str) -> bool {
    let host = host.trim().trim_start_matches('[').trim_end_matches(']');
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return ip.is_loopback();
    }
    false
}

/// Parse and validate a company gateway base URL supplied by the operator.
///
/// Both http and https are accepted because some internal gateways terminate
/// TLS elsewhere. The parsed origin is what an outbound request is checked
/// against, so a malformed base URL never becomes a silent wildcard.
pub fn parse_gateway_base_url(raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Company LLM base URL is not configured".to_string());
    }
    let mut url =
        Url::parse(trimmed).map_err(|error| format!("Invalid company LLM base URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Company LLM base URL must use http or https".to_string());
    }
    if url.host_str().is_none() {
        return Err("Company LLM base URL must include a host".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Company LLM base URL must not include credentials".to_string());
    }
    if url.fragment().is_some() {
        return Err("Company LLM base URL must not include a fragment".to_string());
    }
    url.set_fragment(None);
    Ok(url)
}

/// Require that an outbound company-provider URL stays on the configured gateway.
///
/// The endpoint must be same-origin with the configured base URL and its path
/// must live under the configured base path. Callers must use
/// [`no_redirect_client_builder`] or [`no_redirect_client`] so a response cannot
/// redirect the request to an unvalidated destination mid-flight.
pub fn validate_gateway_endpoint(base_url: &str, endpoint: &str) -> Result<(), String> {
    let base = parse_gateway_base_url(base_url)?;
    let url = parse_gateway_base_url(endpoint)?;
    if same_origin(&url, &base) && path_is_under(&url, &base) {
        Ok(())
    } else {
        let host = url.host_str().unwrap_or("unknown");
        Err(format!(
            "Blocked LLM destination {host}: the company provider may only call its configured origin and base path"
        ))
    }
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left
            .host_str()
            .zip(right.host_str())
            .is_some_and(|(left, right)| left.eq_ignore_ascii_case(right))
        && left.port_or_known_default() == right.port_or_known_default()
}

fn path_is_under(candidate: &Url, base: &Url) -> bool {
    let base_path = base.path().trim_end_matches('/');
    if base_path.is_empty() || base_path == "/" {
        return true;
    }

    candidate.path() == base_path
        || candidate
            .path()
            .strip_prefix(base_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

/// Parse a user-supplied local STT/custom-Whisper base URL and require loopback.
pub fn parse_loopback_base_url(raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Base URL is required".to_string());
    }
    let mut url = Url::parse(trimmed).map_err(|_| "Base URL must be a valid URL".to_string())?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Base URL must start with http:// or https://".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Base URL must not include credentials".to_string());
    }
    if url.fragment().is_some() {
        return Err("Base URL must not include a fragment".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "Base URL must include a host".to_string())?;
    if !is_loopback_host(host) {
        return Err(
            "Local STT must point at localhost / 127.0.0.1 / [::1]; remote hosts are blocked"
                .to_string(),
        );
    }
    url.set_fragment(None);
    Ok(url)
}

/// Require that a URL is plain loopback (used by local STT).
pub fn validate_loopback_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw.trim()).map_err(|error| format!("Invalid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!("Blocked URL scheme: {}", url.scheme()));
    }
    let host = url
        .host_str()
        .ok_or_else(|| "URL must include a host".to_string())?;
    if is_loopback_host(host) {
        Ok(())
    } else {
        Err(format!(
            "Blocked STT destination {host}: local STT must use loopback"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GATEWAY: &str = "https://llm.corp.example/openai/v1";

    #[test]
    fn loopback_hosts_are_recognized() {
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("LOCALHOST"));
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.8.9.10"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("[::1]"));
        assert!(!is_loopback_host("localhost.evil.com"));
        assert!(!is_loopback_host("192.168.1.5"));
    }

    #[test]
    fn gateway_endpoint_must_match_configured_origin_and_path() {
        assert!(validate_gateway_endpoint(
            GATEWAY,
            "https://llm.corp.example/openai/v1/chat/completions"
        )
        .is_ok());
        assert!(validate_gateway_endpoint(GATEWAY, "https://llm.corp.example/openai/v1").is_ok());
        // Same host, sibling path outside the configured base path.
        assert!(validate_gateway_endpoint(GATEWAY, "https://llm.corp.example/other/v1").is_err());
        // Lookalike hosts, scheme downgrade and prefix tricks stay blocked.
        assert!(
            validate_gateway_endpoint(GATEWAY, "https://llm.corp.example.evil.com/openai/v1")
                .is_err()
        );
        assert!(validate_gateway_endpoint(
            GATEWAY,
            "http://llm.corp.example/openai/v1/chat/completions"
        )
        .is_err());
        assert!(
            validate_gateway_endpoint(GATEWAY, "https://llm.corp.example.evil/openai/v1").is_err()
        );
        assert!(
            validate_gateway_endpoint(GATEWAY, "https://llm.corp.example/openai/v1-evil").is_err()
        );
        // Ports and unrelated destinations are not the configured origin.
        assert!(
            validate_gateway_endpoint(GATEWAY, "https://llm.corp.example:8443/openai/v1").is_err()
        );
        assert!(
            validate_gateway_endpoint(GATEWAY, "https://api.openai.com/v1/chat/completions")
                .is_err()
        );
        assert!(validate_gateway_endpoint(GATEWAY, "http://localhost:8000/v1").is_err());
    }

    #[test]
    fn gateway_base_url_must_be_http_or_https_and_credential_free() {
        assert!(parse_gateway_base_url("http://llm.corp.example/v1").is_ok());
        assert!(parse_gateway_base_url("ftp://llm.corp.example/v1").is_err());
        assert!(parse_gateway_base_url("https://user:secret@llm.corp.example/v1").is_err());
        assert!(parse_gateway_base_url("https://llm.corp.example/v1#frag").is_err());
        assert!(parse_gateway_base_url("   ").is_err());
        assert!(parse_gateway_base_url("llm.corp.example/v1").is_err());
    }

    #[test]
    fn unconfigured_gateway_fails_closed() {
        let error = validate_gateway_endpoint("", "https://llm.corp.example/v1").unwrap_err();
        assert!(error.contains("not configured"));
    }

    #[test]
    fn loopback_parser_rejects_remote_hosts() {
        assert!(parse_loopback_base_url("http://localhost:8000/v1").is_ok());
        assert!(parse_loopback_base_url("http://127.0.0.1:8000/v1").is_ok());
        let error = parse_loopback_base_url("https://api.openai.com/v1").unwrap_err();
        assert!(error.contains("remote hosts are blocked"));
        let error = parse_loopback_base_url("https://localhost.evil.com/v1").unwrap_err();
        assert!(error.contains("remote hosts are blocked"));
    }

    #[test]
    fn loopback_url_validation_rejects_company_host() {
        assert!(validate_loopback_url("http://localhost:8000/v1").is_ok());
        assert!(validate_loopback_url("http://[::1]:8000/v1").is_ok());
        assert!(validate_loopback_url("https://llm.corp.example/openai/v1").is_err());
        assert!(validate_loopback_url("https://api.openai.com/v1").is_err());
    }

    #[tokio::test]
    async fn no_redirect_client_surfaces_redirect_instead_of_following_it() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            stream
                .write_all(
                    b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:0/blocked\r\nContent-Length: 0\r\n\r\n",
                )
                .unwrap();
        });

        let response = no_redirect_client()
            .get(format!("http://{address}/start"))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::FOUND);
        server.join().unwrap();
    }
}

//! Shared HTTP client utilities for all providers
//!
//! Provides:
//! - A pre-configured `reqwest::Client` with a 30-second timeout
//! - One-shot retry logic for 5xx errors (powered by `hudhudscript_utils::RetryConfig`, Issue #692)
//! - A helper for detecting local/unauthenticated endpoints

use reqwest::{Client, RequestBuilder, Response};
use std::time::Duration;

use crate::provider::ProviderError;
use hudhudscript_utils::RetryConfig;

use crate::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS;

/// Retry configuration for provider HTTP calls: retry once after 1 second.
fn provider_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 1,
        base_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(1),
        multiplier: 1.0,
        jitter: false,
    }
}

/// Build a shared `reqwest::Client` with a 30-second timeout.
///
/// Call this once at provider construction time and store the result.
pub fn build_http_client() -> Result<Client, ProviderError> {
    Client::builder()
        .timeout(Duration::from_secs(DEFAULT_PROVIDER_TIMEOUT_SECS))
        .build()
        .map_err(|e| ProviderError::NetworkError(format!("Failed to build HTTP client: {}", e)))
}

/// PROVIDER0001: Process-wide shared HTTP client.
/// Connection pool, TLS context, DNS cache — all reused.
static SHARED_CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

pub fn shared_http_client() -> Result<Client, ProviderError> {
    if let Some(c) = SHARED_CLIENT.get() {
        return Ok(c.clone());
    }
    let c = build_http_client()?;
    let _ = SHARED_CLIENT.set(c.clone());
    Ok(c)
}

/// Returns `true` when the URL points to a local or private-network endpoint.
/// Uses proper IP parsing (fixes S6: 172.16/12 range, not 172.0/8).
pub fn is_local_url(url: &str) -> bool {
    let host = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .and_then(|rest| rest.split('/').next())
        .and_then(|h| h.split(':').next())
        .unwrap_or("");
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return ip.is_loopback() || ip.is_unspecified() || is_private_ip(&ip);
    }
    host == "localhost" || host.ends_with(".local") || host.ends_with(".internal")
}

/// Manual private IP check (std's is_private not available on all targets).
fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            octets[0] == 10
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168)
                || octets[0] == 169 && octets[1] == 254
        }
        std::net::IpAddr::V6(v6) => {
            v6.segments()[0] & 0xffc0 == 0xfe80 // link-local
        }
    }
}

/// Send `req` and retry once (after a 1-second delay) if the server returns a
/// 5xx status code.  All other errors and non-5xx responses are returned
/// immediately.
///
/// Uses [`RetryConfig`] from `hudhudscript-utils` for delay computation (Issue #692).
pub async fn send_with_retry(
    req: RequestBuilder,
    cloned: RequestBuilder,
) -> Result<Response, ProviderError> {
    let config = provider_retry_config();
    let response = req.send().await?;
    if response.status().is_server_error() {
        let delay = config.delay_for_attempt(0);
        tokio::time::sleep(delay).await;
        let retry_response = cloned.send().await?;
        Ok(retry_response)
    } else {
        Ok(response)
    }
}

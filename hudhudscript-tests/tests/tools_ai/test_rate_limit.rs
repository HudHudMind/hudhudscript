//! Tests extracted from hudhudscript-tools-ai/src/rate_limit.rs

use hudhudscript_tools_ai::rate_limit::default_rate_limits;
use hudhudscript_tools_ai::{Provider, ProviderRateLimit, RateLimitError, RateLimiter};
use std::collections::HashMap;

fn unlimited_limits() -> HashMap<Provider, ProviderRateLimit> {
    let mut m = HashMap::new();
    m.insert(Provider::OpenAI, ProviderRateLimit { rpm: 0, tpm: 0 });
    m
}

fn strict_limits() -> HashMap<Provider, ProviderRateLimit> {
    let mut m = HashMap::new();
    m.insert(Provider::OpenAI, ProviderRateLimit { rpm: 2, tpm: 100 });
    m
}

#[test]
fn test_unlimited_always_passes() {
    let limiter = RateLimiter::with_limits(unlimited_limits());
    for _ in 0..1000 {
        assert!(limiter.acquire(Provider::OpenAI, 9999).is_ok());
    }
}

#[test]
fn test_rpm_exceeded() {
    let limiter = RateLimiter::with_limits(strict_limits());
    assert!(limiter.acquire(Provider::OpenAI, 1).is_ok());
    assert!(limiter.acquire(Provider::OpenAI, 1).is_ok());
    // Third request should fail
    let result = limiter.acquire(Provider::OpenAI, 1);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RateLimitError::RpmExceeded { .. }
    ));
}

#[test]
fn test_tpm_exceeded() {
    let limiter = RateLimiter::with_limits(strict_limits());
    assert!(limiter.acquire(Provider::OpenAI, 50).is_ok());
    // Next 60 tokens would exceed 100 TPM
    let result = limiter.acquire(Provider::OpenAI, 60);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RateLimitError::TpmExceeded { .. }
    ));
}

#[test]
fn test_check_does_not_record() {
    let limiter = RateLimiter::with_limits(strict_limits());
    // Check only — should not consume a slot
    assert!(limiter.check(Provider::OpenAI, 1).is_ok());
    assert!(limiter.check(Provider::OpenAI, 1).is_ok());
    assert!(limiter.check(Provider::OpenAI, 1).is_ok());
    // Still can acquire because check didn't record
    assert!(limiter.acquire(Provider::OpenAI, 1).is_ok());
}

#[test]
fn test_current_counters() {
    let limiter = RateLimiter::with_limits(unlimited_limits());
    limiter.record(Provider::OpenAI, 42);
    limiter.record(Provider::OpenAI, 58);
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 2);
    assert_eq!(limiter.current_tpm(Provider::OpenAI), 100);
}

#[test]
fn test_reset() {
    let limiter = RateLimiter::with_limits(strict_limits());
    limiter.acquire(Provider::OpenAI, 1).unwrap();
    limiter.acquire(Provider::OpenAI, 1).unwrap();
    limiter.reset();
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 0);
    assert!(limiter.acquire(Provider::OpenAI, 1).is_ok());
}

#[test]
fn test_default_rate_limits_populated() {
    let limits = default_rate_limits();
    assert!(limits.contains_key(&Provider::OpenAI));
    assert!(limits.contains_key(&Provider::Anthropic));
    assert!(limits.contains_key(&Provider::Ollama));
    assert!(limits.contains_key(&Provider::DeepSeek));
}

#[test]
fn test_provider_rate_limit_default() {
    let limit = ProviderRateLimit::default();
    assert_eq!(limit.rpm, 60);
    assert_eq!(limit.tpm, 90_000);
}

#[test]
fn test_rate_limiter_default() {
    let limiter = RateLimiter::default();
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 0);
    assert_eq!(limiter.current_tpm(Provider::OpenAI), 0);
}

#[test]
fn test_check_unknown_provider() {
    // Ollama has 0/0 limits — unlimited
    let limiter = RateLimiter::new();
    assert!(limiter.check(Provider::Ollama, 999999).is_ok());
}

#[test]
fn test_record_only() {
    let limiter = RateLimiter::with_limits(strict_limits());
    limiter.record(Provider::OpenAI, 50);
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 1);
    assert_eq!(limiter.current_tpm(Provider::OpenAI), 50);
}

#[test]
fn test_rate_limit_error_display() {
    let e = RateLimitError::RpmExceeded {
        provider: Provider::OpenAI,
        current: 60,
        limit: 60,
    };
    let s = format!("{}", e);
    assert!(s.contains("RPM"));
    assert!(s.contains("OpenAI"));
    assert!(s.contains("60"));

    let e = RateLimitError::TpmExceeded {
        provider: Provider::Anthropic,
        current: 100000,
        limit: 80000,
    };
    let s = format!("{}", e);
    assert!(s.contains("TPM"));
    assert!(s.contains("Anthropic"));
}

#[test]
fn test_set_provider_limit() {
    let limiter = RateLimiter::new();
    limiter.set_provider_limit(Provider::OpenAI, ProviderRateLimit { rpm: 1, tpm: 10 });
    assert!(limiter.acquire(Provider::OpenAI, 5).is_ok());
    // RPM = 1, so second request fails
    assert!(limiter.acquire(Provider::OpenAI, 5).is_err());
}

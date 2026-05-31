//! Tests extracted from hudhudscript-tools-ai/src/fallback.rs

use hudhudscript_tools_ai::fallback::ProviderAttempt;
use hudhudscript_tools_ai::{
    FallbackChain, FallbackChainBuilder, FallbackEntry, FallbackError, Provider,
};

#[test]
fn test_first_provider_succeeds() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();

    let result = chain
        .execute(|_provider, _model| Ok::<&str, String>("hello"))
        .unwrap();

    assert_eq!(result.value, "hello");
    assert_eq!(result.provider, Provider::OpenAI);
    assert_eq!(result.attempts.len(), 1);
    assert!(result.attempts[0].success);
}

#[test]
fn test_fallback_to_second() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();

    let result = chain
        .execute(|provider, _model| {
            if provider == Provider::OpenAI {
                Err("rate limited".into())
            } else {
                Ok("fallback response")
            }
        })
        .unwrap();

    assert_eq!(result.value, "fallback response");
    assert_eq!(result.provider, Provider::Anthropic);
    assert_eq!(result.attempts.len(), 2);
    assert!(!result.attempts[0].success);
    assert!(result.attempts[1].success);
}

#[test]
fn test_all_providers_fail() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();

    let result = chain.execute(|_provider, _model| Err::<(), String>("fail".into()));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FallbackError::AllProvidersExhausted { .. }
    ));
}

#[test]
fn test_empty_chain_error() {
    let mut chain = FallbackChain::empty();
    let result = chain.execute(|_, _| Ok::<(), String>(()));
    assert!(matches!(result.unwrap_err(), FallbackError::EmptyChain));
}

#[test]
fn test_disabled_entry_skipped() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();

    chain.disable(0);

    let result = chain
        .execute(|_provider, _model| Ok::<&str, String>("ok"))
        .unwrap();

    assert_eq!(result.provider, Provider::Anthropic);
}

#[test]
fn test_auto_disable_after_failures() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .max_consecutive_failures(2)
        .build();

    // Fail twice for OpenAI, succeed with Anthropic
    for _ in 0..2 {
        let _ = chain.execute(|provider, _model| {
            if provider == Provider::OpenAI {
                Err("timeout".into())
            } else {
                Ok("ok")
            }
        });
    }

    // OpenAI should now be auto-disabled
    assert!(!chain.entries()[0].enabled);
}

#[test]
fn test_reset_re_enables_all() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();

    chain.disable(0);
    chain.disable(1);
    chain.reset();

    assert!(chain.entries()[0].enabled);
    assert!(chain.entries()[1].enabled);
}

#[test]
fn test_enable_after_disable() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();

    chain.disable(0);
    assert!(!chain.entries()[0].enabled);

    chain.enable(0);
    assert!(chain.entries()[0].enabled);
}

#[test]
fn test_enable_out_of_bounds() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .build();
    // Should not panic
    chain.enable(999);
    chain.disable(999);
}

#[test]
fn test_push_entry() {
    let mut chain = FallbackChain::empty();
    assert!(chain.is_empty());
    chain.push(FallbackEntry::new(Provider::OpenAI, "gpt-4o"));
    assert_eq!(chain.len(), 1);
    assert!(!chain.is_empty());
}

#[test]
fn test_provider_attempt_display_success() {
    let attempt = ProviderAttempt {
        provider: Provider::OpenAI,
        model: "gpt-4o".to_string(),
        success: true,
        error: None,
    };
    let s = format!("{}", attempt);
    assert!(s.contains("OK"));
    assert!(s.contains("OpenAI"));
    assert!(s.contains("gpt-4o"));
}

#[test]
fn test_provider_attempt_display_failure() {
    let attempt = ProviderAttempt {
        provider: Provider::Anthropic,
        model: "claude-3-sonnet".to_string(),
        success: false,
        error: Some("rate limited".to_string()),
    };
    let s = format!("{}", attempt);
    assert!(s.contains("FAIL"));
    assert!(s.contains("rate limited"));
}

#[test]
fn test_provider_attempt_display_failure_no_error() {
    let attempt = ProviderAttempt {
        provider: Provider::DeepSeek,
        model: "deepseek-chat".to_string(),
        success: false,
        error: None,
    };
    let s = format!("{}", attempt);
    assert!(s.contains("unknown"));
}

#[test]
fn test_fallback_chain_builder_default() {
    let builder = FallbackChainBuilder::default();
    let chain = builder.build();
    assert!(chain.is_empty());
}

#[test]
fn test_all_disabled_providers_exhausted() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .build();
    chain.disable(0);
    chain.disable(1);
    let result = chain.execute(|_, _| Ok::<(), String>(()));
    assert!(matches!(
        result,
        Err(FallbackError::AllProvidersExhausted { .. })
    ));
}

#[test]
fn test_builder_pattern() {
    let chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .add(Provider::DeepSeek, "deepseek-chat")
        .add(Provider::Ollama, "ollama-local")
        .max_consecutive_failures(5)
        .build();

    assert_eq!(chain.len(), 4);
    assert_eq!(chain.max_consecutive_failures, 5);
}

#[test]
fn test_fallback_error_display() {
    let e1 = FallbackError::AllProvidersExhausted {
        last_error: "timeout".to_string(),
    };
    let s = format!("{}", e1);
    assert!(s.contains("All providers exhausted"));
    assert!(s.contains("timeout"));

    let e2 = FallbackError::EmptyChain;
    assert!(format!("{}", e2).contains("No providers configured in fallback chain"));
}

#[test]
fn test_fallback_entry_new() {
    let entry = FallbackEntry::new(Provider::OpenAI, "gpt-4o");
    assert_eq!(entry.provider, Provider::OpenAI);
    assert_eq!(entry.model, "gpt-4o");
    assert!(entry.enabled);
}

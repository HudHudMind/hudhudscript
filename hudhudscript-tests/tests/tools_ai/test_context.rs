//! Tests extracted from hudhudscript-tools-ai/src/context.rs
//! (test_estimate_tokens_empty is already in tools_ai_test_lib.rs — skipped)

use hudhudscript_tools_ai::{
    estimate_tokens, ContextWindow, OutputLimiterConfig, ToolOutputLimiter,
};

#[test]
fn test_estimate_tokens_basic() {
    // "hello" = 5 chars → max(5/4, 1) = 1
    assert_eq!(estimate_tokens("hello"), 1);
    // 40 chars → 10 tokens
    let s = "a".repeat(40);
    assert_eq!(estimate_tokens(&s), 10);
}

#[test]
fn test_limiter_within_budget() {
    let limiter = ToolOutputLimiter::new(OutputLimiterConfig::with_max_tokens(1000));
    let output = "Hello, world!";
    let result = limiter.limit("test_tool", output);
    assert_eq!(result, output);
}

#[test]
fn test_limiter_truncates_large_output() {
    let limiter = ToolOutputLimiter::new(OutputLimiterConfig {
        max_tokens: 5,
        truncation_suffix: "[TRUNC]".to_string(),
        warn_on_truncation: false,
    });
    // 5 tokens * 4 chars = 20 chars for content
    let big_output = "a".repeat(1000);
    let result = limiter.limit("my_tool", &big_output);
    assert!(result.ends_with("[TRUNC]"));
    // Result should be much shorter than the original
    assert!(result.len() < big_output.len());
}

#[test]
fn test_limiter_with_info_flag() {
    let limiter = ToolOutputLimiter::new(OutputLimiterConfig {
        max_tokens: 2,
        truncation_suffix: "...".to_string(),
        warn_on_truncation: false,
    });
    let (_, was_truncated) = limiter.limit_with_info("t", &"a".repeat(100));
    assert!(was_truncated);

    let (_, was_truncated) = limiter.limit_with_info("t", "hi");
    assert!(!was_truncated);
}

#[test]
fn test_context_window_tracks_usage() {
    let mut window = ContextWindow::new(100);
    // Short output
    window.add_output("tool_a", "Hello world"); // ~2 tokens
    assert!(window.used_tokens() > 0);
    assert!(window.remaining_tokens() < 100);
}

#[test]
fn test_context_window_full_drops_output() {
    let mut window = ContextWindow::new(1);
    // First output fills the window
    let big = "a".repeat(100);
    window.add_output("tool_a", &big);

    // Subsequent output should be dropped
    let result = window.add_output("tool_b", "more data");
    assert!(result.is_none());
}

#[test]
fn test_context_window_truncates_to_fit() {
    let mut window = ContextWindow::new(10);
    let big = "a".repeat(1000); // far more than 10 tokens
    let entry = window.add_output("my_tool", &big).unwrap();
    assert!(entry.was_truncated);
    assert!(window.used_tokens() <= 10);
}

#[test]
fn test_limiter_default_limit() {
    let limiter = ToolOutputLimiter::default_limit();
    assert_eq!(limiter.max_tokens(), 2048);
}

#[test]
fn test_output_limiter_config_default() {
    let config = OutputLimiterConfig::default();
    assert_eq!(config.max_tokens, 2048);
    assert!(config.warn_on_truncation);
    assert!(config.truncation_suffix.contains("truncated"));
}

#[test]
fn test_context_window_is_full() {
    let mut window = ContextWindow::new(1);
    assert!(!window.is_full());
    window.add_output("tool_a", &"a".repeat(100));
    assert!(window.is_full());
}

#[test]
fn test_context_window_remaining_tokens() {
    let window = ContextWindow::new(100);
    assert_eq!(window.remaining_tokens(), 100);
    assert_eq!(window.used_tokens(), 0);
}

#[test]
fn test_estimate_tokens_short_string() {
    // Single character → max(1/4, 1) = 1
    assert_eq!(estimate_tokens("a"), 1);
    // 3 chars → max(3/4, 1) = 1
    assert_eq!(estimate_tokens("abc"), 1);
    // 4 chars → 4/4 = 1
    assert_eq!(estimate_tokens("abcd"), 1);
    // 8 chars → 8/4 = 2
    assert_eq!(estimate_tokens("abcdefgh"), 2);
}

#[test]
fn test_context_window_entry_content() {
    let mut window = ContextWindow::new(1000);
    let entry = window.add_output("tool_x", "Hello World").unwrap();
    assert_eq!(entry.tool_name, "tool_x");
    assert_eq!(entry.content, "Hello World");
    assert!(!entry.was_truncated);
}

#[test]
fn test_context_window_multiple_entries() {
    let mut window = ContextWindow::new(200);
    window.add_output("tool_a", "First output");
    window.add_output("tool_b", "Second output");
    assert_eq!(window.entries().len(), 2);
}

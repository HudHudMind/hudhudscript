//! Tests for tokenomics::compression — PromptCompressor, CompressionLevel, CompressionResult

use hudhudscript_tokenomics::compression::*;

// ---------------------------------------------------------------------------
// CompressionLevel::None
// ---------------------------------------------------------------------------

#[test]
fn test_none_passthrough() {
    let c = PromptCompressor::new(CompressionLevel::None);
    let result = c.compress("Hello world");
    assert_eq!(result.compressed, "Hello world");
    assert_eq!(result.reduction_pct, 0.0);
    assert_eq!(result.method, "None");
}

#[test]
fn test_none_empty_text() {
    let c = PromptCompressor::new(CompressionLevel::None);
    let result = c.compress("");
    assert_eq!(result.compressed, "");
    assert_eq!(result.reduction_pct, 0.0);
}

#[test]
fn test_none_preserves_whitespace() {
    let c = PromptCompressor::new(CompressionLevel::None);
    let result = c.compress("a   b   c");
    assert_eq!(result.compressed, "a   b   c");
}

// ---------------------------------------------------------------------------
// CompressionLevel::Light
// ---------------------------------------------------------------------------

#[test]
fn test_light_normalizes_spaces() {
    let c = PromptCompressor::new(CompressionLevel::Light);
    let result = c.compress("Hello   world   this   has   extra   spaces");
    assert!(!result.compressed.contains("  "));
    assert_eq!(result.method, "Light");
}

#[test]
fn test_light_reduces_tokens() {
    let c = PromptCompressor::new(CompressionLevel::Light);
    let result = c.compress("a     b     c     d     e");
    assert!(result.reduction_pct > 0.0);
}

#[test]
fn test_light_trims_ends() {
    let c = PromptCompressor::new(CompressionLevel::Light);
    let result = c.compress("  hello  ");
    assert_eq!(result.compressed, "hello");
}

#[test]
fn test_light_collapses_tabs_newlines() {
    let c = PromptCompressor::new(CompressionLevel::Light);
    let result = c.compress("hello\t\t\nworld");
    assert!(!result.compressed.contains('\t'));
    assert!(!result.compressed.contains("  "));
}

// ---------------------------------------------------------------------------
// CompressionLevel::Medium
// ---------------------------------------------------------------------------

#[test]
fn test_medium_removes_stop_words() {
    let c = PromptCompressor::new(CompressionLevel::Medium);
    let result = c.compress("The quick brown fox jumps over the lazy dog");
    assert!(!result.compressed.to_lowercase().contains(" the "));
    assert!(result.compressed.contains("quick"));
    assert!(result.compressed.contains("fox"));
    assert_eq!(result.method, "Medium");
}

#[test]
fn test_medium_preserves_code_blocks() {
    let c = PromptCompressor::new(CompressionLevel::Medium);
    let input =
        "Please fix this:\n```\nfn the_main() { let the = 1; }\n```\nThe function is broken";
    let result = c.compress(input);
    assert!(result.compressed.contains("fn the_main()"));
}

#[test]
fn test_medium_reduces_token_count() {
    let c = PromptCompressor::new(CompressionLevel::Medium);
    let input = "The quick brown fox is jumping over the very lazy dog";
    let result = c.compress(input);
    assert!(result.compressed_tokens <= result.original_tokens);
}

#[test]
fn test_medium_result_fields() {
    let c = PromptCompressor::new(CompressionLevel::Medium);
    let input = "The quick brown fox is jumping over the very lazy dog";
    let result = c.compress(input);
    assert!(result.original_tokens > 0);
    assert!(result.compressed_tokens > 0);
    assert_eq!(result.method, "Medium");
}

#[test]
fn test_medium_keeps_content_words() {
    let c = PromptCompressor::new(CompressionLevel::Medium);
    let result = c.compress("Implement a binary search algorithm");
    assert!(result.compressed.contains("Implement"));
    assert!(result.compressed.contains("binary"));
    assert!(result.compressed.contains("search"));
    assert!(result.compressed.contains("algorithm"));
}

// ---------------------------------------------------------------------------
// CompressionLevel::Aggressive
// ---------------------------------------------------------------------------

#[test]
fn test_aggressive_high_reduction() {
    let c = PromptCompressor::new(CompressionLevel::Aggressive);
    let long_text = "The weather is nice today. I went to the store and bought some groceries. \
        The programming language Rust is known for memory safety. Machine learning models require large datasets. \
        The cat sat on the mat. Cloud computing has transformed the industry. \
        Quantum computing will revolutionize cryptography. The sun rises in the east.";
    let result = c.compress(long_text);
    assert!(result.reduction_pct > 20.0);
    assert_eq!(result.method, "Aggressive");
}

#[test]
fn test_aggressive_keeps_informative_content() {
    let c = PromptCompressor::new(CompressionLevel::Aggressive);
    let text = "\
        Quantum entanglement enables instantaneous correlation between particles. \
        It was a nice day and things were good. \
        Cryptographic hash functions provide collision resistance guarantees. \
        Everything was fine and nothing happened. \
        Distributed consensus algorithms solve Byzantine fault tolerance. \
        The situation was okay and people were around. \
        Eigenvalue decomposition factorizes matrices into spectral components. \
        Things happened and stuff occurred naturally.";
    let result = c.compress(text);
    assert!(result.compressed.contains("entanglement") || result.compressed.contains("Quantum"));
    assert!(result.compressed.contains("Cryptographic") || result.compressed.contains("collision"));
}

#[test]
fn test_aggressive_short_text_fallback() {
    let c = PromptCompressor::new(CompressionLevel::Aggressive);
    let result = c.compress("Fix the bug");
    assert_eq!(result.method, "Aggressive");
    assert!(result.compressed.contains("Fix"));
    assert!(result.compressed.contains("bug"));
}

#[test]
fn test_aggressive_three_sentences_fallback() {
    let c = PromptCompressor::new(CompressionLevel::Aggressive);
    // Three or fewer sentences fall back to medium compression
    let result = c.compress("First sentence. Second sentence. Third sentence.");
    assert!(!result.compressed.is_empty());
}

#[test]
fn test_aggressive_preserves_order() {
    let c = PromptCompressor::new(CompressionLevel::Aggressive);
    let text = "\
        Alpha sentence with unique rare terminology. \
        Beta sentence with common everyday words. \
        Gamma sentence with specialized scientific jargon. \
        Delta sentence about nothing special really. \
        Epsilon sentence with cryptographic hash functions. \
        Zeta sentence was fairly generic overall. \
        Eta sentence about distributed consensus algorithms. \
        Theta sentence had nothing remarkable.";
    let result = c.compress(text);
    if result.compressed.contains("Alpha") && result.compressed.contains("Gamma") {
        let pos_alpha = result.compressed.find("Alpha").unwrap();
        let pos_gamma = result.compressed.find("Gamma").unwrap();
        assert!(pos_alpha < pos_gamma, "Original order should be preserved");
    }
}

// ---------------------------------------------------------------------------
// CompressionResult fields
// ---------------------------------------------------------------------------

#[test]
fn test_result_original_tokens_positive() {
    let c = PromptCompressor::new(CompressionLevel::Light);
    let result = c.compress("some text here");
    assert!(result.original_tokens > 0);
}

#[test]
fn test_result_compressed_tokens_positive() {
    let c = PromptCompressor::new(CompressionLevel::Light);
    let result = c.compress("some text here");
    assert!(result.compressed_tokens > 0);
}

#[test]
fn test_result_reduction_pct_non_negative() {
    let c = PromptCompressor::new(CompressionLevel::Medium);
    let result = c.compress("The quick brown fox");
    assert!(result.reduction_pct >= 0.0);
}

#[test]
fn test_result_debug() {
    let c = PromptCompressor::new(CompressionLevel::None);
    let result = c.compress("test");
    let debug = format!("{:?}", result);
    assert!(debug.contains("CompressionResult"));
}

// ---------------------------------------------------------------------------
// Cross-level comparisons
// ---------------------------------------------------------------------------

#[test]
fn test_empty_text_compression() {
    // estimate_tokens returns max(len/4, 1), so original_tokens is always >= 1
    // Verify correct behavior for empty string
    let c = PromptCompressor::new(CompressionLevel::None);
    let result = c.compress("");
    assert_eq!(result.compressed, "");
    assert_eq!(result.reduction_pct, 0.0);
}

#[test]
fn test_more_aggressive_means_more_reduction() {
    let text = "The weather is nice today. I went to the store and bought some groceries. \
        The programming language Rust is known for memory safety. Machine learning models require large datasets. \
        The cat sat on the mat. Cloud computing has transformed the industry. \
        Quantum computing will revolutionize cryptography. The sun rises in the east.";
    let none = PromptCompressor::new(CompressionLevel::None).compress(text);
    let light = PromptCompressor::new(CompressionLevel::Light).compress(text);
    let medium = PromptCompressor::new(CompressionLevel::Medium).compress(text);
    // None has 0 reduction, Light and Medium should have progressively more
    assert!(light.reduction_pct >= none.reduction_pct);
    assert!(medium.reduction_pct >= light.reduction_pct);
}

//! Public API tests for tokenomics::prompt_caching

use hudhudscript_tokenomics::prompt_caching::{
    PromptCacheOptimizer, PromptCacheProvider, PromptSegment, SegmentRole,
};

fn sample_segments() -> Vec<PromptSegment> {
    vec![
        PromptSegment {
            role: SegmentRole::UserMessage,
            content: "What is Rust?".to_string(),
            cacheable: false,
            token_estimate: 10,
        },
        PromptSegment {
            role: SegmentRole::System,
            content: "You are a helpful assistant.".to_string(),
            cacheable: true,
            token_estimate: 2000,
        },
        PromptSegment {
            role: SegmentRole::ToolDefinitions,
            content: r#"[{"name":"search"}]"#.to_string(),
            cacheable: true,
            token_estimate: 1500,
        },
        PromptSegment {
            role: SegmentRole::ConversationHistory,
            content: "Previous turns...".to_string(),
            cacheable: true,
            token_estimate: 500,
        },
    ]
}

// ── Construction and provider min_prefix ─────────────────────────────

#[test]
fn test_new_anthropic() {
    let opt = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    assert_eq!(opt.min_prefix_tokens, 1024);
    assert!(opt.auto_breakpoints);
    assert!(opt.reorder_for_cache);
}

#[test]
fn test_new_openai() {
    let opt = PromptCacheOptimizer::new(PromptCacheProvider::OpenAI);
    assert_eq!(opt.min_prefix_tokens, 1024);
}

#[test]
fn test_new_gemini() {
    let opt = PromptCacheOptimizer::new(PromptCacheProvider::Gemini);
    assert_eq!(opt.min_prefix_tokens, 4096);
}

#[test]
fn test_new_deepseek() {
    let opt = PromptCacheOptimizer::new(PromptCacheProvider::DeepSeek);
    assert_eq!(opt.min_prefix_tokens, 1024);
}

#[test]
fn test_new_none_provider() {
    let opt = PromptCacheOptimizer::new(PromptCacheProvider::None);
    assert_eq!(opt.min_prefix_tokens, u64::MAX);
}

// ── optimize_segments (reordering) ──────────────────────────────────

#[test]
fn test_reordering_sorts_by_role() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let mut segments = sample_segments();
    optimizer.optimize_segments(&mut segments);
    assert_eq!(segments[0].role, SegmentRole::System);
    assert_eq!(segments[1].role, SegmentRole::ToolDefinitions);
    assert_eq!(segments[2].role, SegmentRole::ConversationHistory);
    assert_eq!(segments[3].role, SegmentRole::UserMessage);
}

#[test]
fn test_no_reorder_when_disabled() {
    let mut optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    optimizer.reorder_for_cache = false;
    let mut segments = sample_segments();
    let original_first = segments[0].role;
    optimizer.optimize_segments(&mut segments);
    assert_eq!(segments[0].role, original_first);
}

#[test]
fn test_reorder_empty_segments() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let mut segments: Vec<PromptSegment> = vec![];
    optimizer.optimize_segments(&mut segments);
    assert!(segments.is_empty());
}

#[test]
fn test_reorder_single_segment() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let mut segments = vec![PromptSegment {
        role: SegmentRole::UserMessage,
        content: "hello".into(),
        cacheable: false,
        token_estimate: 5,
    }];
    optimizer.optimize_segments(&mut segments);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].role, SegmentRole::UserMessage);
}

// ── add_anthropic_breakpoints ───────────────────────────────────────

#[test]
fn test_anthropic_breakpoints_count() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let mut segments = sample_segments();
    optimizer.optimize_segments(&mut segments);
    let blocks = optimizer.add_anthropic_breakpoints(&segments);
    let breakpoint_count = blocks.iter().filter(|b| b.cache_control.is_some()).count();
    assert_eq!(breakpoint_count, 3);
}

#[test]
fn test_breakpoint_type_is_ephemeral() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let mut segments = sample_segments();
    optimizer.optimize_segments(&mut segments);
    let blocks = optimizer.add_anthropic_breakpoints(&segments);
    for block in &blocks {
        if let Some(ref ctrl) = block.cache_control {
            assert_eq!(ctrl.control_type, "ephemeral");
        }
    }
}

#[test]
fn test_breakpoints_below_min_prefix() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let segments = vec![
        PromptSegment {
            role: SegmentRole::System,
            content: "short".into(),
            cacheable: true,
            token_estimate: 100,
        },
        PromptSegment {
            role: SegmentRole::ToolDefinitions,
            content: "[]".into(),
            cacheable: true,
            token_estimate: 50,
        },
        PromptSegment {
            role: SegmentRole::UserMessage,
            content: "hi".into(),
            cacheable: false,
            token_estimate: 5,
        },
    ];
    let blocks = optimizer.add_anthropic_breakpoints(&segments);
    let count = blocks.iter().filter(|b| b.cache_control.is_some()).count();
    assert_eq!(count, 0);
}

#[test]
fn test_breakpoints_max_four() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    // Create 6 cacheable segments with different roles to force many boundaries
    let segments = vec![
        PromptSegment {
            role: SegmentRole::System,
            content: "a".into(),
            cacheable: true,
            token_estimate: 2000,
        },
        PromptSegment {
            role: SegmentRole::ToolDefinitions,
            content: "b".into(),
            cacheable: true,
            token_estimate: 2000,
        },
        PromptSegment {
            role: SegmentRole::ReferenceDocuments,
            content: "c".into(),
            cacheable: true,
            token_estimate: 2000,
        },
        PromptSegment {
            role: SegmentRole::ConversationHistory,
            content: "d".into(),
            cacheable: true,
            token_estimate: 2000,
        },
        PromptSegment {
            role: SegmentRole::UserMessage,
            content: "e".into(),
            cacheable: true,
            token_estimate: 2000,
        },
    ];
    let blocks = optimizer.add_anthropic_breakpoints(&segments);
    let count = blocks.iter().filter(|b| b.cache_control.is_some()).count();
    assert!(count <= 4);
}

#[test]
fn test_breakpoints_auto_disabled() {
    let mut optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    optimizer.auto_breakpoints = false;
    let mut segments = sample_segments();
    optimizer.optimize_segments(&mut segments);
    let blocks = optimizer.add_anthropic_breakpoints(&segments);
    let count = blocks.iter().filter(|b| b.cache_control.is_some()).count();
    assert_eq!(count, 0);
}

#[test]
fn test_breakpoints_content_preserved() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let mut segments = sample_segments();
    optimizer.optimize_segments(&mut segments);
    let blocks = optimizer.add_anthropic_breakpoints(&segments);
    assert_eq!(blocks.len(), segments.len());
    for (block, seg) in blocks.iter().zip(segments.iter()) {
        assert_eq!(block.content, seg.content);
    }
}

// ── is_cacheable ────────────────────────────────────────────────────

#[test]
fn test_is_cacheable_sufficient_tokens() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let segments = sample_segments();
    assert!(optimizer.is_cacheable(&segments));
}

#[test]
fn test_is_cacheable_insufficient_tokens() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let tiny = vec![PromptSegment {
        role: SegmentRole::System,
        content: "short".into(),
        cacheable: true,
        token_estimate: 10,
    }];
    assert!(!optimizer.is_cacheable(&tiny));
}

#[test]
fn test_none_provider_not_cacheable() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::None);
    let segments = sample_segments();
    assert!(!optimizer.is_cacheable(&segments));
}

#[test]
fn test_is_cacheable_empty() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    assert!(!optimizer.is_cacheable(&[]));
}

#[test]
fn test_is_cacheable_only_non_cacheable() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let segments = vec![PromptSegment {
        role: SegmentRole::UserMessage,
        content: "big content".into(),
        cacheable: false,
        token_estimate: 50000,
    }];
    assert!(!optimizer.is_cacheable(&segments));
}

// ── estimate_savings ────────────────────────────────────────────────

#[test]
fn test_savings_estimate_basic() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let segments = sample_segments();
    let estimate = optimizer.estimate_savings(&segments, 0.03, 0.25, 0.90);
    assert_eq!(estimate.cacheable_tokens, 4000);
    assert!(estimate.savings_pct > 90.0);
    assert!(estimate.savings_usd > 0.0);
    assert!(estimate.break_even_calls >= 2);
}

#[test]
fn test_savings_estimate_zero_tokens() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let estimate = optimizer.estimate_savings(&[], 0.03, 0.25, 0.90);
    assert_eq!(estimate.savings_pct, 0.0);
    assert_eq!(estimate.cacheable_tokens, 0);
}

#[test]
fn test_savings_estimate_zero_discount() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let segments = sample_segments();
    let estimate = optimizer.estimate_savings(&segments, 0.03, 0.25, 0.0);
    assert_eq!(estimate.break_even_calls, u64::MAX);
}

#[test]
fn test_savings_estimate_all_non_cacheable() {
    let optimizer = PromptCacheOptimizer::new(PromptCacheProvider::Anthropic);
    let segments = vec![PromptSegment {
        role: SegmentRole::UserMessage,
        content: "msg".into(),
        cacheable: false,
        token_estimate: 100,
    }];
    let estimate = optimizer.estimate_savings(&segments, 0.03, 0.25, 0.90);
    assert_eq!(estimate.cacheable_tokens, 0);
    assert_eq!(estimate.savings_usd, 0.0);
}

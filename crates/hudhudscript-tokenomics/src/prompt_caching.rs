//! Provider-native prompt caching optimization
//!
//! Optimizes prompt layout for provider-specific caching mechanisms (e.g.
//! Anthropic's prompt caching, OpenAI's automatic prefix caching) by
//! reordering segments and inserting cache breakpoints.

use serde::{Deserialize, Serialize};

/// Supported providers with native prompt caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptCacheProvider {
    Anthropic,
    OpenAI,
    Gemini,
    DeepSeek,
    None,
}

/// Role of a prompt segment, ordered by cache-friendliness (most stable first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SegmentRole {
    /// System prompt — changes least often.
    System = 0,
    /// Tool/function definitions.
    ToolDefinitions = 1,
    /// Reference documents injected via RAG.
    ReferenceDocuments = 2,
    /// Previous conversation turns.
    ConversationHistory = 3,
    /// Current user message — changes every request.
    UserMessage = 4,
}

/// A segment of the prompt that may or may not be cacheable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSegment {
    pub role: SegmentRole,
    pub content: String,
    pub cacheable: bool,
    pub token_estimate: u64,
}

/// Anthropic-specific cache control annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub control_type: String,
}

/// A content block annotated with Anthropic cache control metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicCacheBlock {
    pub content: String,
    pub cache_control: Option<CacheControl>,
}

/// Estimated savings from prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSavingsEstimate {
    /// Number of tokens eligible for caching.
    pub cacheable_tokens: u64,
    /// Percentage of total tokens that are cacheable.
    pub savings_pct: f64,
    /// Estimated USD saved per cached call.
    pub savings_usd: f64,
    /// Number of calls needed before caching becomes cost-positive.
    pub break_even_calls: u64,
}

/// Prompt caching optimizer that reorders segments and adds cache breakpoints.
#[derive(Debug, Clone)]
pub struct PromptCacheOptimizer {
    pub provider: PromptCacheProvider,
    /// Minimum token prefix length for caching to activate.
    pub min_prefix_tokens: u64,
    /// Automatically insert cache breakpoints.
    pub auto_breakpoints: bool,
    /// Reorder segments for optimal cache hit rate.
    pub reorder_for_cache: bool,
}

impl PromptCacheOptimizer {
    pub fn new(provider: PromptCacheProvider) -> Self {
        let min_prefix = match provider {
            PromptCacheProvider::Anthropic => 1024,
            PromptCacheProvider::OpenAI => 1024,
            PromptCacheProvider::Gemini => 4096,
            PromptCacheProvider::DeepSeek => 1024,
            PromptCacheProvider::None => u64::MAX,
        };
        Self {
            provider,
            min_prefix_tokens: min_prefix,
            auto_breakpoints: true,
            reorder_for_cache: true,
        }
    }

    /// Sort segments by role so the most stable content appears first.
    pub fn optimize_segments(&self, segments: &mut [PromptSegment]) {
        if self.reorder_for_cache {
            segments.sort_by(|a, b| a.role.cmp(&b.role));
        }
    }

    /// Insert Anthropic cache breakpoints (max 4) at the boundaries of
    /// cacheable prefix segments.
    pub fn add_anthropic_breakpoints(
        &self,
        segments: &[PromptSegment],
    ) -> Vec<AnthropicCacheBlock> {
        let max_breakpoints: usize = 4;
        let mut blocks: Vec<AnthropicCacheBlock> = Vec::new();
        let mut breakpoints_placed: usize = 0;
        let mut cumulative_tokens: u64 = 0;

        for (i, seg) in segments.iter().enumerate() {
            cumulative_tokens += seg.token_estimate;

            let is_boundary =
                i + 1 < segments.len() && segments[i + 1].role != seg.role && seg.cacheable;

            let at_end = i + 1 == segments.len() && seg.cacheable;

            let should_break = (is_boundary || at_end)
                && breakpoints_placed < max_breakpoints
                && cumulative_tokens >= self.min_prefix_tokens
                && self.auto_breakpoints;

            let cache_control = if should_break {
                breakpoints_placed += 1;
                Some(CacheControl {
                    control_type: "ephemeral".to_string(),
                })
            } else {
                None
            };

            blocks.push(AnthropicCacheBlock {
                content: seg.content.clone(),
                cache_control,
            });
        }

        blocks
    }

    /// Estimate the cost savings from caching the given segments.
    pub fn estimate_savings(
        &self,
        segments: &[PromptSegment],
        cost_per_1k_tokens: f64,
        cache_write_premium: f64,
        cache_read_discount: f64,
    ) -> CacheSavingsEstimate {
        let total_tokens: u64 = segments.iter().map(|s| s.token_estimate).sum();
        let cacheable_tokens: u64 = segments
            .iter()
            .filter(|s| s.cacheable)
            .map(|s| s.token_estimate)
            .sum();

        let savings_pct = if total_tokens > 0 {
            (cacheable_tokens as f64 / total_tokens as f64) * 100.0
        } else {
            0.0
        };

        // Cost of one uncached call for the cacheable portion.
        let uncached_cost = (cacheable_tokens as f64 / 1000.0) * cost_per_1k_tokens;
        // Savings per cached read.
        let savings_per_read = uncached_cost * cache_read_discount;
        // Extra cost for the initial cache write.
        let write_premium_cost = uncached_cost * cache_write_premium;

        let break_even_calls = if savings_per_read > 0.0 {
            (write_premium_cost / savings_per_read).ceil() as u64 + 1
        } else {
            u64::MAX
        };

        CacheSavingsEstimate {
            cacheable_tokens,
            savings_pct,
            savings_usd: savings_per_read,
            break_even_calls,
        }
    }

    /// Check whether the total cacheable tokens meet the provider's minimum.
    pub fn is_cacheable(&self, segments: &[PromptSegment]) -> bool {
        let cacheable_tokens: u64 = segments
            .iter()
            .filter(|s| s.cacheable)
            .map(|s| s.token_estimate)
            .sum();
        cacheable_tokens >= self.min_prefix_tokens
    }
}

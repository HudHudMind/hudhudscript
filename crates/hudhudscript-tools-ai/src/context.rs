//! Context Management: Token Window Limits for Large Tool Outputs (Issue #122)
//!
//! Provides a lightweight token counter and a `ToolOutputLimiter` that truncates
//! tool output strings that exceed a configurable token budget.

use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Token estimation
// ---------------------------------------------------------------------------

/// Estimate the number of tokens in a string using a simple heuristic:
/// approximately 4 characters per token (suitable for English / code).
///
/// For production use, replace with a proper tokeniser (e.g. `tiktoken-rs`).
pub fn estimate_tokens(text: &str) -> usize {
    // Heuristic: 1 token ≈ 4 chars; always return at least 1 for non-empty text.
    let chars = text.chars().count();
    if chars == 0 {
        0
    } else {
        (chars / 4).max(1)
    }
}

// ---------------------------------------------------------------------------
// ToolOutputLimiter
// ---------------------------------------------------------------------------

/// Configuration for the tool output limiter.
#[derive(Debug, Clone)]
pub struct OutputLimiterConfig {
    /// Maximum number of tokens allowed in a single tool output.
    pub max_tokens: usize,
    /// Suffix appended to truncated outputs.
    pub truncation_suffix: String,
    /// When `true`, emit a `warn!` log whenever an output is truncated.
    pub warn_on_truncation: bool,
}

impl Default for OutputLimiterConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            truncation_suffix: "... [output truncated — token limit exceeded]".to_string(),
            warn_on_truncation: true,
        }
    }
}

impl OutputLimiterConfig {
    /// Create a limiter config with a custom max token count.
    pub fn with_max_tokens(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }
}

/// Applies token-budget limits to tool output strings.
#[derive(Debug, Clone)]
pub struct ToolOutputLimiter {
    config: OutputLimiterConfig,
}

impl ToolOutputLimiter {
    /// Create a new limiter with the provided configuration.
    pub fn new(config: OutputLimiterConfig) -> Self {
        Self { config }
    }

    /// Create a limiter with default settings (2 048-token limit).
    pub fn default_limit() -> Self {
        Self::new(OutputLimiterConfig::default())
    }

    /// Apply the token limit to `output`.
    ///
    /// * If `output` is within the budget, it is returned unchanged.
    /// * If it exceeds the budget, it is truncated so that the total
    ///   (including the suffix) fits within `max_tokens`.
    pub fn limit(&self, tool_name: &str, output: &str) -> String {
        let total_tokens = estimate_tokens(output);
        if total_tokens <= self.config.max_tokens {
            debug!(
                tool = tool_name,
                tokens = total_tokens,
                "Tool output within token budget"
            );
            return output.to_string();
        }

        if self.config.warn_on_truncation {
            warn!(
                tool = tool_name,
                tokens = total_tokens,
                limit = self.config.max_tokens,
                "Tool output exceeds token limit — truncating"
            );
        }

        // How many characters we can keep.
        let suffix_tokens = estimate_tokens(&self.config.truncation_suffix);
        let budget_chars = (self.config.max_tokens.saturating_sub(suffix_tokens)) * 4;

        let truncated: String = output.chars().take(budget_chars).collect();
        format!("{}{}", truncated, self.config.truncation_suffix)
    }

    /// Apply the limit and return both the (possibly truncated) output and
    /// whether truncation occurred.
    pub fn limit_with_info(&self, tool_name: &str, output: &str) -> (String, bool) {
        let total_tokens = estimate_tokens(output);
        let was_truncated = total_tokens > self.config.max_tokens;
        (self.limit(tool_name, output), was_truncated)
    }

    /// Maximum configured token count.
    pub fn max_tokens(&self) -> usize {
        self.config.max_tokens
    }
}

// ---------------------------------------------------------------------------
// ContextWindow
// ---------------------------------------------------------------------------

/// Tracks cumulative token usage across multiple tool outputs in a single
/// agent turn, enforcing a per-turn context window.
#[derive(Debug, Clone)]
pub struct ContextWindow {
    /// Maximum total tokens allowed across all tool outputs in this window.
    pub max_total_tokens: usize,
    /// Tokens consumed so far.
    used_tokens: usize,
    /// Individual tool output entries.
    entries: Vec<ContextEntry>,
}

/// A single entry in the context window.
#[derive(Debug, Clone)]
pub struct ContextEntry {
    pub tool_name: String,
    pub content: String,
    pub tokens: usize,
    pub was_truncated: bool,
}

impl ContextWindow {
    /// Create a new context window with the given total token limit.
    pub fn new(max_total_tokens: usize) -> Self {
        Self {
            max_total_tokens,
            used_tokens: 0,
            entries: Vec::new(),
        }
    }

    /// Returns the number of tokens still available in this window.
    pub fn remaining_tokens(&self) -> usize {
        self.max_total_tokens.saturating_sub(self.used_tokens)
    }

    /// Returns `true` if the window is full (no more tokens available).
    pub fn is_full(&self) -> bool {
        self.used_tokens >= self.max_total_tokens
    }

    /// Attempt to add a tool output to the window.
    ///
    /// The output is automatically truncated to fit the remaining budget.
    /// Returns `None` if the window is already full (zero tokens remain).
    pub fn add_output(&mut self, tool_name: &str, output: &str) -> Option<&ContextEntry> {
        if self.is_full() {
            warn!(
                tool = tool_name,
                "Context window full — dropping tool output"
            );
            return None;
        }

        let raw_tokens = estimate_tokens(output);
        let allowed_tokens = self.remaining_tokens();

        let (content, was_truncated) = if raw_tokens > allowed_tokens {
            let suffix = " ... [context window limit reached]";
            let suffix_tokens = estimate_tokens(suffix);
            let text_budget_chars = allowed_tokens.saturating_sub(suffix_tokens) * 4;
            let truncated: String = output.chars().take(text_budget_chars).collect();
            (format!("{}{}", truncated, suffix), true)
        } else {
            (output.to_string(), false)
        };

        let tokens = estimate_tokens(&content);
        self.used_tokens += tokens;

        self.entries.push(ContextEntry {
            tool_name: tool_name.to_string(),
            content,
            tokens,
            was_truncated,
        });

        debug!(
            tool = tool_name,
            tokens,
            total_used = self.used_tokens,
            remaining = self.remaining_tokens(),
            was_truncated,
            "Added tool output to context window"
        );

        self.entries.last()
    }

    /// Return all entries collected so far.
    pub fn entries(&self) -> &[ContextEntry] {
        &self.entries
    }

    /// Total tokens consumed.
    pub fn used_tokens(&self) -> usize {
        self.used_tokens
    }
}

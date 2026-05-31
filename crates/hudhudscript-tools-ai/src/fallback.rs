//! Provider Fallback Chain (Issue #607)
//!
//! When an AI provider fails (network error, rate limit, budget exceeded),
//! the fallback chain automatically retries the request against the next
//! provider in a user-defined priority list.

use crate::cost::Provider;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum FallbackError {
    AllProvidersExhausted { last_error: String },
    EmptyChain,
}

impl std::fmt::Display for FallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            FallbackError::AllProvidersExhausted { last_error } => {
                write!(f, "All providers exhausted. Last error: {}", last_error)
            }
            FallbackError::EmptyChain => write!(f, "No providers configured in fallback chain"),
        }
    }
}

impl std::error::Error for FallbackError {}

/// Describes why a particular provider attempt failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAttempt {
    /// Which provider was tried.
    pub provider: Provider,
    /// Model that was tried.
    pub model: String,
    /// Whether the attempt succeeded.
    pub success: bool,
    /// Error message if the attempt failed.
    pub error: Option<String>,
}

impl fmt::Display for ProviderAttempt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            write!(f, "{}({}) -> OK", self.provider, self.model)
        } else {
            write!(
                f,
                "{}({}) -> FAIL: {}",
                self.provider,
                self.model,
                self.error.as_deref().unwrap_or("unknown")
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Fallback entry
// ---------------------------------------------------------------------------

/// A single entry in the fallback chain: a provider + model pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackEntry {
    pub provider: Provider,
    pub model: String,
    /// If `true`, this entry is temporarily disabled (e.g. after repeated
    /// failures).  Disabled entries are skipped during fallback traversal.
    pub enabled: bool,
}

impl FallbackEntry {
    pub fn new(provider: Provider, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// FallbackResult
// ---------------------------------------------------------------------------

/// The outcome of a fallback-chain execution.
#[derive(Debug, Clone)]
pub struct FallbackResult<T> {
    /// The successful result value (if any provider succeeded).
    pub value: T,
    /// The provider + model that succeeded.
    pub provider: Provider,
    pub model: String,
    /// Full log of attempts, including failures that preceded the success.
    pub attempts: Vec<ProviderAttempt>,
}

// ---------------------------------------------------------------------------
// FallbackChain
// ---------------------------------------------------------------------------

/// An ordered chain of provider/model pairs.  On each request the chain is
/// traversed top-to-bottom; the first provider that succeeds wins.
#[derive(Debug, Clone)]
pub struct FallbackChain {
    entries: Vec<FallbackEntry>,
    /// Maximum number of consecutive failures before a provider is
    /// auto-disabled.  `0` means never auto-disable.
    pub max_consecutive_failures: usize,
    /// Tracks consecutive failure counts per index.
    failure_counts: Vec<usize>,
}

impl FallbackChain {
    /// Create a new chain from the given entries.
    pub fn new(entries: Vec<FallbackEntry>) -> Self {
        let len = entries.len();
        Self {
            entries,
            max_consecutive_failures: 3,
            failure_counts: vec![0; len],
        }
    }

    /// Create an empty chain.
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            max_consecutive_failures: 3,
            failure_counts: Vec::new(),
        }
    }

    /// Append a provider to the end of the chain.
    pub fn push(&mut self, entry: FallbackEntry) {
        self.entries.push(entry);
        self.failure_counts.push(0);
    }

    /// Number of entries (including disabled ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the chain has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return a reference to all entries.
    pub fn entries(&self) -> &[FallbackEntry] {
        &self.entries
    }

    /// Re-enable all providers and reset failure counters.
    pub fn reset(&mut self) {
        for entry in &mut self.entries {
            entry.enabled = true;
        }
        self.failure_counts = vec![0; self.entries.len()];
    }

    /// Disable a specific entry by index.
    pub fn disable(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.enabled = false;
        }
    }

    /// Enable a specific entry by index.
    pub fn enable(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.enabled = true;
        }
        if index < self.failure_counts.len() {
            self.failure_counts[index] = 0;
        }
    }

    /// Execute a closure against the fallback chain.
    ///
    /// The closure `f` receives `(provider, model)` and returns
    /// `Result<T, String>`.  The chain tries each enabled entry in order
    /// until one succeeds.
    pub fn execute<T, F>(&mut self, mut f: F) -> Result<FallbackResult<T>, FallbackError>
    where
        F: FnMut(Provider, &str) -> Result<T, String>,
    {
        if self.entries.is_empty() {
            return Err(FallbackError::EmptyChain);
        }

        let mut attempts = Vec::new();
        let mut last_error = String::new();

        for i in 0..self.entries.len() {
            let entry = &self.entries[i];
            if !entry.enabled {
                continue;
            }

            let provider = entry.provider;
            let model = entry.model.clone();

            match f(provider, &model) {
                Ok(value) => {
                    info!(
                        provider = %provider,
                        model = %model,
                        attempt = i + 1,
                        "Fallback chain: provider succeeded"
                    );
                    // Reset failure count on success
                    self.failure_counts[i] = 0;

                    attempts.push(ProviderAttempt {
                        provider,
                        model: model.clone(),
                        success: true,
                        error: None,
                    });

                    return Ok(FallbackResult {
                        value,
                        provider,
                        model,
                        attempts,
                    });
                }
                Err(err) => {
                    warn!(
                        provider = %provider,
                        model = %model,
                        error = %err,
                        "Fallback chain: provider failed, trying next"
                    );

                    self.failure_counts[i] += 1;
                    if self.max_consecutive_failures > 0
                        && self.failure_counts[i] >= self.max_consecutive_failures
                    {
                        warn!(
                            provider = %provider,
                            model = %model,
                            failures = self.failure_counts[i],
                            "Auto-disabling provider after repeated failures"
                        );
                        self.entries[i].enabled = false;
                    }

                    last_error = err.clone();
                    attempts.push(ProviderAttempt {
                        provider,
                        model,
                        success: false,
                        error: Some(err),
                    });
                }
            }
        }

        Err(FallbackError::AllProvidersExhausted { last_error })
    }
}

// ---------------------------------------------------------------------------
// Convenience builder
// ---------------------------------------------------------------------------

/// Helper to build a typical fallback chain.
///
/// ```
/// use hudhudscript_tools_ai::fallback::{FallbackChainBuilder};
/// use hudhudscript_tools_ai::cost::Provider;
///
/// let chain = FallbackChainBuilder::new()
///     .add(Provider::OpenAI, "gpt-4o")
///     .add(Provider::Anthropic, "claude-3-sonnet")
///     .add(Provider::DeepSeek, "deepseek-chat")
///     .add(Provider::Ollama, "ollama-local")
///     .build();
/// ```
pub struct FallbackChainBuilder {
    entries: Vec<FallbackEntry>,
    max_failures: usize,
}

impl FallbackChainBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_failures: 3,
        }
    }

    /// Append a provider/model to the chain.
    pub fn add(mut self, provider: Provider, model: impl Into<String>) -> Self {
        self.entries.push(FallbackEntry::new(provider, model));
        self
    }

    /// Set the max consecutive failures before auto-disable.
    pub fn max_consecutive_failures(mut self, n: usize) -> Self {
        self.max_failures = n;
        self
    }

    /// Build the [`FallbackChain`].
    pub fn build(self) -> FallbackChain {
        let mut chain = FallbackChain::new(self.entries);
        chain.max_consecutive_failures = self.max_failures;
        chain
    }
}

impl Default for FallbackChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl FallbackError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            FallbackError::AllProvidersExhausted { .. } => {
                hudhudscript_errors::ErrorCode::FallbackAllProvidersExhausted
            }
            FallbackError::EmptyChain => hudhudscript_errors::ErrorCode::FallbackEmptyChain,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<FallbackError> for hudhudscript_errors::Error {
    fn from(e: FallbackError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

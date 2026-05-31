//! Streaming token counter with mid-stream budget enforcement

/// Action to take after processing a chunk
#[derive(Debug, Clone, PartialEq)]
pub enum StreamAction {
    Continue,
    Warning {
        estimated_tokens: usize,
        budget: usize,
    },
    Cancel {
        estimated_tokens: usize,
        budget: usize,
    },
}

/// Result of reconciling estimated vs actual tokens
#[derive(Debug, Clone)]
pub struct TokenReconciliation {
    pub estimated: usize,
    pub actual: usize,
    pub drift_pct: f64,
}

/// Real-time token counter for streaming responses
pub struct StreamingTokenCounter {
    token_count: usize,
    budget_limit: usize,
    warning_threshold: f64,
    cancelled: bool,
    chars_per_token: f64,
}

impl StreamingTokenCounter {
    pub fn new(budget_limit: usize) -> Self {
        Self {
            token_count: 0,
            budget_limit,
            warning_threshold: 0.80,
            cancelled: false,
            chars_per_token: 4.0,
        }
    }

    pub fn with_warning_threshold(mut self, threshold: f64) -> Self {
        self.warning_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Process a streaming chunk, estimating token count
    pub fn process_chunk(&mut self, chunk: &str) -> StreamAction {
        if self.cancelled {
            return StreamAction::Cancel {
                estimated_tokens: self.token_count,
                budget: self.budget_limit,
            };
        }

        let chunk_tokens = (chunk.len() as f64 / self.chars_per_token).ceil() as usize;
        self.token_count += chunk_tokens;

        if self.token_count >= self.budget_limit {
            self.cancelled = true;
            return StreamAction::Cancel {
                estimated_tokens: self.token_count,
                budget: self.budget_limit,
            };
        }

        let usage_pct = self.usage_percentage();
        if usage_pct >= self.warning_threshold {
            return StreamAction::Warning {
                estimated_tokens: self.token_count,
                budget: self.budget_limit,
            };
        }

        StreamAction::Continue
    }

    /// Reconcile estimated count with server-reported actual
    pub fn reconcile(&self, server_tokens: usize) -> TokenReconciliation {
        let drift = if server_tokens > 0 {
            ((self.token_count as f64 - server_tokens as f64) / server_tokens as f64).abs()
        } else {
            0.0
        };
        TokenReconciliation {
            estimated: self.token_count,
            actual: server_tokens,
            drift_pct: drift,
        }
    }

    pub fn current_count(&self) -> usize {
        self.token_count
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn usage_percentage(&self) -> f64 {
        if self.budget_limit == 0 {
            return 0.0;
        }
        self.token_count as f64 / self.budget_limit as f64
    }
}

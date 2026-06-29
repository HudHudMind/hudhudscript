//! TokenomicsProvider — wraps a Provider with budget enforcement and cost tracking

use crate::provider::*;
use std::sync::{Arc, Mutex};

/// Wraps any Provider with tokenomics budget enforcement and usage tracking.
pub struct TokenomicsProvider {
    inner: Arc<dyn Provider>,
    tracker: Arc<Mutex<TokenTracker>>,
    max_tokens_per_call: usize,
    max_tokens_per_day: usize,
    max_tokens_per_month: usize,
}

impl TokenomicsProvider {
    /// Wrap an existing provider with tokenomics enforcement.
    pub fn wrap(
        inner: Arc<dyn Provider>,
        max_tokens_per_call: usize,
        max_tokens_per_day: usize,
        max_tokens_per_month: usize,
    ) -> Self {
        Self {
            inner,
            tracker: Arc::new(Mutex::new(TokenTracker::new())),
            max_tokens_per_call,
            max_tokens_per_day,
            max_tokens_per_month,
        }
    }

    fn check_limits(&self, estimated_tokens: usize) -> Result<(), ProviderError> {
        if estimated_tokens > self.max_tokens_per_call {
            return Err(ProviderError::BudgetExceeded {
                limit: self.max_tokens_per_call,
                requested: estimated_tokens,
            });
        }

        let tracker = self.tracker.lock().unwrap();
        let daily = tracker.daily_usage();
        let monthly = tracker.monthly_usage();

        if daily + estimated_tokens > self.max_tokens_per_day {
            return Err(ProviderError::DailyBudgetExceeded {
                limit: self.max_tokens_per_day,
                current: daily,
            });
        }

        if monthly + estimated_tokens > self.max_tokens_per_month {
            return Err(ProviderError::MonthlyBudgetExceeded {
                limit: self.max_tokens_per_month,
                current: monthly,
            });
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Provider for TokenomicsProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        let estimated = estimate_tokens(&request.prompt);
        self.check_limits(estimated)?;

        let response = self.inner.call(request).await?;

        {
            let mut tracker = self.tracker.lock().unwrap();
            tracker.record(
                response.tokens_used.prompt_tokens,
                response.tokens_used.completion_tokens,
            );
        }

        Ok(response)
    }

    async fn stream_call(
        &self,
        request: LLMRequest,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<LLMResponse, ProviderError> {
        let estimated = estimate_tokens(&request.prompt);
        self.check_limits(estimated)?;

        let response = self.inner.stream_call(request, on_chunk).await?;

        {
            let mut tracker = self.tracker.lock().unwrap();
            tracker.record(
                response.tokens_used.prompt_tokens,
                response.tokens_used.completion_tokens,
            );
        }

        Ok(response)
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        self.inner.list_models().await
    }

    fn info(&self) -> ProviderInfo {
        self.inner.info()
    }

    fn check_budget(&self, tokens: usize) -> Result<(), ProviderError> {
        self.check_limits(tokens)
    }

    async fn get_usage_stats(&self) -> TokenUsageStats {
        let tracker = self.tracker.lock().unwrap();
        tracker.get_stats()
    }
}

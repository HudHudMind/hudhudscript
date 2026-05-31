//! Provider registry and token tracking

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::provider::{traits::Provider, types::TokenUsageStats};

/// Provider registry for managing multiple providers
#[derive(Clone)]
pub struct ProviderRegistry {
    /// Registered providers
    providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a provider
    pub async fn register(&self, name: String, provider: Arc<dyn Provider>) {
        self.providers.write().await.insert(name, provider);
    }

    /// Get a provider by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.read().await.get(name).cloned()
    }

    /// List all registered provider names
    pub async fn list(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }

    /// Remove a provider
    pub async fn unregister(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.write().await.remove(name)
    }

    /// Check if a provider exists
    pub async fn exists(&self, name: &str) -> bool {
        self.providers.read().await.contains_key(name)
    }

    /// Register a provider with automatic tokenomics wrapping.
    pub async fn register_with_tokenomics(
        &self,
        name: String,
        provider: Arc<dyn Provider>,
        max_tokens_per_call: Option<usize>,
        max_tokens_per_day: Option<usize>,
        max_tokens_per_month: Option<usize>,
    ) {
        if let (Some(per_call), Some(per_day), Some(per_month)) = (
            max_tokens_per_call,
            max_tokens_per_day,
            max_tokens_per_month,
        ) {
            let wrapped = Arc::new(crate::tokenomics_provider::TokenomicsProvider::wrap(
                provider, per_call, per_day, per_month,
            ));
            self.providers.write().await.insert(name, wrapped);
        } else {
            self.providers.write().await.insert(name, provider);
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate token count from text (rough approximation)
///
/// This is a simple heuristic: 1 token ≈ 4 characters
/// For production, use tiktoken or similar
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

/// Token tracker for budget enforcement
pub struct TokenTracker {
    /// Daily token usage
    daily_usage: usize,

    /// Monthly token usage
    monthly_usage: usize,

    /// Last daily reset time
    pub last_daily_reset: std::time::SystemTime,

    /// Last monthly reset time
    pub last_monthly_reset: std::time::SystemTime,
}

impl TokenTracker {
    /// Create a new token tracker
    pub fn new() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            daily_usage: 0,
            monthly_usage: 0,
            last_daily_reset: now,
            last_monthly_reset: now,
        }
    }

    /// Record token usage
    pub fn record(&mut self, tokens: usize) {
        // Reset if needed
        self.check_and_reset();

        self.daily_usage += tokens;
        self.monthly_usage += tokens;
    }

    /// Get daily usage
    pub fn daily_usage(&self) -> usize {
        self.daily_usage
    }

    /// Get monthly usage
    pub fn monthly_usage(&self) -> usize {
        self.monthly_usage
    }

    /// Get last reset time
    pub fn last_reset(&self) -> std::time::SystemTime {
        self.last_daily_reset
    }

    /// Check if daily reset is needed
    pub fn should_reset_daily(&self) -> bool {
        let elapsed = std::time::SystemTime::now()
            .duration_since(self.last_daily_reset)
            .unwrap_or_default();
        elapsed.as_secs() > 86400 // 24 hours
    }

    /// Check if monthly reset is needed
    pub fn should_reset_monthly(&self) -> bool {
        let elapsed = std::time::SystemTime::now()
            .duration_since(self.last_monthly_reset)
            .unwrap_or_default();
        elapsed.as_secs() > 2592000 // 30 days
    }

    /// Check and perform resets if needed
    fn check_and_reset(&mut self) {
        if self.should_reset_daily() {
            self.daily_usage = 0;
            self.last_daily_reset = std::time::SystemTime::now();
        }

        if self.should_reset_monthly() {
            self.monthly_usage = 0;
            self.last_monthly_reset = std::time::SystemTime::now();
        }
    }

    /// Get usage statistics
    pub fn get_stats(&self) -> TokenUsageStats {
        TokenUsageStats {
            daily_usage: self.daily_usage,
            monthly_usage: self.monthly_usage,
            estimated_cost: self.estimate_cost(),
            last_reset: self.last_daily_reset,
        }
    }

    /// Estimate cost based on usage (rough estimate)
    fn estimate_cost(&self) -> f64 {
        // Rough estimate: $0.03 per 1K tokens (GPT-4 pricing)
        (self.monthly_usage as f64 / 1000.0) * 0.03
    }
}

impl Default for TokenTracker {
    fn default() -> Self {
        Self::new()
    }
}

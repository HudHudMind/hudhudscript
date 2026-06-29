//! Unified tokenomics facade — connects all subsystems into a single entry point

use crate::attribution::{CostAttributor, CostEvent};
use crate::config::TokenomicsConfig;
use crate::enforcement::{AlertAction, BudgetEnforcer, EnforcementDecision};
use crate::pricing::{CostBreakdown, PricingRegistry};
use crate::storage::{FileStorageBackend, StorageBackend};
use crate::streaming::StreamingTokenCounter;
use crate::types::TokenUsageRecord;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Unified tokenomics manager — single entry point for all subsystems
pub struct TokenomicsManager {
    config: TokenomicsConfig,
    pricing: PricingRegistry,
    storage: Option<FileStorageBackend>,
    user_id: String,
    enforcer: Mutex<BudgetEnforcer>,
    attributor: Mutex<CostAttributor>,
    enabled: bool,
}

impl TokenomicsManager {
    /// Create from config
    pub fn from_config(config: TokenomicsConfig) -> Self {
        let enforcer = BudgetEnforcer::new(
            config.budget.max_tokens_per_call,
            config.budget.max_tokens_per_day,
            config.budget.max_tokens_per_month,
            config.budget.alert_threshold,
            AlertAction::from_str(&config.alerts.on_warning),
            AlertAction::from_str(&config.alerts.on_critical),
            AlertAction::from_str(&config.alerts.on_depleted),
        );

        let storage = if config.enabled {
            FileStorageBackend::default_path().ok()
        } else {
            None
        };

        Self {
            enabled: config.enabled,
            config,
            pricing: PricingRegistry::new(),
            storage,
            user_id: "default".to_string(),
            enforcer: Mutex::new(enforcer),
            attributor: Mutex::new(CostAttributor::new()),
        }
    }

    /// Check if tokenomics is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Pre-call check: should this request be allowed?
    pub fn pre_call_check(&self, estimated_tokens: usize) -> EnforcementDecision {
        if !self.enabled {
            return EnforcementDecision::Allowed;
        }
        let mut enforcer = self.enforcer.lock().unwrap();
        enforcer.check_request(estimated_tokens)
    }

    /// Post-call: record usage, calculate cost, emit attribution event
    #[allow(clippy::too_many_arguments)]
    pub fn post_call_record(
        &self,
        model: &str,
        provider: &str,
        input_tokens: usize,
        output_tokens: usize,
        cached_tokens: usize,
        thinking_tokens: usize,
        is_batch: bool,
        user_id: Option<&str>,
        session_id: Option<&str>,
        feature_tag: Option<&str>,
    ) -> Option<CostBreakdown> {
        if !self.enabled {
            return None;
        }

        let total_tokens = input_tokens + output_tokens + thinking_tokens;

        // Record in enforcer
        {
            let mut enforcer = self.enforcer.lock().unwrap();
            enforcer.record_usage(total_tokens);
        }

        // Calculate cost
        let cost = self.pricing.calculate_cost(
            model,
            input_tokens,
            output_tokens,
            cached_tokens,
            thinking_tokens,
            is_batch,
        );

        // Record attribution event
        if self.config.attribution.enabled {
            if let Some(ref breakdown) = cost {
                let event = CostEvent {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    user_id: user_id.map(String::from),
                    session_id: session_id.map(String::from),
                    feature_tag: feature_tag.map(String::from),
                    environment: None,
                    prompt_version: None,
                    model: model.to_string(),
                    provider: provider.to_string(),
                    input_tokens,
                    output_tokens,
                    thinking_tokens,
                    cached_tokens,
                    total_cost_usd: breakdown.total_cost,
                };
                let mut attributor = self.attributor.lock().unwrap();
                attributor.record_event(event);
            }
        }

        // Persist usage to file storage (fire-and-forget)
        if let Some(ref storage) = self.storage {
            let record = TokenUsageRecord {
                id: uuid::Uuid::new_v4(),
                user_id: user_id.unwrap_or("default").to_string(),
                session_id: session_id.map(String::from),
                operation: "call".to_string(),
                tokens_used: (input_tokens + output_tokens) as u64,
                timestamp: Utc::now(),
                metadata: serde_json::json!({
                    "model": model,
                    "provider": provider,
                    "prompt_tokens": input_tokens,
                    "completion_tokens": output_tokens,
                    "cached_tokens": cached_tokens,
                    "thinking_tokens": thinking_tokens,
                    "cost_usd": cost.as_ref().map(|c| c.total_cost),
                }),
            };
            let storage = storage.clone();
            // Fire-and-forget only if a Tokio runtime is active.
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let _ = storage.save_usage(&record).await;
                });
            }
        }

        cost
    }

    /// Create a streaming token counter for a new stream
    pub fn create_stream_counter(&self) -> StreamingTokenCounter {
        StreamingTokenCounter::new(self.config.budget.max_tokens_per_call)
    }

    /// Get pricing registry reference
    pub fn pricing(&self) -> &PricingRegistry {
        &self.pricing
    }

    /// Get usage summary
    pub fn usage_summary(&self) -> crate::enforcement::UsageSummary {
        let enforcer = self.enforcer.lock().unwrap();
        enforcer.usage_summary()
    }

    /// Get pending alerts (drains the alert queue).
    pub fn drain_alerts(&self) -> Vec<crate::enforcement::Alert> {
        let mut enforcer = self.enforcer.lock().unwrap();
        enforcer.drain_alerts()
    }

    /// Get cost breakdown by feature
    pub fn cost_by_feature(&self) -> std::collections::HashMap<String, f64> {
        let attributor = self.attributor.lock().unwrap();
        attributor.cost_by_feature()
    }

    /// Get cost breakdown by user
    pub fn cost_by_user(&self) -> std::collections::HashMap<String, f64> {
        let attributor = self.attributor.lock().unwrap();
        attributor.cost_by_user()
    }

    /// Get cost breakdown by model
    pub fn cost_by_model(&self) -> std::collections::HashMap<String, f64> {
        let attributor = self.attributor.lock().unwrap();
        attributor.cost_by_model()
    }

    /// Get total cost
    pub fn total_cost(&self) -> f64 {
        let attributor = self.attributor.lock().unwrap();
        attributor.total_cost()
    }

    /// Get total events
    pub fn total_events(&self) -> usize {
        let attributor = self.attributor.lock().unwrap();
        attributor.total_events()
    }

    /// Get alerts
    pub fn alerts(&self) -> Vec<crate::enforcement::Alert> {
        let enforcer = self.enforcer.lock().unwrap();
        enforcer.alerts().to_vec()
    }

    /// Get config reference
    pub fn config(&self) -> &TokenomicsConfig {
        &self.config
    }
}

/// Shared tokenomics manager for use across the runtime
pub type SharedTokenomicsManager = Arc<TokenomicsManager>;

/// Create a shared manager from config
pub fn create_shared_manager(config: TokenomicsConfig) -> SharedTokenomicsManager {
    Arc::new(TokenomicsManager::from_config(config))
}

//! Provider Routing — Issue #630
//!
//! Routes LLM requests to the appropriate provider based on a configurable
//! strategy (round-robin, least-latency, cost-optimized, or manual).
//! Tracks provider health and automatically marks providers as unhealthy
//! after repeated failures, with auto-recovery after a cooldown period.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Routing strategy
// ---------------------------------------------------------------------------

/// Strategy used by [`ProviderRouter`] to select a provider for each request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RoutingStrategy {
    /// Distribute requests evenly across healthy providers.
    #[default]
    RoundRobin,
    /// Prefer the provider with the lowest average latency.
    LeastLatency,
    /// Prefer the cheapest provider (based on `cost_per_1k_tokens`).
    CostOptimized,
    /// Always route to a specific provider by name.
    Manual(String),
}

// ---------------------------------------------------------------------------
// Provider health
// ---------------------------------------------------------------------------

/// Health snapshot for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Provider name (matches the key in [`crate::ProviderRegistry`]).
    pub provider_name: String,
    /// Whether the provider is currently considered healthy.
    pub is_healthy: bool,
    /// Rolling average latency in milliseconds.
    pub avg_latency_ms: f64,
    /// Consecutive error count (resets on success).
    pub error_count: u32,
    /// Total successful calls.
    pub success_count: u64,
    /// Estimated cost per 1K tokens (USD). Used by `CostOptimized`.
    pub cost_per_1k_tokens: f64,
    /// When this provider was last checked (serialised as millis since creation).
    #[serde(skip)]
    pub last_check: Option<Instant>,
    /// When the provider was marked unhealthy (for auto-recovery).
    #[serde(skip)]
    pub marked_unhealthy_at: Option<Instant>,
}

impl ProviderHealth {
    /// Create a new healthy provider entry.
    pub fn new(provider_name: impl Into<String>, cost_per_1k_tokens: f64) -> Self {
        Self {
            provider_name: provider_name.into(),
            is_healthy: true,
            avg_latency_ms: 0.0,
            error_count: 0,
            success_count: 0,
            cost_per_1k_tokens,
            last_check: None,
            marked_unhealthy_at: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Router configuration
// ---------------------------------------------------------------------------

/// Configuration knobs for the router.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Number of consecutive failures before a provider is marked unhealthy.
    pub max_failures: u32,
    /// Duration after which an unhealthy provider is automatically retried.
    pub recovery_timeout: Duration,
    /// Routing strategy.
    pub strategy: RoutingStrategy,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_failures: 3,
            recovery_timeout: Duration::from_secs(60),
            strategy: RoutingStrategy::RoundRobin,
        }
    }
}

// ---------------------------------------------------------------------------
// ProviderRouter
// ---------------------------------------------------------------------------

/// Routes requests to the best available provider based on health and strategy.
pub struct ProviderRouter {
    /// Per-provider health state.
    health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    /// Ordered list of provider names (insertion order) for round-robin.
    provider_order: Arc<RwLock<Vec<String>>>,
    /// Round-robin counter.
    rr_counter: AtomicUsize,
    /// Configuration.
    config: RouterConfig,
}

impl ProviderRouter {
    /// Create a new router with the given configuration.
    pub fn new(config: RouterConfig) -> Self {
        Self {
            health: Arc::new(RwLock::new(HashMap::new())),
            provider_order: Arc::new(RwLock::new(Vec::new())),
            rr_counter: AtomicUsize::new(0),
            config,
        }
    }

    /// Register a provider with the router.
    pub async fn add_provider(&self, name: impl Into<String>, cost_per_1k_tokens: f64) {
        let name = name.into();
        let mut health = self.health.write().await;
        if !health.contains_key(&name) {
            let mut order = self.provider_order.write().await;
            order.push(name.clone());
        }
        health.insert(name.clone(), ProviderHealth::new(name, cost_per_1k_tokens));
    }

    /// Remove a provider from the router.
    pub async fn remove_provider(&self, name: &str) {
        self.health.write().await.remove(name);
        let mut order = self.provider_order.write().await;
        order.retain(|n| n != name);
    }

    /// Select the best provider for the next request.
    ///
    /// Returns `None` if no healthy provider is available.
    pub async fn route(&self) -> Option<String> {
        // Auto-recover providers whose cooldown has elapsed.
        self.auto_recover().await;

        match &self.config.strategy {
            RoutingStrategy::Manual(name) => {
                let health = self.health.read().await;
                health
                    .get(name)
                    .filter(|h| h.is_healthy)
                    .map(|h| h.provider_name.clone())
            }
            RoutingStrategy::RoundRobin => self.route_round_robin().await,
            RoutingStrategy::LeastLatency => self.route_least_latency().await,
            RoutingStrategy::CostOptimized => self.route_cost_optimized().await,
        }
    }

    /// Record a successful call, updating latency and resetting error count.
    pub async fn record_success(&self, provider: &str, latency_ms: f64) {
        let mut health = self.health.write().await;
        if let Some(h) = health.get_mut(provider) {
            h.error_count = 0;
            h.success_count += 1;
            h.last_check = Some(Instant::now());
            // Exponential moving average (alpha = 0.3).
            if h.avg_latency_ms == 0.0 {
                h.avg_latency_ms = latency_ms;
            } else {
                h.avg_latency_ms = h.avg_latency_ms * 0.7 + latency_ms * 0.3;
            }
            debug!(
                "router: provider '{}' success, avg_latency={:.1}ms",
                provider, h.avg_latency_ms
            );
        }
    }

    /// Record a failed call. If the consecutive error count exceeds
    /// `max_failures`, the provider is marked unhealthy.
    pub async fn record_failure(&self, provider: &str) {
        let mut health = self.health.write().await;
        if let Some(h) = health.get_mut(provider) {
            h.error_count += 1;
            h.last_check = Some(Instant::now());
            if h.error_count >= self.config.max_failures && h.is_healthy {
                warn!(
                    "router: marking provider '{}' unhealthy after {} consecutive failures",
                    provider, h.error_count
                );
                h.is_healthy = false;
                h.marked_unhealthy_at = Some(Instant::now());
            }
        }
    }

    /// Explicitly mark a provider as unhealthy.
    pub async fn mark_unhealthy(&self, provider: &str) {
        let mut health = self.health.write().await;
        if let Some(h) = health.get_mut(provider) {
            h.is_healthy = false;
            h.marked_unhealthy_at = Some(Instant::now());
            warn!("router: provider '{}' manually marked unhealthy", provider);
        }
    }

    /// Explicitly mark a provider as healthy.
    pub async fn mark_healthy(&self, provider: &str) {
        let mut health = self.health.write().await;
        if let Some(h) = health.get_mut(provider) {
            h.is_healthy = true;
            h.error_count = 0;
            h.marked_unhealthy_at = None;
            debug!("router: provider '{}' manually marked healthy", provider);
        }
    }

    /// Return a snapshot of all provider health states.
    pub async fn health_snapshot(&self) -> Vec<ProviderHealth> {
        self.health.read().await.values().cloned().collect()
    }

    /// Return the current routing strategy.
    pub fn strategy(&self) -> &RoutingStrategy {
        &self.config.strategy
    }

    /// Change the routing strategy at runtime.
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.config.strategy = strategy;
    }

    // -- private helpers ---------------------------------------------------

    /// Auto-recover providers whose cooldown has expired.
    async fn auto_recover(&self) {
        let mut health = self.health.write().await;
        for h in health.values_mut() {
            if !h.is_healthy {
                if let Some(marked_at) = h.marked_unhealthy_at {
                    if marked_at.elapsed() >= self.config.recovery_timeout {
                        debug!(
                            "router: auto-recovering provider '{}' after cooldown",
                            h.provider_name
                        );
                        h.is_healthy = true;
                        h.error_count = 0;
                        h.marked_unhealthy_at = None;
                    }
                }
            }
        }
    }

    async fn route_round_robin(&self) -> Option<String> {
        let order = self.provider_order.read().await;
        let health = self.health.read().await;

        let healthy: Vec<&String> = order
            .iter()
            .filter(|n| health.get(*n).is_some_and(|h| h.is_healthy))
            .collect();

        if healthy.is_empty() {
            return None;
        }

        let idx = self.rr_counter.fetch_add(1, Ordering::Relaxed) % healthy.len();
        Some(healthy[idx].clone())
    }

    async fn route_least_latency(&self) -> Option<String> {
        let health = self.health.read().await;
        health
            .values()
            .filter(|h| h.is_healthy)
            .min_by(|a, b| {
                a.avg_latency_ms
                    .partial_cmp(&b.avg_latency_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|h| h.provider_name.clone())
    }

    async fn route_cost_optimized(&self) -> Option<String> {
        let health = self.health.read().await;
        health
            .values()
            .filter(|h| h.is_healthy)
            .min_by(|a, b| {
                a.cost_per_1k_tokens
                    .partial_cmp(&b.cost_per_1k_tokens)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|h| h.provider_name.clone())
    }
}

impl Default for ProviderRouter {
    fn default() -> Self {
        Self::new(RouterConfig::default())
    }
}

//! HudHudScript Utilities
//!
//! Common utility functions for retry, rate limiting, debounce, throttle,
//! caching, registry, and error handling.

pub mod cache_strategy;
pub mod cache_trait;
pub mod debounce;
pub mod error;
pub mod rate_limiter;
pub mod registry_trait;
pub mod retry;
pub mod simple_cache;
pub mod throttle;

pub use cache_strategy::{CacheStrategy, LookupMode};
pub use cache_trait::{Cache, CacheStats, EvictionPolicy};
pub use debounce::Debouncer;
pub use error::{HudHudError, HudHudResult, IntoHudHudError};
pub use rate_limiter::RateLimiter;
pub use registry_trait::{Registry, SearchableRegistry};
pub use retry::{retry, retry_sync, RetryConfig, RetryError};
pub use simple_cache::SimpleLruCache;
pub use throttle::Throttle;

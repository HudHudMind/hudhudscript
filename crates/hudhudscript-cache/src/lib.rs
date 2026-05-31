//! # HudHudScript Cache
//!
//! Command cache implementation for governance structures.
//! Enables token optimization by caching governance structures and referencing
//! them via integer IDs instead of transmitting full definitions.
//!
//! ## Lifecycle management
//!
//! - **Eviction** — LRU and LFU eviction policies via [`eviction::EvictionEngine`]
//! - **Quota** — Disk/memory quota monitoring with alerts via [`quota::QuotaMonitor`]
//! - **Index** — Unified cache index across all types via [`index::CacheIndex`]
//! - **Dedup** — Content-addressable deduplication via [`dedup::DedupStore`]
//! - **Prune/Cleanup** — Cache pruning and statistics via [`cache::ManagedCache`]

pub mod cache;
pub mod dedup;
pub mod eviction;
pub mod index;
pub mod quota;
pub mod serialization;
pub mod transmission;

pub use cache::*;
pub use dedup::*;
pub use eviction::*;
pub use index::*;
pub use quota::*;
pub use serialization::*;
pub use transmission::*;

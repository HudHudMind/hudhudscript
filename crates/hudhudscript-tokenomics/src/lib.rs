//! # HudHudScript Tokenomics
//!
//! ML-powered token budget optimization and predictive analytics system.
//!
//! ## Features
//!
//! - **ML-Powered Optimization**: Dynamic, learning-based token management
//! - **Predictive Budgeting**: Proactive budget allocation based on usage patterns
//! - **Federated Learning**: Privacy-preserving collective intelligence
//! - **Reinforcement Learning**: Continuous improvement from user feedback
//! - **Real-time Analytics**: Live monitoring and optimization
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Tokenomics Engine                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ML Models          │  Predictive      │  Federated         │
//! │  - ONNX Runtime     │  - Prophet       │  - Privacy         │
//! │  - Reinforcement    │  - Time Series   │  - Forecasting     │
//! │  - Classification   │  - Forecasting   │  - Sync            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Storage Layer      │  Cache Layer     │  Analytics         │
//! │  - TimescaleDB      │  - Redis         │  - Metrics         │
//! │  - Time Series      │  - Features      │  - Dashboards      │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod analytics;
pub mod attribution;
pub mod bandit;
pub mod batch;
pub mod budget;
pub mod cache;
pub mod cascade;
pub mod cli;
pub mod compression;
pub mod config;
pub mod embedding_cache;
pub mod enforcement;
pub mod error;
pub mod federated;
pub mod forecasting;
pub mod integration;
pub mod metrics;
pub mod ml;
pub mod multimodal;
pub mod optimizer;
pub mod pricing;
pub mod prompt_caching;
pub mod reinforcement;
pub mod response_cache;
pub mod storage;
pub mod streaming;
pub mod thinking;
pub mod tool_tracking;
pub mod types;

pub use error::{Result, TokenomicsError};
pub use integration::{create_shared_manager, SharedTokenomicsManager, TokenomicsManager};
pub use optimizer::TokenomicsEngine;
pub use types::{Budget, OptimizationStrategy, Prediction, TokenUsageRecord};

/// Version of the tokenomics system
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default token budget for new users
pub const DEFAULT_BUDGET: u64 = 100_000;

/// Minimum budget threshold before warning
pub const MIN_BUDGET_THRESHOLD: u64 = 10_000;

/// Maximum prediction horizon in seconds (24 hours)
pub const MAX_PREDICTION_HORIZON: u64 = 86_400;

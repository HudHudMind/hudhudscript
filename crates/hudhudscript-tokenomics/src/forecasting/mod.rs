//! Prophet and linfa integration stubs.
//!
//! These provide the interface for advanced ML features that activate
//! behind feature flags (`prophet-forecasting` and `ml`).
//! When flags are disabled, these stubs provide fallback behavior.

pub mod cluster;
pub mod engine;
pub mod forecaster;
pub mod regress;
pub mod result;

pub use cluster::UsageClusterer;
pub use forecaster::AdvancedForecaster;
pub use regress::CostRegressor;
pub use result::{ForecastMethod, ForecastResult};

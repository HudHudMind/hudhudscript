//! Forecast result types and method enum.

/// Prophet-style forecasting result.
///
/// The `method` field records which algorithm actually produced the result —
/// callers should inspect it to detect fallback. The `requested_method` and
/// `is_fallback` fields make this explicit. (v0.4.47.9 — Issue #835)
#[derive(Debug, Clone)]
pub struct ForecastResult {
    pub predicted_values: Vec<f64>,
    pub lower_bound: Vec<f64>,
    pub upper_bound: Vec<f64>,
    pub confidence: f64,
    pub method: String,
    /// The method the caller originally requested (may differ from `method`
    /// if a fallback was used).
    pub requested_method: String,
    /// True if the actual method differs from the requested method
    /// (i.e., a fallback was used).
    pub is_fallback: bool,
    /// Reason for fallback (if `is_fallback == true`).
    pub fallback_reason: Option<String>,
}

/// Forecasting method selection
#[derive(Debug, Clone, PartialEq)]
pub enum ForecastMethod {
    /// Holt's exponential smoothing (always available)
    Holt,
    /// Facebook Prophet (requires `prophet-forecasting` feature)
    Prophet,
    /// ARIMA (future, not yet implemented)
    Arima,
}

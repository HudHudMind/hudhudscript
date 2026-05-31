//! Time-series forecasting using double exponential smoothing (Holt's method)

use crate::error::Result;
use crate::types::{Prediction, TimeSeriesPoint};

/// Forecasting engine for token usage.
pub struct ForecastingEngine {
    /// Smoothing factor for level (0 < alpha < 1).
    alpha: f64,
    /// Smoothing factor for trend (0 < beta < 1).
    beta: f64,
}

impl ForecastingEngine {
    pub fn new() -> Self {
        Self {
            alpha: 0.3,
            beta: 0.1,
        }
    }

    /// Create with custom smoothing parameters.
    pub fn with_params(alpha: f64, beta: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.01, 0.99),
            beta: beta.clamp(0.01, 0.99),
        }
    }

    /// Forecast future token usage using double exponential smoothing.
    pub async fn forecast(
        &self,
        data: &[TimeSeriesPoint],
        horizon_seconds: u64,
    ) -> Result<Prediction> {
        if data.is_empty() {
            return Ok(Prediction {
                predicted_usage: 0,
                confidence: 0.0,
                horizon_seconds,
                timestamp: chrono::Utc::now(),
                model_version: "holt-ses-v1".to_string(),
            });
        }

        if data.len() == 1 {
            let val = data[0].value;
            return Ok(Prediction {
                predicted_usage: (val * horizon_seconds as f64 / 3600.0).max(0.0) as u64,
                confidence: 0.3,
                horizon_seconds,
                timestamp: chrono::Utc::now(),
                model_version: "holt-ses-v1".to_string(),
            });
        }

        let mut sorted: Vec<&TimeSeriesPoint> = data.iter().collect();
        sorted.sort_by_key(|p| p.timestamp);
        let values: Vec<f64> = sorted.iter().map(|p| p.value).collect();

        // Initialise level and trend.
        let mut level = values[0];
        let mut trend = values[1] - values[0];

        // Track absolute percentage errors for confidence estimation.
        let mut ape_sum = 0.0;
        let mut ape_count = 0usize;

        for observed in &values[1..] {
            let prev_forecast = level + trend;
            let observed = *observed;

            let new_level = self.alpha * observed + (1.0 - self.alpha) * (level + trend);
            let new_trend = self.beta * (new_level - level) + (1.0 - self.beta) * trend;

            if observed.abs() > f64::EPSILON {
                ape_sum += ((observed - prev_forecast) / observed).abs();
                ape_count += 1;
            }

            level = new_level;
            trend = new_trend;
        }

        // Estimate periods ahead.
        let total_span_secs = (sorted.last().unwrap().timestamp - sorted.first().unwrap().timestamp)
            .num_seconds()
            .max(1) as f64;
        let avg_period_secs = total_span_secs / (values.len() - 1) as f64;
        let periods_ahead = horizon_seconds as f64 / avg_period_secs.max(1.0);

        let forecast_value = level + trend * periods_ahead;

        let mape = if ape_count > 0 {
            ape_sum / ape_count as f64
        } else {
            0.5
        };
        let confidence = (1.0 - mape).clamp(0.1, 0.99);

        Ok(Prediction {
            predicted_usage: forecast_value.max(0.0) as u64,
            confidence,
            horizon_seconds,
            timestamp: chrono::Utc::now(),
            model_version: "holt-ses-v1".to_string(),
        })
    }
}

impl Default for ForecastingEngine {
    fn default() -> Self {
        Self::new()
    }
}

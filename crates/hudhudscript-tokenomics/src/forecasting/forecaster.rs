//! Advanced forecaster — dispatches to Holt, Prophet, or ARIMA.

use crate::error::{Result, TokenomicsError};
use crate::types::TimeSeriesPoint;

use super::{ForecastMethod, ForecastResult};

/// Forecasting engine — dispatches to Prophet or fallback Holt's method
pub struct AdvancedForecaster {
    method: ForecastMethod,
}

impl AdvancedForecaster {
    pub fn new(method: ForecastMethod) -> Self {
        Self { method }
    }

    /// Forecast future values from time series data
    pub fn forecast(&self, data: &[TimeSeriesPoint], horizon: usize) -> Result<ForecastResult> {
        if data.len() < 3 {
            return Err(TokenomicsError::ColdStart);
        }

        let requested = format!("{:?}", self.method);
        match self.method {
            ForecastMethod::Holt => {
                let mut r = self.holt_forecast(data, horizon)?;
                r.requested_method = requested;
                r.is_fallback = false;
                Ok(r)
            }
            ForecastMethod::Prophet => {
                #[cfg(feature = "prophet-forecasting")]
                {
                    let mut r = self.prophet_forecast(data, horizon)?;
                    r.requested_method = requested;
                    r.is_fallback = false;
                    Ok(r)
                }
                #[cfg(not(feature = "prophet-forecasting"))]
                {
                    tracing::warn!(
                        "Prophet not available (feature 'prophet-forecasting' disabled), \
                         falling back to Holt's method"
                    );
                    let mut r = self.holt_forecast(data, horizon)?;
                    r.requested_method = requested;
                    r.is_fallback = true;
                    r.fallback_reason =
                        Some("Prophet feature not enabled at compile time".to_string());
                    Ok(r)
                }
            }
            ForecastMethod::Arima => self.arima_forecast(data, horizon),
        }
    }

    /// Holt's double exponential smoothing
    fn holt_forecast(&self, data: &[TimeSeriesPoint], horizon: usize) -> Result<ForecastResult> {
        let values: Vec<f64> = data.iter().map(|p| p.value).collect();
        let n = values.len();
        if n < 2 {
            return Err(TokenomicsError::ColdStart);
        }

        let alpha = 0.3; // level smoothing
        let beta = 0.1; // trend smoothing

        let mut level = values[0];
        let mut trend = values[1] - values[0];

        for &val in &values[1..] {
            let new_level = alpha * val + (1.0 - alpha) * (level + trend);
            let new_trend = beta * (new_level - level) + (1.0 - beta) * trend;
            level = new_level;
            trend = new_trend;
        }

        // Compute residual std for confidence intervals
        let mut residuals = Vec::new();
        let mut l = values[0];
        let mut t = values[1] - values[0];
        for &val in &values[1..] {
            let predicted = l + t;
            residuals.push((val - predicted).abs());
            let nl = alpha * val + (1.0 - alpha) * (l + t);
            let nt = beta * (nl - l) + (1.0 - beta) * t;
            l = nl;
            t = nt;
        }
        let residual_std = if !residuals.is_empty() {
            (residuals.iter().map(|r| r * r).sum::<f64>() / residuals.len() as f64).sqrt()
        } else {
            0.0
        };

        let mut predicted = Vec::with_capacity(horizon);
        let mut lower = Vec::with_capacity(horizon);
        let mut upper = Vec::with_capacity(horizon);

        for h in 1..=horizon {
            let pred = level + trend * h as f64;
            let interval = 1.96 * residual_std * (h as f64).sqrt();
            predicted.push(pred.max(0.0));
            lower.push((pred - interval).max(0.0));
            upper.push(pred + interval);
        }

        Ok(ForecastResult {
            predicted_values: predicted,
            lower_bound: lower,
            upper_bound: upper,
            confidence: 0.7,
            method: "holt".into(),
            requested_method: "Holt".into(), // overridden by caller if needed
            is_fallback: false,
            fallback_reason: None,
        })
    }

    #[cfg(feature = "prophet-forecasting")]
    fn prophet_forecast(&self, data: &[TimeSeriesPoint], horizon: usize) -> Result<ForecastResult> {
        Err(TokenomicsError::ModelError(format!(
            "Prophet forecasting requires the 'prophet' crate to be linked. \
             Got {} data points, horizon {}. Use ForecastMethod::Holt instead.",
            data.len(),
            horizon
        )))
    }

    /// Real AR(1) (ARIMA(1,0,0)) forecast.
    fn arima_forecast(&self, data: &[TimeSeriesPoint], horizon: usize) -> Result<ForecastResult> {
        let values: Vec<f64> = data.iter().map(|p| p.value).collect();
        let n = values.len();
        if n < 3 {
            return Err(TokenomicsError::ColdStart);
        }

        let mean: f64 = values.iter().sum::<f64>() / n as f64;
        let centered: Vec<f64> = values.iter().map(|v| v - mean).collect();

        let gamma_0: f64 = centered.iter().map(|v| v * v).sum::<f64>() / n as f64;
        let gamma_1: f64 = (0..n - 1)
            .map(|i| centered[i] * centered[i + 1])
            .sum::<f64>()
            / (n - 1) as f64;

        if gamma_0.abs() < f64::EPSILON {
            let predicted = vec![mean; horizon];
            return Ok(ForecastResult {
                predicted_values: predicted.clone(),
                lower_bound: predicted.clone(),
                upper_bound: predicted,
                confidence: 0.5,
                method: "arima(1,0,0)".into(),
                requested_method: "Arima".into(),
                is_fallback: false,
                fallback_reason: None,
            });
        }

        let phi = (gamma_1 / gamma_0).clamp(-0.99, 0.99);

        let mut residuals = Vec::with_capacity(n - 1);
        for i in 1..n {
            let predicted = mean + phi * (values[i - 1] - mean);
            residuals.push(values[i] - predicted);
        }
        let residual_var: f64 =
            residuals.iter().map(|r| r * r).sum::<f64>() / residuals.len() as f64;

        let mut predicted = Vec::with_capacity(horizon);
        let mut lower = Vec::with_capacity(horizon);
        let mut upper = Vec::with_capacity(horizon);

        let mut last = values[n - 1];
        let mut variance_sum = 0.0;
        for h in 1..=horizon {
            let pred = mean + phi * (last - mean);
            variance_sum += phi.powi(2 * (h - 1) as i32);
            let forecast_var = residual_var * variance_sum;
            let interval = 1.96 * forecast_var.sqrt();

            predicted.push(pred.max(0.0));
            lower.push((pred - interval).max(0.0));
            upper.push(pred + interval);

            last = pred;
        }

        let confidence = 0.5 + 0.4 * phi.abs();

        Ok(ForecastResult {
            predicted_values: predicted,
            lower_bound: lower,
            upper_bound: upper,
            confidence,
            method: "arima(1,0,0)".into(),
            requested_method: "Arima".into(),
            is_fallback: false,
            fallback_reason: None,
        })
    }

    /// Check if the requested method is available
    pub fn is_method_available(&self) -> bool {
        match self.method {
            ForecastMethod::Holt => true,
            ForecastMethod::Prophet => cfg!(feature = "prophet-forecasting"),
            ForecastMethod::Arima => false,
        }
    }
}

//! Simple linear regression for cost prediction (y = ax + b).
///
/// Treats the feature slice as a sequence of (x, y) pairs where x is the
/// index (0, 1, 2, ...) and y is the feature value. Computes slope and
/// intercept via ordinary least squares, then predicts the next value.

/// Simple linear regression for cost prediction.
pub struct CostRegressor;

impl CostRegressor {
    /// Predict the next cost value using simple linear regression.
    ///
    /// Given a series of observed values, fits y = a*x + b and returns
    /// the predicted value at x = n (one step ahead).
    pub fn predict(features: &[f64]) -> f64 {
        if features.is_empty() {
            return 0.0;
        }
        let n = features.len() as f64;
        if features.len() == 1 {
            return features[0];
        }

        // OLS: slope a = (n * sum(x*y) - sum(x)*sum(y)) / (n * sum(x^2) - (sum(x))^2)
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, &y) in features.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            return sum_y / n;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;

        // Predict at x = n (next step)
        slope * n + intercept
    }
}

//! Machine Learning models for token optimization

use crate::error::{Result, TokenomicsError};
use crate::types::{ModelMetadata, Prediction, TimeSeriesPoint};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ML model for token usage prediction
pub struct TokenPredictionModel {
    metadata: ModelMetadata,
    weights: Vec<f64>,
    bias: f64,
    feature_count: usize,
}

impl TokenPredictionModel {
    /// Create a new model with random initialization
    pub fn new(feature_count: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            metadata: ModelMetadata {
                name: "token_predictor".to_string(),
                version: "0.1.0".to_string(),
                trained_at: Utc::now(),
                accuracy: 0.0,
                samples_count: 0,
            },
            weights: (0..feature_count)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect(),
            bias: rng.gen_range(-1.0..1.0),
            feature_count,
        }
    }

    /// Train the model with historical data
    pub fn train(&mut self, features: &[Vec<f64>], targets: &[f64]) -> Result<()> {
        if features.is_empty() || targets.is_empty() {
            return Err(TokenomicsError::ColdStart);
        }

        if features.len() != targets.len() {
            return Err(TokenomicsError::ModelError(
                "Features and targets length mismatch".to_string(),
            ));
        }

        // Simple gradient descent
        let learning_rate = 0.01;
        let epochs = 100;

        for _ in 0..epochs {
            let mut weight_gradients = vec![0.0; self.feature_count];
            let mut bias_gradient = 0.0;

            for (feature_vec, &target) in features.iter().zip(targets.iter()) {
                let prediction = self.predict_raw(feature_vec)?;
                let error = prediction - target;

                for (i, &feature) in feature_vec.iter().enumerate() {
                    weight_gradients[i] += error * feature;
                }
                bias_gradient += error;
            }

            // Update weights
            for (weight, gradient) in self.weights.iter_mut().zip(weight_gradients.iter()) {
                *weight -= learning_rate * gradient / features.len() as f64;
            }
            self.bias -= learning_rate * bias_gradient / features.len() as f64;
        }

        // Calculate accuracy
        let mut correct = 0;
        for (feature_vec, &target) in features.iter().zip(targets.iter()) {
            let prediction = self.predict_raw(feature_vec)?;
            if (prediction - target).abs() < target * 0.1 {
                // Within 10% error
                correct += 1;
            }
        }

        self.metadata.accuracy = correct as f64 / features.len() as f64;
        self.metadata.samples_count = features.len();
        self.metadata.trained_at = Utc::now();

        Ok(())
    }

    /// Predict token usage
    pub fn predict(&self, features: &[f64], horizon_seconds: u64) -> Result<Prediction> {
        let predicted_usage = self.predict_raw(features)?;

        Ok(Prediction {
            predicted_usage: predicted_usage.max(0.0) as u64,
            confidence: self.metadata.accuracy,
            horizon_seconds,
            timestamp: Utc::now(),
            model_version: self.metadata.version.clone(),
        })
    }

    fn predict_raw(&self, features: &[f64]) -> Result<f64> {
        if features.len() != self.feature_count {
            return Err(TokenomicsError::ModelError(format!(
                "Expected {} features, got {}",
                self.feature_count,
                features.len()
            )));
        }

        let mut result = self.bias;
        for (weight, feature) in self.weights.iter().zip(features.iter()) {
            result += weight * feature;
        }

        Ok(result)
    }

    /// Save model to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let data = ModelData {
            metadata: self.metadata.clone(),
            weights: self.weights.clone(),
            bias: self.bias,
            feature_count: self.feature_count,
        };

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load model from disk
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let data: ModelData = serde_json::from_str(&json)?;

        Ok(Self {
            metadata: data.metadata,
            weights: data.weights,
            bias: data.bias,
            feature_count: data.feature_count,
        })
    }

    /// Get model metadata
    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    /// Check if model needs retraining
    pub fn needs_retraining(&self) -> bool {
        self.metadata.samples_count == 0
    }
}

#[derive(Serialize, Deserialize)]
struct ModelData {
    metadata: ModelMetadata,
    weights: Vec<f64>,
    bias: f64,
    feature_count: usize,
}

/// Feature extractor for token usage patterns
pub struct FeatureExtractor;

impl FeatureExtractor {
    /// Extract features from time series data
    pub fn extract(data: &[TimeSeriesPoint]) -> Vec<f64> {
        if data.is_empty() {
            return vec![0.0; 10]; // Return zero features
        }

        let values: Vec<f64> = data.iter().map(|p| p.value).collect();

        vec![
            Self::mean(&values),
            Self::std_dev(&values),
            Self::min(&values),
            Self::max(&values),
            Self::median(&values),
            Self::trend(&values),
            Self::volatility(&values),
            values.len() as f64,
            Self::recent_average(&values, 5),
            Self::momentum(&values),
        ]
    }

    fn mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    fn std_dev(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mean = Self::mean(values);
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    fn min(values: &[f64]) -> f64 {
        values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    fn max(values: &[f64]) -> f64 {
        values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    fn median(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    fn trend(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        (values[values.len() - 1] - values[0]) / values.len() as f64
    }

    fn volatility(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        let changes: Vec<f64> = values.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        Self::mean(&changes)
    }

    fn recent_average(values: &[f64], n: usize) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let start = values.len().saturating_sub(n);
        Self::mean(&values[start..])
    }

    fn momentum(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        let recent = Self::recent_average(values, 5);
        let overall = Self::mean(values);
        recent - overall
    }
}

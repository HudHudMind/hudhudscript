//! Federated learning for privacy-preserving model training
//!
//! Implements the Federated Averaging (FedAvg) algorithm.

use crate::error::{Result, TokenomicsError};
use crate::types::FederatedUpdate;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Federated learning coordinator.
pub struct FederatedLearning {
    global_weights: Arc<RwLock<Vec<f64>>>,
    min_updates: usize,
    pending_updates: Arc<RwLock<Vec<FederatedUpdate>>>,
    model_version: Arc<RwLock<u64>>,
}

impl FederatedLearning {
    pub fn new(_enabled: bool) -> Self {
        Self {
            global_weights: Arc::new(RwLock::new(Vec::new())),
            min_updates: 1,
            pending_updates: Arc::new(RwLock::new(Vec::new())),
            model_version: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with a specific minimum-update threshold.
    pub fn with_min_updates(_enabled: bool, min_updates: usize) -> Self {
        Self {
            min_updates: min_updates.max(1),
            ..Self::new(true)
        }
    }

    /// Initialise the global model to the given weights.
    pub async fn init_global_model(&self, weights: Vec<f64>) {
        let mut gw = self.global_weights.write().await;
        *gw = weights;
        let mut ver = self.model_version.write().await;
        *ver = 0;
    }

    /// Submit local model update
    pub async fn submit_update(&self, update: FederatedUpdate) -> Result<()> {
        if update.gradients.is_empty() {
            return Err(TokenomicsError::FederatedError(
                "empty gradient vector".into(),
            ));
        }
        self.pending_updates.write().await.push(update);
        Ok(())
    }

    /// Aggregate updates from multiple clients using FedAvg (weighted average
    /// by sample_count).
    pub async fn aggregate_updates(&self, updates: Vec<FederatedUpdate>) -> Result<Vec<f64>> {
        if updates.is_empty() {
            return Err(TokenomicsError::FederatedError(
                "no updates to aggregate".into(),
            ));
        }

        let dim = updates[0].gradients.len();
        for u in &updates {
            if u.gradients.len() != dim {
                return Err(TokenomicsError::FederatedError(format!(
                    "dimension mismatch: expected {dim}, got {}",
                    u.gradients.len()
                )));
            }
        }

        let total_samples: f64 = updates.iter().map(|u| u.sample_count as f64).sum();
        if total_samples < f64::EPSILON {
            return Err(TokenomicsError::FederatedError(
                "total sample count is zero".into(),
            ));
        }

        let mut avg = vec![0.0f64; dim];
        for u in &updates {
            let weight = u.sample_count as f64 / total_samples;
            for (j, g) in u.gradients.iter().enumerate() {
                avg[j] += weight * g;
            }
        }

        Ok(avg)
    }

    /// Run a full federated round: drain pending updates, aggregate, apply.
    pub async fn run_round(&self) -> Result<Vec<f64>> {
        let updates: Vec<FederatedUpdate> = {
            let mut pending = self.pending_updates.write().await;
            if pending.len() < self.min_updates {
                return Err(TokenomicsError::FederatedError(format!(
                    "need at least {} updates, have {}",
                    self.min_updates,
                    pending.len()
                )));
            }
            pending.drain(..).collect()
        };

        let aggregated = self.aggregate_updates(updates).await?;
        self.apply_update(aggregated.clone()).await?;
        Ok(aggregated)
    }

    /// Apply aggregated update to global model
    pub async fn apply_update(&self, gradients: Vec<f64>) -> Result<()> {
        let mut gw = self.global_weights.write().await;

        if gw.is_empty() {
            *gw = gradients;
        } else {
            if gw.len() != gradients.len() {
                return Err(TokenomicsError::FederatedError(format!(
                    "gradient dimension {} does not match model dimension {}",
                    gradients.len(),
                    gw.len()
                )));
            }
            for (w, g) in gw.iter_mut().zip(gradients.iter()) {
                *w += g;
            }
        }

        let mut ver = self.model_version.write().await;
        *ver += 1;
        Ok(())
    }

    /// Get a snapshot of the current global model weights.
    pub async fn global_weights(&self) -> Vec<f64> {
        self.global_weights.read().await.clone()
    }

    /// Get the current model version counter.
    pub async fn model_version(&self) -> u64 {
        *self.model_version.read().await
    }
}

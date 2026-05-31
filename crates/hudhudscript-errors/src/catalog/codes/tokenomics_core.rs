use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum TokenomicsCoreErrorCode {
    /// E0277 — Named Budget Does Not Exist
    TokenomicsBudgetNotFound = 277,
    /// E0278 — Tokenomics Cache Layer Failure
    TokenomicsCacheError = 278,
    /// E0279 — ML Cost Predictor Has No Training Data
    TokenomicsColdStart = 279,
    /// E0280 — Tokenomics Configuration Is Invalid
    TokenomicsConfigError = 280,
    /// E0281 — Tokenomics Database Backend Failed
    TokenomicsDatabaseError = 281,
    /// E0282 — Federated Learning Sync Failed
    TokenomicsFederatedError = 282,
    /// E0283 — Not Enough Budget Remaining For Operation
    TokenomicsInsufficientBudget = 283,
    /// E0284 — Budget Amount Is Invalid
    TokenomicsInvalidBudget = 284,
    /// E0285 — Tokenomics File I/O Failure
    TokenomicsIoError = 285,
    /// E0286 — Cost Model Accuracy Has Dropped Below Threshold
    TokenomicsModelDrift = 286,
    /// E0287 — ML Cost Model Internal Failure
    TokenomicsModelError = 287,
    /// E0288 — Cost Model Is Overfitting Training Data
    TokenomicsOverfitting = 288,
    /// E0289 — Cost Prediction Could Not Be Computed
    TokenomicsPredictionFailed = 289,
    /// E0290 — Tokenomics Redis Backend Failed
    TokenomicsRedisError = 290,
    /// E0291 — Reinforcement Learning Step Failed
    TokenomicsReinforcementError = 291,
    /// E0292 — Tokenomics Value Serialization Failed
    TokenomicsSerializationError = 292,
    /// E0293 — Tokenomics Storage Backend Failed
    TokenomicsStorageError = 293,
    /// E0294 — Unclassified Tokenomics Error
    TokenomicsUnknown = 294,
}

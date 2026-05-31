use super::*;
use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const TOKENOMICS_INVALID_BUDGET: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(284),
        long_code: "HHS_E_TOKENOMICS_INVALID_BUDGET",
        short_code: "E0284",
        title: "Budget Amount Is Invalid",
        short_description: "A budget value was rejected because it is negative, NaN, or otherwise nonsensical.",
        long_description: "Budget amounts must be finite, non-negative numbers. This error fires when you try to create or update a budget with a value that violates those constraints — a negative limit, NaN, or infinity.

Validate inputs at the source. If the value comes from a user, surface a clear validation error rather than passing junk down. Currency conversions can also produce NaN if exchange rates are missing — guard against that explicitly.

The wrapped message echoes the offending value.",
        hints: &["Validate budget values at the input boundary", "Reject NaN and infinity before calling the API", "Guard currency conversion against missing rates", "Inspect the wrapped value for what was passed"],
        example_bad: Some("tokenomics.set_limit(\"team-a\", -100.0)?;"),
        example_good: Some("let limit = limit.max(0.0);
tokenomics.set_limit(\"team-a\", limit)?;"),
        see_also: &["TokenomicsBudgetNotFound", "TokenomicsConfigError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_IO_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(285),
        long_code: "HHS_E_TOKENOMICS_IO_ERROR",
        short_code: "E0285",
        title: "Tokenomics File I/O Failure",
        short_description: "Reading or writing tokenomics state files failed at the OS level.",
        long_description: "Tokenomics persists some state to flat files (snapshots, exported reports). This error wraps an OS-level failure during one of those reads or writes — permission denied, disk full, file not found.

Verify the directory exists and is writable, check disk space, and inspect the wrapped OS message for the exact cause. Most I/O errors are environmental rather than logic bugs.

For production deployments, monitor the tokenomics state directory like any other critical data path.",
        hints: &["Check disk space and permissions on the state directory", "Inspect the wrapped OS error for specifics", "Monitor the tokenomics state directory in production", "Avoid sharing the directory between processes without locks"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsDatabaseError", "TokenomicsStorageError", "PersistenceIo"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_MODEL_DRIFT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(286),
        long_code: "HHS_E_TOKENOMICS_MODEL_DRIFT",
        short_code: "E0286",
        title: "Cost Model Accuracy Has Dropped Below Threshold",
        short_description: "The ML cost predictor's measured accuracy fell below the configured drift threshold.",
        long_description: "HudHudScript continuously evaluates the cost predictor against actual observed costs. When accuracy drops below the threshold (often because provider pricing changed or your traffic mix shifted), this error signals that the current model is no longer trustworthy.

The runtime should automatically retrain or fall back to rule-based estimation when this fires. If you see it surface to user code, the caller forgot to wrap predictions in the drift-handling fallback.

Drift is normal and expected over time — the system is designed to detect and recover from it, not to prevent it.",
        hints: &["Trigger retraining via `tokenomics.retrain()`", "Fall back to rule-based estimation while retraining", "Lower the drift threshold if false positives are common", "Investigate provider pricing changes as a cause"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsColdStart", "TokenomicsOverfitting", "TokenomicsPredictionFailed"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_MODEL_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(287),
        long_code: "HHS_E_TOKENOMICS_MODEL_ERROR",
        short_code: "E0287",
        title: "ML Cost Model Internal Failure",
        short_description: "The ML model layer raised an internal error during training, inference, or evaluation.",
        long_description: "This wraps generic failures in the ML model layer that aren't covered by more specific variants — matrix shape mismatches, bad hyperparameters, numeric instability.

The wrapped message is the primary diagnosis. Most model errors are recoverable by retrying with cleaned data or by falling back to rule-based estimation while the model is restored.

If you see this consistently, the model state on disk may be corrupt; deleting it forces a fresh cold start.",
        hints: &["Inspect the wrapped error for the failed operation", "Fall back to rule-based estimation", "Delete corrupt model state to force cold start", "Validate hyperparameters in config"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsColdStart", "TokenomicsModelDrift", "TokenomicsPredictionFailed"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_OVERFITTING: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(288),
        long_code: "HHS_E_TOKENOMICS_OVERFITTING",
        short_code: "E0288",
        title: "Cost Model Is Overfitting Training Data",
        short_description: "Validation loss is rising while training loss falls, indicating the cost model is memorizing rather than generalizing.",
        long_description: "When the cost predictor's training loss continues to drop but validation loss starts climbing, the model is overfitting — it has learned noise specific to your training set rather than generalizable patterns. The runtime detects this and refuses to deploy the new model.

The automatic remedy is to keep the previous model and retrain later with more data, fewer parameters, or stronger regularization. If you see this surfacing to user code, the caller is bypassing the automatic protection.

Overfitting often appears when your traffic is too uniform — make sure the model has enough variety in its samples.",
        hints: &["Keep the previous model rather than deploying the bad one", "Strengthen regularization in tokenomics ML config", "Collect more varied training samples", "Reduce model capacity if overfitting persists"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsColdStart", "TokenomicsModelDrift", "TokenomicsModelError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_PREDICTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(289),
        long_code: "HHS_E_TOKENOMICS_PREDICTION_FAILED",
        short_code: "E0289",
        title: "Cost Prediction Could Not Be Computed",
        short_description: "A specific prediction call failed, separate from drift or cold-start conditions.",
        long_description: "This is the catch-all for prediction failures that aren't more specifically classified — invalid feature vector, model loaded but in a bad state, runtime panic in inference.

The wrapped message gives the specific cause. The recommended response is to fall back to rule-based cost estimation for the affected request and continue serving traffic.

If failures cluster around specific request shapes, that pattern is the diagnostic — check whether your feature extraction handles those inputs.",
        hints: &["Fall back to rule-based estimation for this request", "Inspect the wrapped error for the specific cause", "Check feature extraction for unusual request shapes", "Reload the model if it appears to be in a bad state"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsColdStart", "TokenomicsModelError", "TokenomicsModelDrift"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_REDIS_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(290),
        long_code: "HHS_E_TOKENOMICS_REDIS_ERROR",
        short_code: "E0290",
        title: "Tokenomics Redis Backend Failed",
        short_description: "The Redis backend used by tokenomics for shared state raised an error.",
        long_description: "When tokenomics is configured with a shared Redis backend (for cross-node budget enforcement), this error wraps Redis client failures: connection refused, command rejected, replication lag.

Verify Redis is reachable from all nodes, that the configured database exists, and that your client version is compatible. For transient errors, the runtime retries with backoff.

If Redis is down for an extended period, you lose cross-node budget coordination — local enforcement still applies but limits won't be shared.",
        hints: &["Verify Redis is reachable from all tokenomics nodes", "Check the configured Redis db and credentials", "Allow the client to retry transient failures", "Monitor Redis health independently of the runtime"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsDatabaseError", "TokenomicsStorageError", "TokenomicsCacheError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_REINFORCEMENT_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(291),
        long_code: "HHS_E_TOKENOMICS_REINFORCEMENT_ERROR",
        short_code: "E0291",
        title: "Reinforcement Learning Step Failed",
        short_description: "The reinforcement learning loop used to optimize routing or budget allocation failed to converge.",
        long_description: "Tokenomics includes an RL component that learns optimal provider routing and budget allocation policies from observed outcomes. This error fires when the RL update step fails — bad reward signal, divergent value function, or runtime failure inside the agent.

The runtime falls back to the previous policy while you investigate. The wrapped error names the failed step. Common causes are sparse rewards (not enough completed episodes) and incorrect reward shaping.

If this fires often, RL may not be appropriate for your traffic shape — consider disabling it and relying on static routing.",
        hints: &["Keep the previous RL policy on failure", "Check that reward signal is non-sparse", "Verify reward shaping makes sense for your goals", "Disable RL if your traffic doesn't suit it"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsFederatedError", "TokenomicsModelError", "TokenomicsPredictionFailed"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_SERIALIZATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(292),
        long_code: "HHS_E_TOKENOMICS_SERIALIZATION_ERROR",
        short_code: "E0292",
        title: "Tokenomics Value Serialization Failed",
        short_description: "Encoding or decoding tokenomics state failed at the codec layer.",
        long_description: "Tokenomics stores model parameters, budgets, and history in serialized form. This error means the codec rejected a value — schema drift after upgrade, or a non-serializable extension type.

Run pending migrations after upgrades, and version your custom feature extractors so old training data can be detected. The wrapped message identifies the offending value or field.

Quarantine corrupt records rather than wiping the entire dataset.",
        hints: &["Run pending tokenomics migrations after upgrades", "Version custom feature extractors", "Quarantine corrupt records, don't wipe history", "Inspect the wrapped error for the offending field"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsDatabaseError", "TokenomicsStorageError", "PersistenceSerialization"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_STORAGE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(293),
        long_code: "HHS_E_TOKENOMICS_STORAGE_ERROR",
        short_code: "E0293",
        title: "Tokenomics Storage Backend Failed",
        short_description: "The abstract storage layer used by tokenomics raised an error not specific to any one backend.",
        long_description: "Tokenomics talks to storage through an abstraction that can sit on top of files, sqlite, or redis. This error is the abstraction-level error raised when the underlying backend fails or when the abstraction itself rejects an operation.

The wrapped message indicates the underlying cause; treat it like any backend failure. If the abstraction layer itself is misbehaving, log with full context and report a bug — abstraction-level errors should be rare in production.

For multi-backend deployments, verify all configured backends are healthy.",
        hints: &["Inspect the wrapped underlying error", "Verify all configured storage backends are healthy", "Report abstraction-level bugs with full context", "Use a single backend until you need multi-backend"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsDatabaseError", "TokenomicsRedisError", "TokenomicsIoError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_UNKNOWN: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(294),
        long_code: "HHS_E_TOKENOMICS_UNKNOWN",
        short_code: "E0294",
        title: "Unclassified Tokenomics Error",
        short_description: "An error was raised by the tokenomics layer that doesn't fit any specific category.",
        long_description: "This is the catch-all variant for tokenomics failures that don't match a more specific error code. Seeing this in production is unusual and usually indicates a bug — most legitimate failures should be raised under a more specific variant.

Log the wrapped message with full context and file an issue so the error can be reclassified into a specific code in a future release. As a workaround, treat it as a transient failure and retry once.

If the same `Unknown` error keeps recurring, that pattern itself is the bug to report.",
        hints: &["Log the wrapped message with full context", "File an issue so it can be reclassified", "Retry once as a workaround", "Report recurring patterns to the maintainers"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsModelError", "TokenomicsConfigError", "TokenomicsStorageError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

use super::*;
use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const COST_BUDGET_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(49),
        long_code: "HHS_E_COST_BUDGET_EXCEEDED",
        short_code: "E0049",
        title: "LLM Spend Exceeded Configured Budget",
        short_description: "An LLM call was blocked because executing it would push spending past the configured cost budget.",
        long_description: "HudHudScript tracks LLM spend per call using a pricing table and refuses to dispatch a request that would exceed the budget you set. The error reports both the current spend and the configured limit so you can decide how to proceed.

Raise the budget, free headroom by trimming context, or route the call to a cheaper model. If the budget is enforced per-time-window, also wait for the next window to open.

This is a hard guard, not advisory — production agents will fail closed rather than rack up unexpected charges.",
        hints: &["Increase `budget.cost_limit` if the spend is legitimate", "Switch to a cheaper model for non-critical calls", "Trim context length to lower per-call cost", "Use `cost.estimate(req)` before dispatch to avoid surprises"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsInsufficientBudget", "ProviderBudgetExceeded", "ProviderDailyBudgetExceeded"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const COST_UNKNOWN_MODEL: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(50),
        long_code: "HHS_E_COST_UNKNOWN_MODEL",
        short_code: "E0050",
        title: "No Pricing Entry For Requested Model",
        short_description: "The cost engine has no pricing data for the model you tried to call.",
        long_description: "To compute spend, the tokenomics layer looks up the per-token price of the requested model. If the model name is missing from the pricing table, this error fires and the call is blocked rather than silently logging zero cost.

Either add the model to your pricing config (`pricing.add(model, input_price, output_price)`) or correct the model name if you typed it wrong. New models from upstream providers must be registered before they can be used with budget enforcement.

For self-hosted models, set the price to zero explicitly so the cost engine knows it's intentional.",
        hints: &["Add the model to `pricing.toml` with input/output rates", "Double-check spelling against the provider's catalog", "Set self-hosted models to zero cost explicitly", "Subscribe to upstream pricing updates"],
        example_bad: None,
        example_good: None,
        see_also: &["CostUnknownProvider", "ProviderNotFound", "TokenomicsConfigError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const COST_UNKNOWN_PROVIDER: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(51),
        long_code: "HHS_E_COST_UNKNOWN_PROVIDER",
        short_code: "E0051",
        title: "No Pricing Entry For Requested Provider",
        short_description: "The cost engine has no pricing table for the provider you tried to call.",
        long_description: "Pricing tables are organised per provider (OpenAI, Anthropic, Ollama, etc.). When a call is made under a provider name that has no table loaded, the cost engine refuses to estimate spend and raises this error.

Add a pricing entry for the provider, or load a community-maintained pricing bundle. If the provider is intentionally free (such as a local Ollama deployment), register it with zero rates to satisfy the cost guard.

If the provider name is a typo, the fix is to correct the call site rather than register a fake table.",
        hints: &["Register the provider in `pricing.toml`", "Verify the provider name matches the dispatcher", "Use zero-rate entries for local models", "Load `pricing-community.toml` for ready-made tables"],
        example_bad: None,
        example_good: None,
        see_also: &["CostUnknownModel", "ProviderNotConfigured"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const RATE_LIMIT_RPM_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(207),
        long_code: "HHS_E_RATE_LIMIT_RPM_EXCEEDED",
        short_code: "E0207",
        title: "Requests-Per-Minute Limit Reached",
        short_description: "Provider RPM rate limit was hit; further requests are blocked until the window rolls.",
        long_description: "Each provider has a configured requests-per-minute cap. The local rate limiter tracks the rolling window and blocks dispatch when the cap is reached, rather than letting the upstream return its own 429.

Wait for the window to roll, slow your call rate, request a higher quota from the provider, or fail over to another provider via the fallback chain. Watch for thundering-herd patterns where many clients hit the limit at the same instant.

This is a local guard — it does not include retry-after handling for upstream 429s, which is a separate code path.",
        hints: &["Slow request rate to stay under the configured RPM", "Raise the provider RPM quota if available", "Fall over to a secondary provider when throttled", "Add jitter to avoid thundering-herd retries"],
        example_bad: None,
        example_good: None,
        see_also: &["RateLimitTpmExceeded", "ProviderApiError", "FallbackAllProvidersExhausted"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const RATE_LIMIT_TPM_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(208),
        long_code: "HHS_E_RATE_LIMIT_TPM_EXCEEDED",
        short_code: "E0208",
        title: "Tokens-Per-Minute Limit Reached",
        short_description: "Provider TPM rate limit was hit; further requests are blocked until the window rolls.",
        long_description: "Many LLM providers cap not just request count but token throughput per minute. This error means that token cap was reached on the local rolling window.

Reduce per-request token usage (shorter prompts, lower max_tokens), spread requests over time, or raise the TPM quota with the provider. If you have multiple keys, route across them with a load balancer to multiply effective throughput.

TPM caps are usually the binding constraint for high-volume workloads — RPM caps are easier to satisfy.",
        hints: &["Trim prompts to lower per-call token usage", "Spread requests across the minute, not in bursts", "Raise the provider TPM quota", "Load-balance across multiple keys"],
        example_bad: None,
        example_good: None,
        see_also: &["RateLimitRpmExceeded", "ProviderBudgetExceeded", "TokenomicsInsufficientBudget"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_BUDGET_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(277),
        long_code: "HHS_E_TOKENOMICS_BUDGET_NOT_FOUND",
        short_code: "E0277",
        title: "Named Budget Does Not Exist",
        short_description: "A budget operation referenced a budget name that has not been registered with the tokenomics layer.",
        long_description: "Tokenomics budgets are addressed by name (e.g. `\"team-a\"`, `\"prod-agents\"`). This error fires when you read or update a budget under a name the registry has never seen.

Register the budget first via `tokenomics.create_budget(name, ...)` or correct the name at the call site. If budgets are loaded from config, verify the config file actually defines the entry.

List registered budgets with `tokenomics.list_budgets()` when in doubt.",
        hints: &["Register the budget via `tokenomics.create_budget()`", "Verify the name matches your config", "Call `tokenomics.list_budgets()` to inspect", "Use stable names; avoid generated suffixes"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsInvalidBudget", "TokenomicsConfigError", "CostBudgetExceeded"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_CACHE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(278),
        long_code: "HHS_E_TOKENOMICS_CACHE_ERROR",
        short_code: "E0278",
        title: "Tokenomics Cache Layer Failure",
        short_description: "The tokenomics in-process cache for budget and prediction state raised an error.",
        long_description: "Tokenomics maintains a small in-process cache to avoid hitting the database on every call. This error wraps a failure inside that cache — eviction of pinned entries, codec mismatch, or capacity overflow.

The wrapped message identifies the cause. Most cache errors are recoverable by clearing and refetching from the underlying store; the runtime will do so automatically unless the failure is structural.

If this error appears repeatedly, raise the cache size or check whether the cache backend is healthy.",
        hints: &["Allow the runtime to refetch from underlying storage", "Raise tokenomics cache capacity if pressure is high", "Inspect the wrapped error for the cause", "Check the cache backend health independently"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsStorageError", "TokenomicsRedisError", "CacheQuotaExceeded"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_COLD_START: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(279),
        long_code: "HHS_E_TOKENOMICS_COLD_START",
        short_code: "E0279",
        title: "ML Cost Predictor Has No Training Data",
        short_description: "The cost prediction model cannot run yet because it has too few historical samples.",
        long_description: "HudHudScript predicts LLM call costs using a small ML model trained on your own usage history. Until that history exists (typically a few hundred calls), the model is in a cold-start state and cannot make reliable predictions.

This error is informational: the runtime should fall back to rule-based estimation (token count times unit price) until enough data accumulates. If you see this raised instead of handled, the caller forgot to wrap the prediction in the cold-start fallback.

Let the system collect data for a day or two of normal traffic before relying on ML predictions.",
        hints: &["Use rule-based estimation while history is small", "Wait for several hundred calls to accumulate", "Wrap predict calls in `predict_or_fallback()`", "Don't disable cold-start handling in production"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsModelError", "TokenomicsPredictionFailed", "TokenomicsModelDrift"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_CONFIG_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(280),
        long_code: "HHS_E_TOKENOMICS_CONFIG_ERROR",
        short_code: "E0280",
        title: "Tokenomics Configuration Is Invalid",
        short_description: "The tokenomics layer was loaded with a configuration that failed validation.",
        long_description: "Tokenomics config covers budgets, pricing, ML model parameters, and storage backend selection. This error fires when one of those fields is missing, malformed, or out of range.

The wrapped message names the offending field. Fix the config and reload — most fields can be hot-reloaded without restarting the process.

Keep tokenomics config under version control alongside your application config so changes are reviewable.",
        hints: &["Read the wrapped message for the offending field", "Use `tokenomics.validate_config()` at startup", "Hot-reload after fixing where supported", "Version-control your tokenomics config"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsBudgetNotFound", "TokenomicsInvalidBudget", "ProviderInvalidConfig"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_DATABASE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(281),
        long_code: "HHS_E_TOKENOMICS_DATABASE_ERROR",
        short_code: "E0281",
        title: "Tokenomics Database Backend Failed",
        short_description: "The persistent database backend used by tokenomics raised an error.",
        long_description: "Tokenomics persists budgets, usage history, and model state to a database (SQLite by default, optionally Postgres). This error wraps the database driver's own error — connection failure, schema mismatch, constraint violation, lock timeout.

The wrapped message is your primary diagnosis. Run database migrations after upgrades, ensure the connection pool is sized for your load, and watch for schema drift if you share the database with other services.

Do not catch and ignore — losing tokenomics writes means losing budget enforcement.",
        hints: &["Run pending tokenomics migrations after upgrades", "Inspect the wrapped database error", "Size the connection pool for your concurrency", "Don't share the schema with unrelated services"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsStorageError", "TokenomicsRedisError", "TokenomicsIoError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_FEDERATED_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(282),
        long_code: "HHS_E_TOKENOMICS_FEDERATED_ERROR",
        short_code: "E0282",
        title: "Federated Learning Sync Failed",
        short_description: "Federated learning aggregation across nodes failed to synchronize model parameters.",
        long_description: "Tokenomics can run as a federated learning system: multiple nodes train local models on their own usage and periodically aggregate parameters. This error fires when the sync round fails — network partition, version mismatch between nodes, or invalid gradient updates from one peer.

The wrapped message indicates which sync phase failed. Each node continues using its local model until the next sync round; federated learning is designed to tolerate occasional sync failures.

If you see persistent sync errors, check that all nodes run compatible runtime versions and that the aggregation server is reachable.",
        hints: &["Verify network reachability between nodes", "Confirm all nodes run compatible runtime versions", "Allow nodes to keep local models between syncs", "Inspect the wrapped error for the failed phase"],
        example_bad: None,
        example_good: None,
        see_also: &["TokenomicsModelError", "TokenomicsModelDrift", "TokenomicsReinforcementError"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

pub const TOKENOMICS_INSUFFICIENT_BUDGET: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(283),
        long_code: "HHS_E_TOKENOMICS_INSUFFICIENT_BUDGET",
        short_code: "E0283",
        title: "Not Enough Budget Remaining For Operation",
        short_description: "An operation was blocked because the remaining budget is below the requested amount.",
        long_description: "This error fires before a charge is recorded: the requested cost exceeds the remaining headroom in the named budget. The error reports both the needed amount and the available amount.

Raise the budget, top up the account, or shrink the request. For agents that should fail soft, catch this and substitute a cheaper path (smaller model, cached response).

This is distinct from `CostBudgetExceeded` (which fires after a charge is attempted) — this one is the pre-flight check.",
        hints: &["Top up or raise the budget for the needed amount", "Substitute a cheaper model on this code path", "Use cached responses where possible", "Distinguish from post-charge `CostBudgetExceeded`"],
        example_bad: None,
        example_good: None,
        see_also: &["CostBudgetExceeded", "ProviderBudgetExceeded", "TokenomicsBudgetNotFound"],
        since_version: "0.4.5",
        category: ErrorCategory::Tokenomics,
    };

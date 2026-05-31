use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RATE_LIMIT_RPM_EXCEEDED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(207),
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
        category: ExceptionCategory::Tokenomics,
    };

pub const RATE_LIMIT_TPM_EXCEEDED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(208),
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
        category: ExceptionCategory::Tokenomics,
    };

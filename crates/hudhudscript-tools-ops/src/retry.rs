//! Tool Call Retry and Fallback Logic (Issue #119)
//!
//! Provides `RetryPolicy` for configuring retry behaviour and `ToolCallExecutor`
//! for executing tool calls with automatic retries and an optional fallback tool.
//!
//! Delay computation is delegated to `hudhudscript_utils::RetryConfig` (Issue #692).

use std::time::Duration;
use tracing::{debug, warn};

use hudhudscript_tools_schema::registry::{RegistryError, ToolRegistry};
use hudhudscript_utils::RetryConfig;

/// Policy controlling retry and fallback behaviour for tool calls.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 means no retries).
    pub max_retries: u32,
    /// Initial back-off delay between attempts in milliseconds.
    pub backoff_ms: u64,
    /// Optional fallback tool name to call when all retries are exhausted.
    pub fallback_tool: Option<String>,
    /// Whether to use exponential backoff (doubles delay each attempt).
    pub exponential: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 100,
            fallback_tool: None,
            exponential: true,
        }
    }
}

impl RetryPolicy {
    /// Create a policy with no retries and no fallback.
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            backoff_ms: 0,
            fallback_tool: None,
            exponential: false,
        }
    }

    /// Create a policy that retries up to `max_retries` times with a fixed delay.
    pub fn fixed(max_retries: u32, backoff_ms: u64) -> Self {
        Self {
            max_retries,
            backoff_ms,
            fallback_tool: None,
            exponential: false,
        }
    }

    /// Attach a fallback tool that is called if all retries fail.
    pub fn with_fallback(mut self, fallback_tool: impl Into<String>) -> Self {
        self.fallback_tool = Some(fallback_tool.into());
        self
    }

    /// Compute the delay before attempt `attempt` (0-indexed).
    ///
    /// Delegates to [`RetryConfig::delay_for_attempt`] from `hudhudscript-utils`
    /// for the exponential path, ensuring a single backoff algorithm across the
    /// project (Issue #692).  The attempt index is capped at 10 so the maximum
    /// factor is 2^10 = 1024, matching the original behaviour.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        if self.backoff_ms == 0 {
            return Duration::ZERO;
        }
        if self.exponential {
            // Cap at attempt 10 (factor 1024) to avoid overflow, matching old
            // bit-shift behaviour: `1u64 << attempt.min(10)`.
            let capped_attempt = attempt.min(10);
            let config = RetryConfig {
                max_retries: self.max_retries,
                base_delay: Duration::from_millis(self.backoff_ms),
                // Set max_delay high enough that the cap is driven by the
                // attempt clamp above, not by the delay ceiling.
                max_delay: Duration::from_secs(u64::MAX / 1_000),
                multiplier: 2.0,
                // Disable jitter to preserve deterministic behaviour expected by
                // existing callers of RetryPolicy.
                jitter: false,
            };
            config.delay_for_attempt(capped_attempt)
        } else {
            Duration::from_millis(self.backoff_ms)
        }
    }
}

/// Outcome of a tool call execution attempt.
#[derive(Debug)]
pub enum ToolCallOutcome {
    /// The primary tool succeeded.
    Success(serde_json::Value),
    /// All primary retries failed; the fallback tool succeeded.
    FallbackSuccess {
        fallback_tool: String,
        result: serde_json::Value,
    },
    /// All retries failed and no fallback was available/succeeded.
    Failed {
        attempts: u32,
        last_error: RegistryError,
    },
}

/// Executes tool calls with retry and fallback support.
pub struct ToolCallExecutor<'a> {
    registry: &'a ToolRegistry,
    policy: RetryPolicy,
}

impl<'a> ToolCallExecutor<'a> {
    /// Create a new executor using the given registry and policy.
    pub fn new(registry: &'a ToolRegistry, policy: RetryPolicy) -> Self {
        Self { registry, policy }
    }

    /// Call `tool_name` with `arguments`, retrying according to the policy.
    ///
    /// Returns a [`ToolCallOutcome`] describing how execution completed.
    pub async fn call(&self, tool_name: &str, arguments: serde_json::Value) -> ToolCallOutcome {
        let mut last_error: Option<RegistryError> = None;
        let max_attempts = self.policy.max_retries + 1;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = self.policy.delay_for(attempt - 1);
                debug!(
                    tool = tool_name,
                    attempt,
                    delay_ms = delay.as_millis(),
                    "Retrying tool call"
                );
                if delay > Duration::ZERO {
                    tokio::time::sleep(delay).await;
                }
            }

            match self.registry.call_tool(tool_name, arguments.clone()).await {
                Ok(result) => {
                    debug!(tool = tool_name, attempt, "Tool call succeeded");
                    return ToolCallOutcome::Success(result);
                }
                Err(err) => {
                    warn!(
                        tool = tool_name,
                        attempt,
                        error = %err,
                        "Tool call failed"
                    );
                    last_error = Some(err);
                }
            }
        }

        // All primary attempts exhausted — try fallback if configured.
        if let Some(ref fallback) = self.policy.fallback_tool {
            warn!(
                primary_tool = tool_name,
                fallback_tool = fallback.as_str(),
                "All retries exhausted, attempting fallback tool"
            );
            match self.registry.call_tool(fallback, arguments).await {
                Ok(result) => {
                    return ToolCallOutcome::FallbackSuccess {
                        fallback_tool: fallback.clone(),
                        result,
                    };
                }
                Err(err) => {
                    warn!(fallback_tool = fallback.as_str(), error = %err, "Fallback tool also failed");
                }
            }
        }

        ToolCallOutcome::Failed {
            attempts: max_attempts,
            last_error: last_error.unwrap_or_else(|| {
                // Should be unreachable: if we ran any attempts and they all
                // failed, last_error must have been set. If we reach this,
                // there's a logic bug — give an explicit "no attempts" error.
                RegistryError::CallFailed(
                    "no retry attempts were made (logic bug — please report)".to_string(),
                )
            }),
        }
    }
}

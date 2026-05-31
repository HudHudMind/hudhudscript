use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const COST_BUDGET_EXCEEDED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(49),
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
        category: ExceptionCategory::Tokenomics,
    };

pub const COST_UNKNOWN_MODEL: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(50),
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
        category: ExceptionCategory::Tokenomics,
    };

pub const COST_UNKNOWN_PROVIDER: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(51),
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
        category: ExceptionCategory::Tokenomics,
    };

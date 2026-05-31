use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum TokenomicsProviderErrorCode {
    /// E0197 — LLM Provider API Returned An Error
    ProviderApiError = 197,
    /// E0198 — Per-Request Token Budget Exceeded
    ProviderBudgetExceeded = 198,
    /// E0199 — Daily Token Or Spend Budget Exceeded
    ProviderDailyBudgetExceeded = 199,
    /// E0200 — Provider Configuration Is Invalid
    ProviderInvalidConfig = 200,
    /// E0201 — Monthly Token Or Spend Budget Exceeded
    ProviderMonthlyBudgetExceeded = 201,
    /// E0202 — Network Failure Talking To Provider
    ProviderNetworkError = 202,
    /// E0203 — Provider Referenced But Not Configured
    ProviderNotConfigured = 203,
    /// E0204 — Provider Lookup Returned Nothing
    ProviderNotFound = 204,
    /// E0205 — Token Optimization Pass Failed
    ProviderOptimizationError = 205,
    /// E0206 — Provider Request Or Response Serialization Failed
    ProviderSerializationError = 206,
}

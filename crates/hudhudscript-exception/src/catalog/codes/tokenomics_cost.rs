use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum TokenomicsCostExceptionCode {
    /// E0049 — LLM Spend Exceeded Configured Budget
    CostBudgetExceeded = 49,
    /// E0050 — No Pricing Entry For Requested Model
    CostUnknownModel = 50,
    /// E0051 — No Pricing Entry For Requested Provider
    CostUnknownProvider = 51,
}

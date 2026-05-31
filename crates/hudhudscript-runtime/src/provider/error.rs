//! Provider error types and catalog integration

/// Provider errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    NotFound(String),
    NotConfigured(String),
    ApiError(String),
    BudgetExceeded { limit: usize, requested: usize },
    DailyBudgetExceeded { limit: usize, current: usize },
    MonthlyBudgetExceeded { limit: usize, current: usize },
    InvalidConfig(String),
    NetworkError(String),
    SerializationError(String),
    OptimizationError(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ProviderError::NotFound(s) => write!(f, "Provider not found: {}", s),
            ProviderError::NotConfigured(s) => write!(f, "Provider not configured: {}", s),
            ProviderError::ApiError(s) => write!(f, "API error: {}", s),
            ProviderError::BudgetExceeded { limit, requested } => write!(
                f,
                "Budget exceeded: requested {} tokens, limit is {}",
                requested, limit
            ),
            ProviderError::DailyBudgetExceeded { limit, current } => write!(
                f,
                "Daily budget exceeded: current usage {}, limit is {}",
                current, limit
            ),
            ProviderError::MonthlyBudgetExceeded { limit, current } => write!(
                f,
                "Monthly budget exceeded: current usage {}, limit is {}",
                current, limit
            ),
            ProviderError::InvalidConfig(s) => write!(f, "Invalid configuration: {}", s),
            ProviderError::NetworkError(s) => write!(f, "Network error: {}", s),
            ProviderError::SerializationError(s) => write!(f, "Serialization error: {}", s),
            ProviderError::OptimizationError(s) => write!(f, "Token optimization failed: {}", s),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        // Note: this loses the inner error chain. To preserve cause chains,
        // ProviderError::NetworkError would need to wrap reqwest::Error directly
        // (instead of stringifying), but that's a breaking API change.
        // (Issue #844 — proper #[source] chaining is tracked separately)
        ProviderError::NetworkError(format!("HTTP: {}", err))
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        // Same caveat as above for source chains. Including line/col context
        // from serde_json::Error in the message.
        ProviderError::SerializationError(format!(
            "JSON parse error at line {} col {}: {}",
            err.line(),
            err.column(),
            err
        ))
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl ProviderError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ProviderError::ApiError(..) => hudhudscript_errors::ErrorCode::ProviderApiError,
            ProviderError::BudgetExceeded { .. } => {
                hudhudscript_errors::ErrorCode::ProviderBudgetExceeded
            }
            ProviderError::DailyBudgetExceeded { .. } => {
                hudhudscript_errors::ErrorCode::ProviderDailyBudgetExceeded
            }
            ProviderError::InvalidConfig(..) => {
                hudhudscript_errors::ErrorCode::ProviderInvalidConfig
            }
            ProviderError::MonthlyBudgetExceeded { .. } => {
                hudhudscript_errors::ErrorCode::ProviderMonthlyBudgetExceeded
            }
            ProviderError::NetworkError(..) => hudhudscript_errors::ErrorCode::ProviderNetworkError,
            ProviderError::NotConfigured(..) => {
                hudhudscript_errors::ErrorCode::ProviderNotConfigured
            }
            ProviderError::NotFound(..) => hudhudscript_errors::ErrorCode::ProviderNotFound,
            ProviderError::OptimizationError(..) => {
                hudhudscript_errors::ErrorCode::ProviderOptimizationError
            }
            ProviderError::SerializationError(..) => {
                hudhudscript_errors::ErrorCode::ProviderSerializationError
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<ProviderError> for hudhudscript_errors::Error {
    fn from(e: ProviderError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

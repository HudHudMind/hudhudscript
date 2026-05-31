/// Errors that can occur during cost tracking.
#[derive(Debug)]
pub enum CostError {
    UnknownProvider(String),
    UnknownModel(String),
    BudgetExceeded { spent: f64, limit: f64 },
}

impl std::fmt::Display for CostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CostError::UnknownProvider(s) => write!(f, "Unknown provider: {}", s),
            CostError::UnknownModel(s) => write!(f, "Unknown model: {}", s),
            CostError::BudgetExceeded { spent, limit } => {
                write!(f, "Budget exceeded: spent {:.6}, limit {:.6}", spent, limit)
            }
        }
    }
}

impl std::error::Error for CostError {}

impl CostError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CostError::BudgetExceeded { .. } => hudhudscript_errors::ErrorCode::CostBudgetExceeded,
            CostError::UnknownModel(..) => hudhudscript_errors::ErrorCode::CostUnknownModel,
            CostError::UnknownProvider(..) => hudhudscript_errors::ErrorCode::CostUnknownProvider,
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

impl From<CostError> for hudhudscript_errors::Error {
    fn from(e: CostError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

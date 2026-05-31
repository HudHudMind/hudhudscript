/// Council errors for the orchestration layer.
///
/// Note: there is a separate `CouncilError` in `hudhudscript-governance` that
/// covers governance-domain council errors (constitution lookup, role validation,
/// etc.). This type covers orchestration-time errors (no members, execution
/// timeout). They are intentionally separate because they live at different
/// abstraction layers — see Issue #825 / #849 for the rationale.
#[derive(Debug)]
pub enum CouncilError {
    NotFound(String),
    NoMembers,
    ExecutionFailed(String),
    Timeout,
}

impl std::fmt::Display for CouncilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CouncilError::NotFound(s) => write!(f, "Council not found: {}", s),
            CouncilError::NoMembers => write!(f, "No members in council"),
            CouncilError::ExecutionFailed(s) => write!(f, "Execution failed: {}", s),
            CouncilError::Timeout => write!(f, "Timeout"),
        }
    }
}

impl std::error::Error for CouncilError {}

impl CouncilError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CouncilError::ExecutionFailed(..) => {
                hudhudscript_errors::ErrorCode::CouncilExecutionFailed
            }
            CouncilError::NoMembers => hudhudscript_errors::ErrorCode::CouncilNoMembers,
            CouncilError::NotFound(..) => hudhudscript_errors::ErrorCode::CouncilNotFound,
            CouncilError::Timeout => hudhudscript_errors::ErrorCode::CouncilTimeout,
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

impl From<CouncilError> for hudhudscript_errors::Error {
    fn from(e: CouncilError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

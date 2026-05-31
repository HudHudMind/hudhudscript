use hudhudscript_tools_schema::schema::ValidationError;

/// Error returned by custom tools
#[derive(Debug)]
pub enum ToolError {
    InvalidArguments(String),
    ExecutionFailed(String),
    Validation(ValidationError),
    SecurityViolation(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ToolError::InvalidArguments(s) => write!(f, "Invalid arguments: {}", s),
            ToolError::ExecutionFailed(s) => write!(f, "Execution failed: {}", s),
            ToolError::Validation(e) => write!(f, "Validation error: {}", e),
            ToolError::SecurityViolation(s) => write!(f, "Security violation: {}", s),
        }
    }
}

impl std::error::Error for ToolError {}

impl From<ValidationError> for ToolError {
    fn from(e: ValidationError) -> Self {
        ToolError::Validation(e)
    }
}

impl ToolError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ToolError::ExecutionFailed(..) => hudhudscript_errors::ErrorCode::ToolExecutionFailed,
            ToolError::InvalidArguments(..) => hudhudscript_errors::ErrorCode::ToolInvalidArguments,
            ToolError::SecurityViolation(..) => {
                hudhudscript_errors::ErrorCode::ToolSecurityViolation
            }
            ToolError::Validation(..) => hudhudscript_errors::ErrorCode::ToolValidation,
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

impl From<ToolError> for hudhudscript_errors::Error {
    fn from(e: ToolError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

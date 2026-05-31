use std::fmt;

/// VCS Errors
#[derive(Debug)]
pub enum VcsError {
    BranchNotFound(String),
    BranchAlreadyExists(String),
    MergeConflict(String),
    InvalidOperation(String),
    ParseError(String),
}

impl fmt::Display for VcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            VcsError::BranchNotFound(s) => write!(f, "Branch not found: {}", s),
            VcsError::BranchAlreadyExists(s) => write!(f, "Branch already exists: {}", s),
            VcsError::MergeConflict(s) => write!(f, "Merge conflict: {}", s),
            VcsError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
            VcsError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for VcsError {}

impl VcsError {
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            VcsError::BranchAlreadyExists(..) => {
                hudhudscript_errors::ErrorCode::VcsBranchAlreadyExists
            }
            VcsError::BranchNotFound(..) => hudhudscript_errors::ErrorCode::VcsBranchNotFound,
            VcsError::InvalidOperation(..) => hudhudscript_errors::ErrorCode::VcsInvalidOperation,
            VcsError::MergeConflict(..) => hudhudscript_errors::ErrorCode::VcsMergeConflict,
            VcsError::ParseError(..) => hudhudscript_errors::ErrorCode::VcsParseError,
        }
    }

    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<VcsError> for hudhudscript_errors::Error {
    fn from(e: VcsError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

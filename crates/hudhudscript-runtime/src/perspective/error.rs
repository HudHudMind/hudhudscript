//! Errors produced by perspective-layer operations.

use crate::agent::AgentId;

/// Errors produced by perspective-layer operations.
#[derive(Debug)]
pub enum PerspectiveError {
    WriteAccessDenied { agent: AgentId, field: String },
    FieldHidden { field: String },
}

impl std::fmt::Display for PerspectiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            PerspectiveError::WriteAccessDenied { agent, field } => write!(
                f,
                "agent '{}' does not have write access to field '{}'",
                agent, field
            ),
            PerspectiveError::FieldHidden { field } => write!(
                f,
                "field '{}' is not visible from the current perspective",
                field
            ),
        }
    }
}

impl std::error::Error for PerspectiveError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl PerspectiveError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            PerspectiveError::FieldHidden { .. } => {
                hudhudscript_errors::ErrorCode::PerspectiveFieldHidden
            }
            PerspectiveError::WriteAccessDenied { .. } => {
                hudhudscript_errors::ErrorCode::PerspectiveWriteAccessDenied
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

impl From<PerspectiveError> for hudhudscript_errors::Error {
    fn from(e: PerspectiveError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

use crate::schema::ValidationError;
/// Registry error types
#[derive(Debug, Clone)]
pub enum RegistryError {
    ToolNotFound(String),
    ServerNotFound(String),
    DiscoveryFailed(String),
    CallFailed(String),
    DuplicateTool(String),
    ValidationFailed(ValidationError),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            RegistryError::ToolNotFound(s) => write!(f, "Tool not found: {}", s),
            RegistryError::ServerNotFound(s) => write!(f, "Server not found: {}", s),
            RegistryError::DiscoveryFailed(s) => write!(f, "Tool discovery failed: {}", s),
            RegistryError::CallFailed(s) => write!(f, "Tool call failed: {}", s),
            RegistryError::DuplicateTool(s) => write!(f, "Duplicate tool: {}", s),
            RegistryError::ValidationFailed(e) => write!(f, "Validation failed: {}", e),
        }
    }
}

impl std::error::Error for RegistryError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl RegistryError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            RegistryError::CallFailed(..) => hudhudscript_errors::ErrorCode::RegistryCallFailed,
            RegistryError::DiscoveryFailed(..) => {
                hudhudscript_errors::ErrorCode::RegistryDiscoveryFailed
            }
            RegistryError::DuplicateTool(..) => {
                hudhudscript_errors::ErrorCode::RegistryDuplicateTool
            }
            RegistryError::ServerNotFound(..) => {
                hudhudscript_errors::ErrorCode::RegistryServerNotFound
            }
            RegistryError::ToolNotFound(..) => hudhudscript_errors::ErrorCode::RegistryToolNotFound,
            RegistryError::ValidationFailed(..) => {
                hudhudscript_errors::ErrorCode::RegistryValidationFailed
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

impl From<RegistryError> for hudhudscript_errors::Error {
    fn from(e: RegistryError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

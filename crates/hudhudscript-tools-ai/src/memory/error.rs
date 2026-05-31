/// Errors that can occur during memory operations.
#[derive(Debug)]
pub enum MemoryError {
    NotFound(String),
    Backend(String),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            MemoryError::NotFound(s) => write!(f, "Memory entry not found: {}", s),
            MemoryError::Backend(s) => write!(f, "Backend error: {}", s),
            MemoryError::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for MemoryError {}

impl From<serde_json::Error> for MemoryError {
    fn from(e: serde_json::Error) -> Self {
        MemoryError::Serialization(e)
    }
}

impl MemoryError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            MemoryError::Backend(..) => hudhudscript_errors::ErrorCode::MemoryBackend,
            MemoryError::NotFound(..) => hudhudscript_errors::ErrorCode::MemoryNotFound,
            MemoryError::Serialization(..) => hudhudscript_errors::ErrorCode::MemorySerialization,
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

impl From<MemoryError> for hudhudscript_errors::Error {
    fn from(e: MemoryError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

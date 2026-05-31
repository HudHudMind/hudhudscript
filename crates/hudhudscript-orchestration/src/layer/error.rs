use super::LayerId;

/// Layer errors
#[derive(Debug)]
pub enum LayerError {
    LayerAlreadyExists(LayerId),
    LayerNotFound(LayerId),
    DependencyNotFound(LayerId),
    ExecutionFailed(String),
    TimeoutExceeded(LayerId),
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            LayerError::LayerAlreadyExists(id) => write!(f, "Layer already exists: {}", id),
            LayerError::LayerNotFound(id) => write!(f, "Layer not found: {}", id),
            LayerError::DependencyNotFound(id) => write!(f, "Dependency not found: {}", id),
            LayerError::ExecutionFailed(s) => write!(f, "Execution failed: {}", s),
            LayerError::TimeoutExceeded(id) => write!(f, "Layer execution timed out: {}", id),
        }
    }
}

impl std::error::Error for LayerError {}

impl LayerError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            LayerError::DependencyNotFound(..) => {
                hudhudscript_errors::ErrorCode::LayerDependencyNotFound
            }
            LayerError::ExecutionFailed(..) => hudhudscript_errors::ErrorCode::LayerExecutionFailed,
            LayerError::LayerAlreadyExists(..) => {
                hudhudscript_errors::ErrorCode::LayerLayerAlreadyExists
            }
            LayerError::LayerNotFound(..) => hudhudscript_errors::ErrorCode::LayerLayerNotFound,
            LayerError::TimeoutExceeded(..) => hudhudscript_errors::ErrorCode::LayerTimeoutExceeded,
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

impl From<LayerError> for hudhudscript_errors::Error {
    fn from(e: LayerError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

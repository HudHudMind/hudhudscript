

/// Bridge error type
#[derive(Debug, Clone)]
pub enum BridgeError {
    InitFailed(String),
    RenderFailed(String),
    ConnectionLost(String),
    FrameworkError(String),
    /// The selected framework is not implemented in this build.
    Unsupported(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::InitFailed(msg) => write!(f, "Bridge init failed: {}", msg),
            BridgeError::RenderFailed(msg) => write!(f, "Render failed: {}", msg),
            BridgeError::ConnectionLost(msg) => write!(f, "Connection lost: {}", msg),
            BridgeError::FrameworkError(msg) => write!(f, "Framework error: {}", msg),
            BridgeError::Unsupported(msg) => write!(f, "Unsupported UI framework: {}", msg),
        }
    }
}

impl std::error::Error for BridgeError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl BridgeError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            BridgeError::ConnectionLost(..) => hudhudscript_errors::ErrorCode::BridgeConnectionLost,
            BridgeError::FrameworkError(..) => hudhudscript_errors::ErrorCode::BridgeFrameworkError,
            BridgeError::InitFailed(..) => hudhudscript_errors::ErrorCode::BridgeInitFailed,
            BridgeError::RenderFailed(..) => hudhudscript_errors::ErrorCode::BridgeRenderFailed,
            BridgeError::Unsupported(..) => hudhudscript_errors::ErrorCode::BridgeFrameworkError,
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

impl From<BridgeError> for hudhudscript_errors::Error {
    fn from(e: BridgeError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

use std::fmt;

/// Errors produced by the COML runtime.
#[derive(Debug)]
pub enum CyberneticsError {
    ActuationFailed { loop_name: String, reason: String },
    ObserverError { loop_name: String, reason: String },
    PolicyError { loop_name: String, reason: String },
}

impl fmt::Display for CyberneticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CyberneticsError::ActuationFailed { loop_name, reason } => {
                write!(f, "actuation failed in loop '{}': {}", loop_name, reason)
            }
            CyberneticsError::ObserverError { loop_name, reason } => {
                write!(f, "observer error in loop '{}': {}", loop_name, reason)
            }
            CyberneticsError::PolicyError { loop_name, reason } => {
                write!(f, "policy error in loop '{}': {}", loop_name, reason)
            }
        }
    }
}

impl std::error::Error for CyberneticsError {}

impl CyberneticsError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CyberneticsError::ActuationFailed { .. } => {
                hudhudscript_errors::ErrorCode::CyberneticsActuationFailed
            }
            CyberneticsError::ObserverError { .. } => {
                hudhudscript_errors::ErrorCode::CyberneticsObserverError
            }
            CyberneticsError::PolicyError { .. } => {
                hudhudscript_errors::ErrorCode::CyberneticsPolicyError
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

impl From<CyberneticsError> for hudhudscript_errors::Error {
    fn from(e: CyberneticsError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

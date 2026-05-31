use super::WorkflowId;

/// Orchestration errors
#[derive(Debug)]
pub enum OrchestrationError {
    LayerError(crate::layer::LayerError),
    NetworkError(crate::network::NetworkError),
    WorkflowAlreadyExists(String),
    WorkflowNotFound(WorkflowId),
    NetworkNotFound(crate::network::NetworkId),
    NetworkExecutionFailed(String),
    InvalidWorkflow(String),
    WorkflowTimedOut(WorkflowId),
}

impl std::fmt::Display for OrchestrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            OrchestrationError::LayerError(e) => write!(f, "Layer error: {}", e),
            OrchestrationError::NetworkError(e) => write!(f, "Network error: {}", e),
            OrchestrationError::WorkflowAlreadyExists(s) => {
                write!(f, "Workflow already exists: {}", s)
            }
            OrchestrationError::WorkflowNotFound(id) => {
                write!(f, "Workflow not found: {}", id)
            }
            OrchestrationError::NetworkNotFound(id) => {
                write!(f, "Network not found: {}", id)
            }
            OrchestrationError::NetworkExecutionFailed(s) => {
                write!(f, "Network execution failed: {}", s)
            }
            OrchestrationError::InvalidWorkflow(s) => {
                write!(f, "Invalid workflow: {}", s)
            }
            OrchestrationError::WorkflowTimedOut(id) => {
                write!(f, "Workflow timed out: {}", id)
            }
        }
    }
}

impl std::error::Error for OrchestrationError {}

impl From<crate::layer::LayerError> for OrchestrationError {
    fn from(e: crate::layer::LayerError) -> Self {
        OrchestrationError::LayerError(e)
    }
}

impl From<crate::network::NetworkError> for OrchestrationError {
    fn from(e: crate::network::NetworkError) -> Self {
        OrchestrationError::NetworkError(e)
    }
}

impl OrchestrationError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            OrchestrationError::InvalidWorkflow(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationInvalidWorkflow
            }
            OrchestrationError::LayerError(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationLayerError
            }
            OrchestrationError::NetworkError(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationNetworkError
            }
            OrchestrationError::NetworkExecutionFailed(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationNetworkExecutionFailed
            }
            OrchestrationError::NetworkNotFound(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationNetworkNotFound
            }
            OrchestrationError::WorkflowAlreadyExists(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationWorkflowAlreadyExists
            }
            OrchestrationError::WorkflowNotFound(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationWorkflowNotFound
            }
            OrchestrationError::WorkflowTimedOut(..) => {
                hudhudscript_errors::ErrorCode::OrchestrationWorkflowTimedOut
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

impl From<OrchestrationError> for hudhudscript_errors::Error {
    fn from(e: OrchestrationError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

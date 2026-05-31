//! Runtime errors and error catalog integration.

/// Runtime errors for the agent runtime layer.
///
/// Variants currently use `String` payloads for backward compat. New code should
/// use the typed constructors below which build well-formatted messages from
/// structured fields. (Issue #843, v0.4.47.9)
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Agent already exists: {0}")]
    AgentAlreadyExists(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Tool error: {0}")]
    ToolError(String),

    #[error("Resource error: {0}")]
    ResourceError(String),
}

impl RuntimeError {
    /// Build an `ExecutionFailed` error from agent + task + reason.
    pub fn execution_failed(
        agent: impl Into<String>,
        task: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ExecutionFailed(format!(
            "agent '{}' task '{}': {}",
            agent.into(),
            task.into(),
            reason.into()
        ))
    }

    /// Build a `StateError` from a state key and reason.
    pub fn state_error(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::StateError(format!("key '{}': {}", key.into(), reason.into()))
    }

    /// Build a `ToolError` from a tool name and reason.
    pub fn tool_error(tool: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ToolError(format!("tool '{}': {}", tool.into(), reason.into()))
    }

    /// Build a `ResourceError` from resource id and reason.
    pub fn resource_error(resource: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ResourceError(format!("resource '{}': {}", resource.into(), reason.into()))
    }

    /// Stable error code for documentation lookup.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::AgentAlreadyExists(_) => "E_AGENT_DUPLICATE",
            Self::AgentNotFound(_) => "E_AGENT_NOT_FOUND",
            Self::TaskNotFound(_) => "E_TASK_NOT_FOUND",
            Self::ExecutionFailed(_) => "E_EXECUTION",
            Self::StateError(_) => "E_STATE",
            Self::ToolError(_) => "E_TOOL",
            Self::ResourceError(_) => "E_RESOURCE",
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl RuntimeError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            RuntimeError::AgentAlreadyExists(..) => {
                hudhudscript_errors::ErrorCode::RuntimeAgentAlreadyExists
            }
            RuntimeError::AgentNotFound(..) => hudhudscript_errors::ErrorCode::RuntimeAgentNotFound,
            RuntimeError::ExecutionFailed(..) => {
                hudhudscript_errors::ErrorCode::RuntimeExecutionFailed
            }
            RuntimeError::ResourceError(..) => hudhudscript_errors::ErrorCode::RuntimeResourceError,
            RuntimeError::StateError(..) => hudhudscript_errors::ErrorCode::RuntimeStateError,
            RuntimeError::TaskNotFound(..) => hudhudscript_errors::ErrorCode::RuntimeTaskNotFound,
            RuntimeError::ToolError(..) => hudhudscript_errors::ErrorCode::RuntimeToolError,
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

impl From<RuntimeError> for hudhudscript_errors::Error {
    fn from(e: RuntimeError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

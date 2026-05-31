use tracing::warn;

use super::{ApprovalId, ApprovalRegistry};

/// Checks whether a named tool requires human approval.
///
/// The set of tools requiring approval is configurable at runtime.
pub struct ApprovalGate {
    /// Tool names that require human approval before execution.
    pub(crate) tools_requiring_approval: Vec<String>,
    pub(crate) registry: ApprovalRegistry,
}

impl ApprovalGate {
    pub fn new(registry: ApprovalRegistry) -> Self {
        Self {
            tools_requiring_approval: Vec::new(),
            registry,
        }
    }

    /// Register a tool as requiring approval.
    pub fn require_approval_for(&mut self, tool_name: impl Into<String>) {
        let name = tool_name.into();
        if !self.tools_requiring_approval.contains(&name) {
            self.tools_requiring_approval.push(name);
        }
    }

    /// Returns `true` if the tool must wait for human approval.
    pub fn needs_approval(&self, tool_name: &str) -> bool {
        self.tools_requiring_approval.iter().any(|t| t == tool_name)
    }

    /// Submit a request and return its ID (for callers to resolve).
    pub fn request_approval(&self, tool_name: &str, arguments: serde_json::Value) -> ApprovalId {
        if !self.needs_approval(tool_name) {
            warn!(
                tool = tool_name,
                "request_approval called for tool that does not require approval"
            );
        }
        self.registry.submit(tool_name, arguments)
    }

    /// Expose the underlying registry for resolving requests.
    pub fn registry(&self) -> &ApprovalRegistry {
        &self.registry
    }
}

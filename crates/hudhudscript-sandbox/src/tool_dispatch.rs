//! Secure Tool Dispatch (Issue #115)
//!
//! Provides a `ToolDispatcher` that checks sandbox permissions before
//! executing any tool, ensuring that tools cannot escape their security
//! boundaries.

use crate::{Sandbox, SandboxConfig, SandboxError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Permission set required by a tool before it may be dispatched
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolPermissions {
    /// File paths the tool needs read access to (empty = none required)
    pub file_reads: Vec<String>,
    /// File paths the tool needs write access to (empty = none required)
    pub file_writes: Vec<String>,
    /// (host, port) pairs the tool needs to contact (empty = none required)
    pub network_hosts: Vec<(String, u16)>,
    /// Shell commands the tool may spawn (empty = none required)
    pub spawn_commands: Vec<String>,
}

/// Result of dispatching a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    /// Tool name that was invoked
    pub tool_name: String,
    /// Whether the sandbox check passed and the tool was executed
    pub executed: bool,
    /// Output value (present when `executed == true`)
    pub output: Option<serde_json::Value>,
    /// Error description (present when `executed == false`)
    pub error: Option<String>,
}

impl DispatchResult {
    fn denied(tool_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            executed: false,
            output: None,
            error: Some(reason.into()),
        }
    }

    fn success(tool_name: impl Into<String>, output: serde_json::Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            executed: true,
            output: Some(output),
            error: None,
        }
    }
}

/// A callable tool handler registered with the dispatcher
pub trait ToolHandler: Send + Sync {
    /// Permissions this tool requires
    fn permissions(&self) -> ToolPermissions;

    /// Execute the tool with the given JSON arguments.
    /// The sandbox check has already been performed before this is called.
    fn call(&self, arguments: &serde_json::Value) -> Result<serde_json::Value, String>;
}

/// Dispatches tool calls after verifying sandbox permissions
pub struct ToolDispatcher {
    sandbox: Sandbox,
    handlers: HashMap<String, Box<dyn ToolHandler>>,
}

impl ToolDispatcher {
    /// Create a dispatcher backed by the given sandbox configuration
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            sandbox: Sandbox::new(config),
            handlers: HashMap::new(),
        }
    }

    /// Create a dispatcher with restrictive (production) sandbox defaults
    pub fn restrictive() -> Self {
        Self::new(SandboxConfig::default_restrictive())
    }

    /// Create a dispatcher with permissive (development) sandbox defaults
    pub fn permissive() -> Self {
        Self::new(SandboxConfig::default_permissive())
    }

    /// Register a tool handler under `name`
    pub fn register(&mut self, name: impl Into<String>, handler: Box<dyn ToolHandler>) {
        self.handlers.insert(name.into(), handler);
    }

    /// Dispatch a tool call.
    ///
    /// Steps:
    /// 1. Look up the handler.
    /// 2. Enumerate all permissions the tool declares.
    /// 3. Check each permission against the sandbox — deny on first failure.
    /// 4. Call the handler and return the result.
    pub fn dispatch(&self, tool_name: &str, arguments: &serde_json::Value) -> DispatchResult {
        let handler = match self.handlers.get(tool_name) {
            Some(h) => h,
            None => return DispatchResult::denied(tool_name, format!("unknown tool: {tool_name}")),
        };

        let perms = handler.permissions();

        // --- file read checks ---
        for path in &perms.file_reads {
            if let Err(e) = self.sandbox.check_file_access(path, false) {
                return DispatchResult::denied(tool_name, sandbox_deny_msg("file read", path, &e));
            }
        }

        // --- file write checks ---
        for path in &perms.file_writes {
            if let Err(e) = self.sandbox.check_file_access(path, true) {
                return DispatchResult::denied(tool_name, sandbox_deny_msg("file write", path, &e));
            }
        }

        // --- network checks ---
        for (host, port) in &perms.network_hosts {
            if let Err(e) = self.sandbox.check_network_access(host, *port) {
                return DispatchResult::denied(
                    tool_name,
                    sandbox_deny_msg("network", &format!("{host}:{port}"), &e),
                );
            }
        }

        // --- process checks ---
        for cmd in &perms.spawn_commands {
            if let Err(e) = self.sandbox.check_process_execution(cmd) {
                return DispatchResult::denied(
                    tool_name,
                    sandbox_deny_msg("spawn command", cmd, &e),
                );
            }
        }

        // All checks passed — execute
        match handler.call(arguments) {
            Ok(output) => DispatchResult::success(tool_name, output),
            Err(err) => DispatchResult {
                tool_name: tool_name.to_string(),
                executed: true,
                output: None,
                error: Some(err),
            },
        }
    }

    /// List all registered tool names
    pub fn registered_tools(&self) -> Vec<&str> {
        self.handlers.keys().map(String::as_str).collect()
    }
}

fn sandbox_deny_msg(kind: &str, resource: &str, err: &SandboxError) -> String {
    format!("sandbox denied {kind} access to '{resource}': {err}")
}

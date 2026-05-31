use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum SecurityErrorCode {
    /// E0186 — Agent is not registered with the permission system
    PermissionAgentNotRegistered = 186,
    /// E0187 — Agent is not permitted to perform this action
    PermissionDenied = 187,
    /// E0191 — Field hidden by current perspective
    PerspectiveFieldHidden = 191,
    /// E0192 — Write access denied by perspective
    PerspectiveWriteAccessDenied = 192,
    /// E0221 — Invalid role definition
    RoleInvalidRole = 221,
    /// E0222 — Permission not found on role
    RolePermissionNotFound = 222,
    /// E0223 — Role not found in registry
    RoleRoleNotFound = 223,
    /// E0255 — Filesystem Access Denied By Sandbox
    SandboxFileSystemDenied = 255,
    /// E0256 — Invalid Sandbox Configuration
    SandboxInvalidConfig = 256,
    /// E0257 — Network Access Denied By Sandbox
    SandboxNetworkDenied = 257,
    /// E0258 — Process Execution Denied By Sandbox
    SandboxProcessDenied = 258,
    /// E0259 — Sandbox Resource Limit Exceeded
    SandboxResourceLimitExceeded = 259,
    /// E0260 — Sandbox System Call Failed
    SandboxSystemCallFailed = 260,
}

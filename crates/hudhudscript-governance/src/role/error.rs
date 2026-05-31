//! Role errors — unified error catalog bridge.

/// Result type for role operations
pub type RoleResult<T> = Result<T, RoleError>;

/// Errors that can occur during role operations
#[derive(Debug, Clone, PartialEq)]
pub enum RoleError {
    /// Invalid role name
    InvalidRole(String),
    /// Role not found
    RoleNotFound(String),
    /// Permission not found
    PermissionNotFound(String),
}

impl RoleError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            RoleError::InvalidRole(..) => hudhudscript_errors::ErrorCode::RoleInvalidRole,
            RoleError::RoleNotFound(..) => hudhudscript_errors::ErrorCode::RoleRoleNotFound,
            RoleError::PermissionNotFound(..) => {
                hudhudscript_errors::ErrorCode::RolePermissionNotFound
            }
        }
    }
}

impl std::fmt::Display for RoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            RoleError::InvalidRole(role) => write!(f, "Invalid role: {}", role),
            RoleError::RoleNotFound(role) => write!(f, "Role not found: {}", role),
            RoleError::PermissionNotFound(perm) => write!(f, "Permission not found: {}", perm),
        }
    }
}

impl std::error::Error for RoleError {}

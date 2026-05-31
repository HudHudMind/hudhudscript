//! Error types for the governance system

use std::fmt;

/// Governance system errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceError {
    /// Constitution not found
    ConstitutionNotFound(ConstitutionNotFoundError),

    /// Cache ID collision
    CacheIdCollision(CacheIdCollisionError),

    /// Circular dependency detected
    CircularDependency(CircularDependencyError),

    /// Invalid role
    InvalidRole(InvalidRoleError),

    /// Format validation error
    FormatValidation(FormatValidationError),

    /// Agent not found
    AgentNotFound(String),

    /// Resource not found
    ResourceNotFound(String),

    /// Invalid configuration
    InvalidConfiguration(String),

    /// Serialization error
    SerializationError(String),
}

/// Constitution not found error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionNotFoundError {
    /// The requested constitution ID
    pub constitution_id: String,

    /// Additional context
    pub context: Option<String>,
}

impl ConstitutionNotFoundError {
    /// Create a new constitution not found error
    pub fn new(constitution_id: String) -> Self {
        Self {
            constitution_id,
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(constitution_id: String, context: String) -> Self {
        Self {
            constitution_id,
            context: Some(context),
        }
    }
}

/// Cache ID collision error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheIdCollisionError {
    /// The conflicting ID
    pub id: String,

    /// Type of the conflicting item
    pub item_type: String,

    /// Additional context
    pub context: Option<String>,
}

impl CacheIdCollisionError {
    /// Create a new cache ID collision error
    pub fn new(id: String, item_type: String) -> Self {
        Self {
            id,
            item_type,
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(id: String, item_type: String, context: String) -> Self {
        Self {
            id,
            item_type,
            context: Some(context),
        }
    }
}

/// Circular dependency error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircularDependencyError {
    /// The dependency chain that forms the cycle
    pub dependency_chain: Vec<String>,

    /// Additional context
    pub context: Option<String>,
}

impl CircularDependencyError {
    /// Create a new circular dependency error
    pub fn new(dependency_chain: Vec<String>) -> Self {
        Self {
            dependency_chain,
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(dependency_chain: Vec<String>, context: String) -> Self {
        Self {
            dependency_chain,
            context: Some(context),
        }
    }

    /// Get the cycle as a formatted string
    pub fn cycle_string(&self) -> String {
        self.dependency_chain.join(" -> ")
    }
}

/// Invalid role error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidRoleError {
    /// The invalid role name
    pub role_name: String,

    /// Valid roles (if available)
    pub valid_roles: Option<Vec<String>>,

    /// Additional context
    pub context: Option<String>,
}

impl InvalidRoleError {
    /// Create a new invalid role error
    pub fn new(role_name: String) -> Self {
        Self {
            role_name,
            valid_roles: None,
            context: None,
        }
    }

    /// Create with valid roles
    pub fn with_valid_roles(role_name: String, valid_roles: Vec<String>) -> Self {
        Self {
            role_name,
            valid_roles: Some(valid_roles),
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(role_name: String, context: String) -> Self {
        Self {
            role_name,
            valid_roles: None,
            context: Some(context),
        }
    }
}

/// Format validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatValidationError {
    /// The invalid value
    pub value: String,

    /// Expected format
    pub expected_format: String,

    /// Field name
    pub field_name: Option<String>,

    /// Additional context
    pub context: Option<String>,
}

impl FormatValidationError {
    /// Create a new format validation error
    pub fn new(value: String, expected_format: String) -> Self {
        Self {
            value,
            expected_format,
            field_name: None,
            context: None,
        }
    }

    /// Create with field name
    pub fn with_field(value: String, expected_format: String, field_name: String) -> Self {
        Self {
            value,
            expected_format,
            field_name: Some(field_name),
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(value: String, expected_format: String, context: String) -> Self {
        Self {
            value,
            expected_format,
            field_name: None,
            context: Some(context),
        }
    }
}

impl GovernanceError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            GovernanceError::ConstitutionNotFound(..) => {
                hudhudscript_errors::ErrorCode::GovernanceConstitutionNotFound
            }
            GovernanceError::CacheIdCollision(..) => {
                hudhudscript_errors::ErrorCode::GovernanceCacheIdCollision
            }
            GovernanceError::CircularDependency(..) => {
                hudhudscript_errors::ErrorCode::GovernanceCircularDependency
            }
            GovernanceError::InvalidRole(..) => {
                hudhudscript_errors::ErrorCode::GovernanceInvalidRole
            }
            GovernanceError::FormatValidation(..) => {
                hudhudscript_errors::ErrorCode::GovernanceFormatValidation
            }
            GovernanceError::AgentNotFound(..) => {
                hudhudscript_errors::ErrorCode::GovernanceAgentNotFound
            }
            GovernanceError::ResourceNotFound(..) => {
                hudhudscript_errors::ErrorCode::GovernanceResourceNotFound
            }
            GovernanceError::InvalidConfiguration(..) => {
                hudhudscript_errors::ErrorCode::GovernanceInvalidConfiguration
            }
            GovernanceError::SerializationError(..) => {
                hudhudscript_errors::ErrorCode::GovernanceSerializationError
            }
        }
    }
}

// Display implementations
impl fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            GovernanceError::ConstitutionNotFound(e) => write!(f, "{}", e),
            GovernanceError::CacheIdCollision(e) => write!(f, "{}", e),
            GovernanceError::CircularDependency(e) => write!(f, "{}", e),
            GovernanceError::InvalidRole(e) => write!(f, "{}", e),
            GovernanceError::FormatValidation(e) => write!(f, "{}", e),
            GovernanceError::AgentNotFound(id) => write!(f, "Agent not found: {}", id),
            GovernanceError::ResourceNotFound(id) => write!(f, "Resource not found: {}", id),
            GovernanceError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            GovernanceError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl fmt::Display for ConstitutionNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Constitution not found: {}", self.constitution_id)?;
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        Ok(())
    }
}

impl fmt::Display for CacheIdCollisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cache ID collision: {} already exists as {}",
            self.id, self.item_type
        )?;
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        Ok(())
    }
}

impl fmt::Display for CircularDependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circular dependency detected: {}", self.cycle_string())?;
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        Ok(())
    }
}

impl fmt::Display for InvalidRoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid role: {}", self.role_name)?;
        if let Some(valid_roles) = &self.valid_roles {
            write!(f, ". Valid roles: {}", valid_roles.join(", "))?;
        }
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        Ok(())
    }
}

impl fmt::Display for FormatValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(field_name) = &self.field_name {
            write!(f, "Format validation error for field '{}': ", field_name)?;
        } else {
            write!(f, "Format validation error: ")?;
        }
        write!(
            f,
            "'{}' does not match expected format '{}'",
            self.value, self.expected_format
        )?;
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        Ok(())
    }
}

// std::error::Error implementations
impl std::error::Error for GovernanceError {}
impl std::error::Error for ConstitutionNotFoundError {}
impl std::error::Error for CacheIdCollisionError {}
impl std::error::Error for CircularDependencyError {}
impl std::error::Error for InvalidRoleError {}
impl std::error::Error for FormatValidationError {}

// Conversion implementations
impl From<ConstitutionNotFoundError> for GovernanceError {
    fn from(e: ConstitutionNotFoundError) -> Self {
        GovernanceError::ConstitutionNotFound(e)
    }
}

impl From<CacheIdCollisionError> for GovernanceError {
    fn from(e: CacheIdCollisionError) -> Self {
        GovernanceError::CacheIdCollision(e)
    }
}

impl From<CircularDependencyError> for GovernanceError {
    fn from(e: CircularDependencyError) -> Self {
        GovernanceError::CircularDependency(e)
    }
}

impl From<InvalidRoleError> for GovernanceError {
    fn from(e: InvalidRoleError) -> Self {
        GovernanceError::InvalidRole(e)
    }
}

impl From<FormatValidationError> for GovernanceError {
    fn from(e: FormatValidationError) -> Self {
        GovernanceError::FormatValidation(e)
    }
}

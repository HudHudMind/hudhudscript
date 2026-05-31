use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum DatabaseExceptionCode {
    /// E0065 — Database connection could not be established
    DatabaseConnectionFailed = 65,
    /// E0066 — Database feature flag is disabled at build time
    DatabaseFeatureNotEnabled = 66,
    /// E0067 — Database call received invalid arguments
    DatabaseInvalidArguments = 67,
    /// E0068 — SQL query execution failed
    DatabaseQueryFailed = 68,
    /// E0069 — Database backend is not supported
    DatabaseUnsupportedBackend = 69,
}

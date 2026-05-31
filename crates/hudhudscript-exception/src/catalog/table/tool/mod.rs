use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

mod approval;
pub use approval::*;
mod database;
pub use database::*;
mod git;
pub use git::*;
mod http;
pub use http::*;
mod open;
pub use open::*;
mod tool;
pub use tool::*;
mod vcs;
pub use vcs::*;

pub static TABLE: &[ExceptionEntry] = &[
    APPROVAL_INVALID_TRANSITION,
    APPROVAL_NOT_FOUND,
    DATABASE_CONNECTION_FAILED,
    DATABASE_FEATURE_NOT_ENABLED,
    DATABASE_INVALID_ARGUMENTS,
    DATABASE_QUERY_FAILED,
    DATABASE_UNSUPPORTED_BACKEND,
    GIT_COMMAND_FAILED,
    GIT_GIT_NOT_FOUND,
    GIT_INVALID_ARGUMENTS,
    GIT_PARSE_ERROR,
    GIT_REPOSITORY_NOT_FOUND,
    GIT_SPAWN_FAILED,
    HTTP_TOOL_INVALID_URL,
    HTTP_TOOL_PARSE_ERROR,
    HTTP_TOOL_REQUEST_FAILED,
    HTTP_TOOL_TIMEOUT,
    OPEN_API_PARSE_ERROR,
    OPEN_API_REGISTRY_ERROR,
    TOOL_EXECUTION_FAILED,
    TOOL_INVALID_ARGUMENTS,
    TOOL_SECURITY_VIOLATION,
    TOOL_VALIDATION,
    VCS_BRANCH_ALREADY_EXISTS,
    VCS_BRANCH_NOT_FOUND,
    VCS_INVALID_OPERATION,
    VCS_PARSE_ERROR,
];

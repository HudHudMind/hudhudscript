use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ToolGitErrorCode {
    /// E0087 — git subprocess exited non-zero
    GitCommandFailed = 87,
    /// E0088 — git binary is not on PATH
    GitGitNotFound = 88,
    /// E0089 — git tool received invalid arguments
    GitInvalidArguments = 89,
    /// E0090 — Failed to parse git output
    GitParseError = 90,
    /// E0091 — No git repository at the given path
    GitRepositoryNotFound = 91,
    /// E0092 — Failed to spawn git subprocess
    GitSpawnFailed = 92,
}

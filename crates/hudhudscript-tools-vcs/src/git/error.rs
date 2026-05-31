use std::fmt;

#[derive(Debug)]
pub enum GitError {
    GitNotFound,
    CommandFailed { code: i32, stderr: String },
    SpawnFailed(String),
    InvalidArguments(String),
    RepositoryNotFound(String),
    ParseError(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            GitError::GitNotFound => write!(f, "git command not found on PATH"),
            GitError::CommandFailed { code, stderr } => {
                write!(f, "git command failed (exit {}): {}", code, stderr)
            }
            GitError::SpawnFailed(s) => write!(f, "git process could not be spawned: {}", s),
            GitError::InvalidArguments(s) => write!(f, "Invalid arguments: {}", s),
            GitError::RepositoryNotFound(s) => write!(f, "Repository not found at: {}", s),
            GitError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for GitError {}

impl GitError {
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            GitError::CommandFailed { .. } => hudhudscript_errors::ErrorCode::GitCommandFailed,
            GitError::GitNotFound => hudhudscript_errors::ErrorCode::GitGitNotFound,
            GitError::InvalidArguments(..) => hudhudscript_errors::ErrorCode::GitInvalidArguments,
            GitError::ParseError(..) => hudhudscript_errors::ErrorCode::GitParseError,
            GitError::RepositoryNotFound(..) => {
                hudhudscript_errors::ErrorCode::GitRepositoryNotFound
            }
            GitError::SpawnFailed(..) => hudhudscript_errors::ErrorCode::GitSpawnFailed,
        }
    }

    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<GitError> for hudhudscript_errors::Error {
    fn from(e: GitError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

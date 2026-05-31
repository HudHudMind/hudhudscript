/// Database tool errors
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    UnsupportedBackend(String),
    FeatureNotEnabled,
    InvalidArguments(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            DatabaseError::ConnectionFailed(s) => write!(f, "Connection failed: {}", s),
            DatabaseError::QueryFailed(s) => write!(f, "Query execution failed: {}", s),
            DatabaseError::UnsupportedBackend(s) => write!(f, "Unsupported backend: {}", s),
            DatabaseError::FeatureNotEnabled => write!(
                f,
                "Feature not enabled: compile with the `db` feature for full sqlx support"
            ),
            DatabaseError::InvalidArguments(s) => write!(f, "Invalid arguments: {}", s),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl DatabaseError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            DatabaseError::ConnectionFailed(..) => {
                hudhudscript_errors::ErrorCode::DatabaseConnectionFailed
            }
            DatabaseError::FeatureNotEnabled => {
                hudhudscript_errors::ErrorCode::DatabaseFeatureNotEnabled
            }
            DatabaseError::InvalidArguments(..) => {
                hudhudscript_errors::ErrorCode::DatabaseInvalidArguments
            }
            DatabaseError::QueryFailed(..) => hudhudscript_errors::ErrorCode::DatabaseQueryFailed,
            DatabaseError::UnsupportedBackend(..) => {
                hudhudscript_errors::ErrorCode::DatabaseUnsupportedBackend
            }
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

impl From<DatabaseError> for hudhudscript_errors::Error {
    fn from(e: DatabaseError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

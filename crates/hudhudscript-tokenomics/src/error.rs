//! Error types for tokenomics system

pub type Result<T> = std::result::Result<T, TokenomicsError>;

#[derive(Debug)]
pub enum TokenomicsError {
    InsufficientBudget {
        needed: u64,
        available: u64,
    },
    BudgetNotFound(String),
    InvalidBudget(u64),
    ModelError(String),
    PredictionFailed(String),
    StorageError(String),
    CacheError(String),
    ConfigError(String),
    FederatedError(String),
    ReinforcementError(String),
    ColdStart,
    ModelDrift,
    Overfitting,
    #[cfg(feature = "postgres-storage")]
    DatabaseError(sqlx::Error),
    #[cfg(feature = "redis-cache")]
    RedisError(redis::RedisError),
    SerializationError(serde_json::Error),
    IoError(std::io::Error),
    Unknown(String),
}

impl std::fmt::Display for TokenomicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            TokenomicsError::InsufficientBudget { needed, available } => write!(
                f,
                "Insufficient budget: need {}, have {}",
                needed, available
            ),
            TokenomicsError::BudgetNotFound(s) => write!(f, "Budget not found: {}", s),
            TokenomicsError::InvalidBudget(n) => write!(f, "Invalid budget amount: {}", n),
            TokenomicsError::ModelError(s) => write!(f, "ML model error: {}", s),
            TokenomicsError::PredictionFailed(s) => write!(f, "Prediction failed: {}", s),
            TokenomicsError::StorageError(s) => write!(f, "Storage error: {}", s),
            TokenomicsError::CacheError(s) => write!(f, "Cache error: {}", s),
            TokenomicsError::ConfigError(s) => write!(f, "Configuration error: {}", s),
            TokenomicsError::FederatedError(s) => write!(f, "Federated learning error: {}", s),
            TokenomicsError::ReinforcementError(s) => {
                write!(f, "Reinforcement learning error: {}", s)
            }
            TokenomicsError::ColdStart => write!(f, "Cold start: insufficient training data"),
            TokenomicsError::ModelDrift => {
                write!(f, "Model drift detected: accuracy below threshold")
            }
            TokenomicsError::Overfitting => {
                write!(f, "Overfitting detected: validation loss increasing")
            }
            #[cfg(feature = "postgres-storage")]
            TokenomicsError::DatabaseError(e) => write!(f, "Database error: {}", e),
            #[cfg(feature = "redis-cache")]
            TokenomicsError::RedisError(e) => write!(f, "Redis error: {}", e),
            TokenomicsError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            TokenomicsError::IoError(e) => write!(f, "IO error: {}", e),
            TokenomicsError::Unknown(s) => write!(f, "Unknown error: {}", s),
        }
    }
}

impl std::error::Error for TokenomicsError {}

#[cfg(feature = "postgres-storage")]
impl From<sqlx::Error> for TokenomicsError {
    fn from(e: sqlx::Error) -> Self {
        TokenomicsError::DatabaseError(e)
    }
}
#[cfg(feature = "redis-cache")]
impl From<redis::RedisError> for TokenomicsError {
    fn from(e: redis::RedisError) -> Self {
        TokenomicsError::RedisError(e)
    }
}
impl From<serde_json::Error> for TokenomicsError {
    fn from(e: serde_json::Error) -> Self {
        TokenomicsError::SerializationError(e)
    }
}
impl From<std::io::Error> for TokenomicsError {
    fn from(e: std::io::Error) -> Self {
        TokenomicsError::IoError(e)
    }
}

impl TokenomicsError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::CacheError(_) | Self::ColdStart | Self::PredictionFailed(_)
        )
    }

    pub fn should_fallback_to_rules(&self) -> bool {
        matches!(
            self,
            Self::ColdStart | Self::ModelDrift | Self::ModelError(_) | Self::PredictionFailed(_)
        )
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl TokenomicsError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            TokenomicsError::BudgetNotFound(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsBudgetNotFound
            }
            TokenomicsError::CacheError(..) => hudhudscript_errors::ErrorCode::TokenomicsCacheError,
            TokenomicsError::ColdStart => hudhudscript_errors::ErrorCode::TokenomicsColdStart,
            TokenomicsError::ConfigError(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsConfigError
            }
            #[cfg(feature = "postgres-storage")]
            TokenomicsError::DatabaseError(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsDatabaseError
            }
            TokenomicsError::FederatedError(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsFederatedError
            }
            TokenomicsError::InsufficientBudget { .. } => {
                hudhudscript_errors::ErrorCode::TokenomicsInsufficientBudget
            }
            TokenomicsError::InvalidBudget(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsInvalidBudget
            }
            TokenomicsError::IoError(..) => hudhudscript_errors::ErrorCode::TokenomicsIoError,
            TokenomicsError::ModelDrift => hudhudscript_errors::ErrorCode::TokenomicsModelDrift,
            TokenomicsError::ModelError(..) => hudhudscript_errors::ErrorCode::TokenomicsModelError,
            TokenomicsError::Overfitting => hudhudscript_errors::ErrorCode::TokenomicsOverfitting,
            TokenomicsError::PredictionFailed(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsPredictionFailed
            }
            #[cfg(feature = "redis-cache")]
            TokenomicsError::RedisError(..) => hudhudscript_errors::ErrorCode::TokenomicsRedisError,
            TokenomicsError::ReinforcementError(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsReinforcementError
            }
            TokenomicsError::SerializationError(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsSerializationError
            }
            TokenomicsError::StorageError(..) => {
                hudhudscript_errors::ErrorCode::TokenomicsStorageError
            }
            TokenomicsError::Unknown(..) => hudhudscript_errors::ErrorCode::TokenomicsUnknown,
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

impl From<TokenomicsError> for hudhudscript_errors::Error {
    fn from(e: TokenomicsError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

use crate::layer::LayerId;

use super::NetworkId;

/// Network errors
#[derive(Debug)]
pub enum NetworkError {
    NetworkAlreadyExists(String),
    NetworkNotFound(NetworkId),
    LayerNotFound(LayerId),
    CyclicDependency,
    LayerExecutionFailed(LayerId, String),
    InvalidTopology(String),
    TimeoutExceeded(NetworkId),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            NetworkError::NetworkAlreadyExists(s) => write!(f, "Network already exists: {}", s),
            NetworkError::NetworkNotFound(id) => write!(f, "Network not found: {}", id),
            NetworkError::LayerNotFound(id) => write!(f, "Layer not found: {}", id),
            NetworkError::CyclicDependency => write!(f, "Cyclic dependency detected"),
            NetworkError::LayerExecutionFailed(id, msg) => {
                write!(f, "Layer execution failed: {} - {}", id, msg)
            }
            NetworkError::InvalidTopology(s) => write!(f, "Invalid topology: {}", s),
            NetworkError::TimeoutExceeded(id) => write!(f, "Network execution timed out: {}", id),
        }
    }
}

impl std::error::Error for NetworkError {}

impl NetworkError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            NetworkError::CyclicDependency => {
                hudhudscript_errors::ErrorCode::NetworkCyclicDependency
            }
            NetworkError::InvalidTopology(..) => {
                hudhudscript_errors::ErrorCode::NetworkInvalidTopology
            }
            NetworkError::LayerExecutionFailed(..) => {
                hudhudscript_errors::ErrorCode::NetworkLayerExecutionFailed
            }
            NetworkError::LayerNotFound(..) => hudhudscript_errors::ErrorCode::NetworkLayerNotFound,
            NetworkError::NetworkAlreadyExists(..) => {
                hudhudscript_errors::ErrorCode::NetworkNetworkAlreadyExists
            }
            NetworkError::NetworkNotFound(..) => {
                hudhudscript_errors::ErrorCode::NetworkNetworkNotFound
            }
            NetworkError::TimeoutExceeded(..) => {
                hudhudscript_errors::ErrorCode::NetworkTimeoutExceeded
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

impl From<NetworkError> for hudhudscript_errors::Error {
    fn from(e: NetworkError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

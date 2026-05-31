//! Error types for the native FFI crate.

/// Convenience alias used throughout this crate.
pub type Result<T> = std::result::Result<T, NativeError>;

/// Errors that can occur when working with native libraries.
#[derive(Debug)]
pub enum NativeError {
    /// Failed to load a shared library from a specific path.
    LibraryLoad { path: String, reason: String },

    /// The library could not be found in any search path.
    LibraryNotFound {
        name: String,
        search_paths: Vec<String>,
    },

    /// Attempted to call a function on a library that has not been loaded.
    LibraryNotLoaded { name: String },

    /// A symbol (function) was not found in the shared library.
    SymbolNotFound {
        symbol: String,
        library: String,
        reason: String,
    },

    /// A registered function was not found.
    FunctionNotFound { function: String, library: String },

    /// Wrong number of arguments supplied to a native function.
    ArgumentCount {
        function: String,
        expected: usize,
        got: usize,
    },

    /// Too many arguments for the current trampoline (internal limitation).
    TooManyArguments { function: String, max: usize },

    /// A string value contained interior NUL bytes and cannot be passed as C `const char*`.
    InvalidString { value: String },

    /// An unsupported type was encountered during conversion.
    UnsupportedType { type_name: String, context: String },

    /// A build step (cmake / conan) failed.
    BuildError { message: String },
}

impl NativeError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            NativeError::LibraryLoad { .. } => hudhudscript_errors::ErrorCode::NativeLibraryLoad,
            NativeError::LibraryNotFound { .. } => {
                hudhudscript_errors::ErrorCode::NativeLibraryNotFound
            }
            NativeError::LibraryNotLoaded { .. } => {
                hudhudscript_errors::ErrorCode::NativeLibraryNotLoaded
            }
            NativeError::SymbolNotFound { .. } => {
                hudhudscript_errors::ErrorCode::NativeSymbolNotFound
            }
            NativeError::FunctionNotFound { .. } => {
                hudhudscript_errors::ErrorCode::NativeFunctionNotFound
            }
            NativeError::ArgumentCount { .. } => {
                hudhudscript_errors::ErrorCode::NativeArgumentCount
            }
            NativeError::TooManyArguments { .. } => {
                hudhudscript_errors::ErrorCode::NativeTooManyArguments
            }
            NativeError::InvalidString { .. } => {
                hudhudscript_errors::ErrorCode::NativeInvalidString
            }
            NativeError::UnsupportedType { .. } => {
                hudhudscript_errors::ErrorCode::NativeUnsupportedType
            }
            NativeError::BuildError { .. } => hudhudscript_errors::ErrorCode::NativeBuildError,
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

impl std::fmt::Display for NativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            NativeError::LibraryLoad { path, reason } => {
                write!(f, "failed to load native library at '{}': {}", path, reason)
            }
            NativeError::LibraryNotFound { name, search_paths } => write!(
                f,
                "native library '{}' not found in search paths: {:?}",
                name, search_paths
            ),
            NativeError::LibraryNotLoaded { name } => {
                write!(f, "native library '{}' is not loaded", name)
            }
            NativeError::SymbolNotFound {
                symbol,
                library,
                reason,
            } => write!(
                f,
                "symbol '{}' not found in library '{}': {}",
                symbol, library, reason
            ),
            NativeError::FunctionNotFound { function, library } => write!(
                f,
                "function '{}' is not registered on library '{}'",
                function, library
            ),
            NativeError::ArgumentCount {
                function,
                expected,
                got,
            } => write!(
                f,
                "function '{}' expects {} arguments, got {}",
                function, expected, got
            ),
            NativeError::TooManyArguments { function, max } => write!(
                f,
                "function '{}' has too many arguments (max {} supported)",
                function, max
            ),
            NativeError::InvalidString { value } => {
                write!(f, "string value contains interior NUL bytes: '{}'", value)
            }
            NativeError::UnsupportedType { type_name, context } => {
                write!(f, "unsupported native type '{}' in {}", type_name, context)
            }
            NativeError::BuildError { message } => write!(f, "native build error: {}", message),
        }
    }
}

impl std::error::Error for NativeError {}

impl From<NativeError> for hudhudscript_errors::Error {
    fn from(e: NativeError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

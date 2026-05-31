pub type Result<T> = std::result::Result<T, PackageError>;

#[derive(Debug)]
pub enum PackageError {
    Io(std::io::Error),
    Network(reqwest::Error),
    Serialization(serde_json::Error),
    Toml(toml::de::Error),
    PackageNotFound(String),
    VersionNotFound(String),
    ChecksumMismatch(String),
    DependencyResolution(String),
    InvalidPackageName(String),
    InvalidVersion(String),
    SecurityVulnerability(String),
    BuildFailed(String),
    EntryPointNotFound(String),
    TomlSerialize(toml::ser::Error),
    Other(anyhow::Error),
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            PackageError::Io(e) => write!(f, "IO error: {}", e),
            PackageError::Network(e) => write!(f, "Network error: {}", e),
            PackageError::Serialization(e) => write!(f, "Serialization error: {}", e),
            PackageError::Toml(e) => write!(f, "TOML error: {}", e),
            PackageError::PackageNotFound(s) => write!(f, "Package not found: {}", s),
            PackageError::VersionNotFound(s) => write!(f, "Version not found: {}", s),
            PackageError::ChecksumMismatch(s) => write!(f, "Checksum mismatch for package: {}", s),
            PackageError::DependencyResolution(s) => {
                write!(f, "Dependency resolution failed: {}", s)
            }
            PackageError::InvalidPackageName(s) => write!(f, "Invalid package name: {}", s),
            PackageError::InvalidVersion(s) => write!(f, "Invalid version: {}", s),
            PackageError::SecurityVulnerability(s) => {
                write!(f, "Security vulnerability detected: {}", s)
            }
            PackageError::BuildFailed(s) => write!(f, "Build failed: {}", s),
            PackageError::EntryPointNotFound(s) => write!(f, "Entry point not found: {}", s),
            PackageError::TomlSerialize(e) => write!(f, "TOML serialization error: {}", e),
            PackageError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for PackageError {}

impl From<std::io::Error> for PackageError {
    fn from(e: std::io::Error) -> Self {
        PackageError::Io(e)
    }
}
impl From<reqwest::Error> for PackageError {
    fn from(e: reqwest::Error) -> Self {
        PackageError::Network(e)
    }
}
impl From<serde_json::Error> for PackageError {
    fn from(e: serde_json::Error) -> Self {
        PackageError::Serialization(e)
    }
}
impl From<toml::de::Error> for PackageError {
    fn from(e: toml::de::Error) -> Self {
        PackageError::Toml(e)
    }
}
impl From<toml::ser::Error> for PackageError {
    fn from(e: toml::ser::Error) -> Self {
        PackageError::TomlSerialize(e)
    }
}
impl From<anyhow::Error> for PackageError {
    fn from(e: anyhow::Error) -> Self {
        PackageError::Other(e)
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl PackageError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            PackageError::BuildFailed(..) => hudhudscript_errors::ErrorCode::PackageBuildFailed,
            PackageError::ChecksumMismatch(..) => {
                hudhudscript_errors::ErrorCode::PackageChecksumMismatch
            }
            PackageError::DependencyResolution(..) => {
                hudhudscript_errors::ErrorCode::PackageDependencyResolution
            }
            PackageError::EntryPointNotFound(..) => {
                hudhudscript_errors::ErrorCode::PackageEntryPointNotFound
            }
            PackageError::InvalidPackageName(..) => {
                hudhudscript_errors::ErrorCode::PackageInvalidPackageName
            }
            PackageError::InvalidVersion(..) => {
                hudhudscript_errors::ErrorCode::PackageInvalidVersion
            }
            PackageError::Io(..) => hudhudscript_errors::ErrorCode::PackageIo,
            PackageError::Network(..) => hudhudscript_errors::ErrorCode::PackageNetwork,
            PackageError::Other(..) => hudhudscript_errors::ErrorCode::PackageOther,
            PackageError::PackageNotFound(..) => {
                hudhudscript_errors::ErrorCode::PackagePackageNotFound
            }
            PackageError::SecurityVulnerability(..) => {
                hudhudscript_errors::ErrorCode::PackageSecurityVulnerability
            }
            PackageError::Serialization(..) => hudhudscript_errors::ErrorCode::PackageSerialization,
            PackageError::Toml(..) => hudhudscript_errors::ErrorCode::PackageToml,
            PackageError::TomlSerialize(..) => hudhudscript_errors::ErrorCode::PackageTomlSerialize,
            PackageError::VersionNotFound(..) => {
                hudhudscript_errors::ErrorCode::PackageVersionNotFound
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

impl From<PackageError> for hudhudscript_errors::Error {
    fn from(e: PackageError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageCoreErrorCode {
    /// E0167 — Package build step failed
    PackageBuildFailed = 167,
    /// E0168 — Downloaded package failed checksum verification
    PackageChecksumMismatch = 168,
    /// E0169 — Dependency resolver could not satisfy constraints
    PackageDependencyResolution = 169,
    /// E0170 — Package entry point file missing
    PackageEntryPointNotFound = 170,
    /// E0171 — Invalid package name
    PackageInvalidPackageName = 171,
    /// E0172 — Invalid semver version string
    PackageInvalidVersion = 172,
    /// E0173 — I/O error in package operation
    PackageIo = 173,
    /// E0174 — Package manager network error
    PackageNetwork = 174,
    /// E0175 — Unspecified package manager error
    PackageOther = 175,
    /// E0176 — Package not found in registry
    PackagePackageNotFound = 176,
    /// E0177 — Package has known security vulnerability
    PackageSecurityVulnerability = 177,
    /// E0178 — Package data serialization error
    PackageSerialization = 178,
    /// E0179 — Failed to parse TOML
    PackageToml = 179,
    /// E0180 — Failed to write TOML
    PackageTomlSerialize = 180,
    /// E0181 — No matching version in registry
    PackageVersionNotFound = 181,
}

//! Native dependency configuration.
//!
//! Defines the `[native-dependencies]` section format for HudHudScript project files,
//! allowing users to declare C/C++ libraries their scripts depend on.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level native configuration, typically parsed from a project manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativeConfig {
    /// Map of dependency name to its configuration.
    #[serde(default, rename = "native-dependencies")]
    pub native_dependencies: HashMap<String, NativeDependency>,
}

/// A single native (C/C++) dependency declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeDependency {
    /// Human-readable name (defaults to the map key).
    #[serde(default)]
    pub name: String,

    /// Local filesystem path to a CMake project directory.
    #[serde(default)]
    pub path: Option<String>,

    /// Conan package reference (e.g. `"opencv/4.9.0"`).
    #[serde(default)]
    pub conan: Option<String>,

    /// Specific library components to link against.
    #[serde(default)]
    pub components: Vec<String>,

    /// Override the shared library filename (without platform prefix/suffix).
    #[serde(default)]
    pub lib_name: Option<String>,

    /// Build configuration to use.
    #[serde(default)]
    pub build_type: BuildType,
}

/// CMake-style build type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BuildType {
    Debug,
    #[default]
    Release,
    RelWithDebInfo,
}

impl BuildType {
    /// Returns the string representation used by CMake.
    pub fn as_cmake_str(&self) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Release => "Release",
            Self::RelWithDebInfo => "RelWithDebInfo",
        }
    }
}

impl NativeDependency {
    /// Resolve the effective shared-library name for this dependency.
    ///
    /// If `lib_name` is set it takes precedence, otherwise the dependency `name` is used.
    pub fn effective_lib_name(&self) -> &str {
        self.lib_name.as_deref().unwrap_or(&self.name)
    }

    /// Build the platform-specific shared library filename.
    pub fn shared_lib_filename(&self) -> String {
        let base = self.effective_lib_name();
        if cfg!(target_os = "windows") {
            format!("{base}.dll")
        } else if cfg!(target_os = "macos") {
            format!("lib{base}.dylib")
        } else {
            format!("lib{base}.so")
        }
    }
}

/// Parse a TOML string containing `[native-dependencies]` into a [`NativeConfig`].
pub fn parse_native_config(toml_str: &str) -> Result<NativeConfig, toml::de::Error> {
    toml::from_str(toml_str)
}

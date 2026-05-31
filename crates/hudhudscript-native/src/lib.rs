//! # hudhudscript-native
//!
//! Native C/C++ FFI integration for HudHudScript.
//!
//! This crate provides:
//!
//! - **Configuration** ([`config`]) — parse `[native-dependencies]` from project manifests.
//! - **FFI bridge** ([`ffi`]) — safe wrapper around `libloading` for dynamic library access.
//! - **Type conversions** ([`types`]) — bidirectional mapping between HudHudScript values
//!   and C ABI types.
//! - **Library loader** ([`loader`]) — search-path-based discovery and caching of shared
//!   libraries.
//! - **Builder** ([`builder`]) — shell out to CMake / Conan to compile native dependencies.
//! - **Runtime trait** ([`runtime`]) — `NativeCallable` trait that the interpreter can
//!   consume without depending on this crate.

pub mod builder;
pub mod config;
pub mod error;
pub mod ffi;
pub mod loader;
pub mod runtime;
pub mod types;

// Re-export the most commonly used items at the crate root.
pub use builder::NativeBuilder;
pub use config::{BuildType, NativeConfig, NativeDependency};
pub use error::NativeError;
pub use ffi::{NativeFunction, NativeLibrary};
pub use loader::NativeLoader;
pub use runtime::{NativeCallable, NativeRuntime};
pub use types::{NativeType, NativeValue};

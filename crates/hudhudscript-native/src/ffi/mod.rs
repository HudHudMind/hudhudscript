//! FFI bridge — safe wrapper around `libloading` for dynamic library access.
//!
//! [`NativeLibrary`] loads a shared object (`.so`, `.dylib`, `.dll`) at runtime
//! and exposes registered functions through a type-safe call interface.

pub mod convert;
pub mod dispatch;
pub mod library;
pub mod types;

pub use types::{NativeFunction, NativeLibrary};

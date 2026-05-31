#![allow(dead_code, clippy::needless_lifetimes, clippy::derivable_impls)]
//! C ABI type definitions and value conversions.
//!
//! Provides [`NativeType`] (the type descriptor) and [`NativeValue`] (a concrete value)
//! that bridge HudHudScript runtime values to C-compatible representations.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Descriptor for a C ABI type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeType {
    /// C `void`
    Void,
    /// C `int32_t`
    Int32,
    /// C `int64_t`
    Int64,
    /// C `double`
    Float64,
    /// C `const char*` (null-terminated UTF-8 string)
    String,
    /// C `bool` / `_Bool`
    Bool,
    /// Pointer + length pair (for arrays)
    Array,
    /// Opaque pointer (`void*`)
    Pointer,
}

/// A concrete value that can be passed to or received from a native function.
#[derive(Debug, Clone)]
pub enum NativeValue {
    Void,
    Int32(i32),
    Int64(i64),
    Float64(f64),
    String(std::string::String),
    Bool(bool),
    Array(Vec<NativeValue>),
    Pointer(*mut std::ffi::c_void),
    Null,
}

// NativeValue holds a raw pointer variant, but we never dereference it from Rust.
// The pointer is only shuttled to/from the native library.
unsafe impl Send for NativeValue {}
unsafe impl Sync for NativeValue {}

impl NativeValue {
    /// Return the [`NativeType`] that describes this value.
    pub fn native_type(&self) -> NativeType {
        match self {
            Self::Void => NativeType::Void,
            Self::Int32(_) => NativeType::Int32,
            Self::Int64(_) => NativeType::Int64,
            Self::Float64(_) => NativeType::Float64,
            Self::String(_) => NativeType::String,
            Self::Bool(_) => NativeType::Bool,
            Self::Array(_) => NativeType::Array,
            Self::Pointer(_) | Self::Null => NativeType::Pointer,
        }
    }

    // -- Conversions from HudHudScript-style values --------------------------

    /// Convert an f64 (HudHudScript `Number`) to the requested [`NativeType`].
    pub fn from_number(n: f64, target: NativeType) -> Self {
        match target {
            NativeType::Int32 => Self::Int32(n as i32),
            NativeType::Int64 => Self::Int64(n as i64),
            NativeType::Float64 => Self::Float64(n),
            NativeType::Bool => Self::Bool(n != 0.0),
            _ => Self::Float64(n),
        }
    }

    /// Convert a Rust string to a [`NativeValue::String`].
    pub fn from_string(s: std::string::String) -> Self {
        Self::String(s)
    }

    /// Convert a bool to the requested [`NativeType`].
    pub fn from_bool(b: bool, target: NativeType) -> Self {
        match target {
            NativeType::Int32 => Self::Int32(i32::from(b)),
            NativeType::Bool => Self::Bool(b),
            _ => Self::Bool(b),
        }
    }

    // -- Conversions back to HudHudScript-style primitives -------------------

    /// Try to produce an `f64` from this native value.
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Int32(i) => Some(f64::from(*i)),
            Self::Int64(i) => Some(*i as f64),
            Self::Float64(f) => Some(*f),
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Try to produce a Rust [`String`] from this native value.
    pub fn to_string_value(&self) -> Option<std::string::String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Try to produce a bool from this native value.
    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::Int32(i) => Some(*i != 0),
            Self::Int64(i) => Some(*i != 0),
            _ => None,
        }
    }

    /// Returns `true` if the value represents null/void.
    pub fn is_null_or_void(&self) -> bool {
        matches!(self, Self::Void | Self::Null)
    }

    // -- C ABI helpers -------------------------------------------------------

    /// Convert a [`NativeValue::String`] to a `CString` for passing to C.
    ///
    /// Returns `None` if the value is not a string or contains interior NUL bytes.
    pub fn to_c_string(&self) -> Option<CString> {
        match self {
            Self::String(s) => CString::new(s.as_bytes()).ok(),
            _ => None,
        }
    }

    /// Build a `NativeValue::String` from a raw C `const char*`.
    ///
    /// # Safety
    /// The pointer must be non-null and point to a valid NUL-terminated UTF-8 string.
    pub unsafe fn from_c_str(ptr: *const c_char) -> Self {
        if ptr.is_null() {
            return Self::Null;
        }
        let cstr = unsafe { CStr::from_ptr(ptr) };
        match cstr.to_str() {
            Ok(s) => Self::String(s.to_owned()),
            Err(_) => Self::Null,
        }
    }
}

impl NativeType {
    /// Size in bytes of the C representation (useful for FFI buffer sizing).
    pub fn size_of(&self) -> usize {
        match self {
            Self::Void => 0,
            Self::Int32 => 4,
            Self::Int64 => 8,
            Self::Float64 => 8,
            Self::String => std::mem::size_of::<*const c_char>(),
            Self::Bool => 1,
            Self::Array => {
                std::mem::size_of::<*const std::ffi::c_void>() + std::mem::size_of::<usize>()
            }
            Self::Pointer => std::mem::size_of::<*const std::ffi::c_void>(),
        }
    }
}

/// Convenience: convert a `Vec<NativeValue>` into a `HashMap` keyed by index.
/// Useful when mapping positional C args back to named parameters.
pub fn positional_to_named(
    values: &[NativeValue],
    names: &[&str],
) -> HashMap<std::string::String, NativeValue> {
    names
        .iter()
        .zip(values.iter())
        .map(|(name, val)| ((*name).to_owned(), val.clone()))
        .collect()
}

//! Raw argument conversion — [`NativeValue`] → C ABI values.

use std::ffi::CString;

use crate::error::{NativeError, Result};
use crate::types::NativeValue;

use super::NativeLibrary;

impl NativeLibrary {
    /// Look up a symbol in the loaded library.
    pub(super) unsafe fn get_symbol<T>(&self, name: &str) -> Result<libloading::Symbol<'_, T>> {
        self.lib
            .get(name.as_bytes())
            .map_err(|e| NativeError::SymbolNotFound {
                symbol: name.to_owned(),
                library: self.name.clone(),
                reason: e.to_string(),
            })
    }

    /// Convert a [`NativeValue`] to an integer-sized raw value for the trampoline.
    pub(super) fn to_raw_arg(&self, val: &NativeValue) -> Result<i64> {
        match val {
            NativeValue::Int32(i) => Ok(i64::from(*i)),
            NativeValue::Int64(i) => Ok(*i),
            NativeValue::Bool(b) => Ok(i64::from(*b)),
            NativeValue::Float64(f) => Ok(*f as i64),
            NativeValue::String(s) => {
                let cstr = CString::new(s.as_bytes())
                    .map_err(|_| NativeError::InvalidString { value: s.clone() })?;
                Ok(cstr.into_raw() as i64)
            }
            NativeValue::Pointer(p) => Ok(*p as i64),
            NativeValue::Null => Ok(0),
            NativeValue::Void => Ok(0),
            NativeValue::Array(_) => Err(NativeError::UnsupportedType {
                type_name: "Array".to_owned(),
                context: "raw argument conversion".to_owned(),
            }),
        }
    }

    /// Prepare arguments for an FFI call, returning raw values and tracking
    /// any CString allocations. The returned `_string_guards` vector MUST be
    /// kept alive until after the FFI call completes to prevent use-after-free.
    pub fn prepare_call_args(&self, args: &[NativeValue]) -> Result<(Vec<i64>, Vec<CString>)> {
        let mut raw_args = Vec::with_capacity(args.len());
        let mut string_guards = Vec::new();

        for val in args {
            match val {
                NativeValue::String(s) => {
                    let cstr = CString::new(s.as_bytes())
                        .map_err(|_| NativeError::InvalidString { value: s.clone() })?;
                    raw_args.push(cstr.as_ptr() as i64);
                    string_guards.push(cstr); // Keep alive
                }
                other => {
                    raw_args.push(self.to_raw_arg(other)?);
                }
            }
        }

        Ok((raw_args, string_guards))
    }

    /// Convert a [`NativeValue`] to `f64` for float-returning trampolines.
    pub(super) fn to_raw_f64_arg(&self, val: &NativeValue) -> Result<f64> {
        match val {
            NativeValue::Float64(f) => Ok(*f),
            NativeValue::Int32(i) => Ok(f64::from(*i)),
            NativeValue::Int64(i) => Ok(*i as f64),
            NativeValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            other => Err(NativeError::UnsupportedType {
                type_name: format!("{:?}", other.native_type()),
                context: "f64 argument conversion".to_owned(),
            }),
        }
    }
}

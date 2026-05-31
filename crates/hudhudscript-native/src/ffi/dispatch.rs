//! Internal dispatch helpers for each return-type / arity combination.

use std::ffi::c_char;

use crate::error::{NativeError, Result};
use crate::types::NativeValue;

use super::NativeLibrary;

impl NativeLibrary {
    pub(super) fn call_void(&self, name: &str, args: &[NativeValue]) -> Result<NativeValue> {
        unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn()> = self.get_symbol(name)?;
                    f();
                }
                1 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64)> = self.get_symbol(name)?;
                    f(a0);
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 1,
                    })
                }
            }
        }
        Ok(NativeValue::Void)
    }

    pub(super) fn call_returning_i32(
        &self,
        name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let result: i32 = unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn() -> i32> =
                        self.get_symbol(name)?;
                    f()
                }
                1 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64) -> i32> =
                        self.get_symbol(name)?;
                    f(a0)
                }
                2 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let a1 = self.to_raw_arg(&args[1])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64, i64) -> i32> =
                        self.get_symbol(name)?;
                    f(a0, a1)
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 2,
                    })
                }
            }
        };
        Ok(NativeValue::Int32(result))
    }

    pub(super) fn call_returning_i64(
        &self,
        name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let result: i64 = unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn() -> i64> =
                        self.get_symbol(name)?;
                    f()
                }
                1 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64) -> i64> =
                        self.get_symbol(name)?;
                    f(a0)
                }
                2 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let a1 = self.to_raw_arg(&args[1])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64, i64) -> i64> =
                        self.get_symbol(name)?;
                    f(a0, a1)
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 2,
                    })
                }
            }
        };
        Ok(NativeValue::Int64(result))
    }

    pub(super) fn call_returning_f64(
        &self,
        name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let result: f64 = unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn() -> f64> =
                        self.get_symbol(name)?;
                    f()
                }
                1 => {
                    let a0 = self.to_raw_f64_arg(&args[0])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(f64) -> f64> =
                        self.get_symbol(name)?;
                    f(a0)
                }
                2 => {
                    let a0 = self.to_raw_f64_arg(&args[0])?;
                    let a1 = self.to_raw_f64_arg(&args[1])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(f64, f64) -> f64> =
                        self.get_symbol(name)?;
                    f(a0, a1)
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 2,
                    })
                }
            }
        };
        Ok(NativeValue::Float64(result))
    }

    pub(super) fn call_returning_bool(
        &self,
        name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let result: bool = unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn() -> bool> =
                        self.get_symbol(name)?;
                    f()
                }
                1 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64) -> bool> =
                        self.get_symbol(name)?;
                    f(a0)
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 1,
                    })
                }
            }
        };
        Ok(NativeValue::Bool(result))
    }

    pub(super) fn call_returning_string(
        &self,
        name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let ptr: *const c_char = unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn() -> *const c_char> =
                        self.get_symbol(name)?;
                    f()
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 0,
                    })
                }
            }
        };
        Ok(unsafe { NativeValue::from_c_str(ptr) })
    }

    pub(super) fn call_returning_pointer(
        &self,
        name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let ptr: *mut std::ffi::c_void = unsafe {
            match args.len() {
                0 => {
                    let f: libloading::Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> =
                        self.get_symbol(name)?;
                    f()
                }
                1 => {
                    let a0 = self.to_raw_arg(&args[0])?;
                    let f: libloading::Symbol<unsafe extern "C" fn(i64) -> *mut std::ffi::c_void> =
                        self.get_symbol(name)?;
                    f(a0)
                }
                _ => {
                    return Err(NativeError::TooManyArguments {
                        function: name.to_owned(),
                        max: 1,
                    })
                }
            }
        };
        if ptr.is_null() {
            Ok(NativeValue::Null)
        } else {
            Ok(NativeValue::Pointer(ptr))
        }
    }
}

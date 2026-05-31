use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum NativeExceptionCode {
    /// E0135 — Wrong number of arguments to native function
    NativeArgumentCount = 135,
    /// E0136 — Native binding build failed
    NativeBuildError = 136,
    /// E0137 — Native function not registered
    NativeFunctionNotFound = 137,
    /// E0138 — String contains interior NUL bytes
    NativeInvalidString = 138,
    /// E0139 — Failed to load native library
    NativeLibraryLoad = 139,
    /// E0140 — Native library file not found
    NativeLibraryNotFound = 140,
    /// E0141 — Native library handle not loaded
    NativeLibraryNotLoaded = 141,
    /// E0142 — Symbol not found in library
    NativeSymbolNotFound = 142,
    /// E0143 — Too many arguments to native function
    NativeTooManyArguments = 143,
    /// E0144 — Unsupported FFI type
    NativeUnsupportedType = 144,
}

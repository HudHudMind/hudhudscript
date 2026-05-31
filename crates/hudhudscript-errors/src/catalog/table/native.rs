use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const NATIVE_ARGUMENT_COUNT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(135),
        long_code: "HHS_E_NATIVE_ARGUMENT_COUNT",
        short_code: "E0135",
        title: "Wrong number of arguments to native function",
        short_description: "A call into a native FFI function passed a different number of arguments than the function's registered C signature expects.",
        long_description: "Each native function is registered with the FFI bridge with a fixed arity from its C declaration. The interpreter checks the call site against that arity and rejects mismatches before invoking the symbol — calling a C function with the wrong arg count corrupts the stack and causes hard-to-diagnose crashes, so this is enforced strictly.

The error message names the function and gives both the expected and actual counts. Either the binding declaration is wrong (the C header says 3 args, you registered it as 2) or the call site is wrong.

If the C function is variadic (e.g. `printf`), you must declare and call it through HudHudScript's variadic FFI helper rather than the standard fixed-arity binding — the regular path does not support varargs.",
        hints: &["Compare your binding's arity against the C header declaration", "For variadic C functions, use the variadic FFI helper", "Check that you did not accidentally drop or add an argument", "Re-generate bindings if you have a tool that produces them"],
        example_bad: Some("// C: int add(int a, int b);
native_call(\"add\", 1, 2, 3) // 3 args, expected 2"),
        example_good: Some("native_call(\"add\", 1, 2)"),
        see_also: &["NativeTooManyArguments", "NativeUnsupportedType", "NativeFunctionNotFound"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_BUILD_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(136),
        long_code: "HHS_E_NATIVE_BUILD_ERROR",
        short_code: "E0136",
        title: "Native binding build failed",
        short_description: "Constructing or compiling a native function binding failed before any call could be made.",
        long_description: "This error is raised by the FFI binding builder — the component that turns a declaration like `extern fn foo(i32, *const u8) -> i32` into a callable closure. Building the closure can fail if the type list is internally inconsistent, if a return type is unsupported, if the underlying `libffi` cif preparation rejects the signature, or if the requested calling convention is unavailable on this platform.

The wrapped message tells you which step failed. Most fixes are at the binding declaration: simplify the signature, replace unsupported types with their pointer equivalents, or split a complex struct return into an out-parameter pattern.

This is a build-time FFI error, not a call-time error; it happens once when the binding is registered, so failing fast here is desirable.",
        hints: &["Read the wrapped libffi message — it points at the bad type", "Replace unsupported struct returns with out-parameter patterns", "Verify the calling convention is supported on the target platform", "Simplify the signature and add complexity back incrementally"],
        example_bad: None,
        example_good: None,
        see_also: &["NativeUnsupportedType", "NativeArgumentCount", "NativeLibraryLoad"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_FUNCTION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(137),
        long_code: "HHS_E_NATIVE_FUNCTION_NOT_FOUND",
        short_code: "E0137",
        title: "Native function not registered",
        short_description: "A call referenced a function name that has not been registered as a binding on the named native library.",
        long_description: "Each native library has a registry of functions that have been declared and bound. You called a function by name and that name is not in the registry. This is distinct from `SymbolNotFound`: the symbol may exist in the .so file, but you have not yet declared it in HudHudScript so the runtime has no signature to call it with.

Declare the function with its full C signature before calling it, then re-run. If the function is dynamically discovered, use the lower-level `lookup_symbol` API which returns a typed handle instead of relying on the named registry.

Double-check that the library handle in the call matches the one you used to register the function — registering on `lib_a` and calling on `lib_b` will hit this error even if the names match.",
        hints: &["Declare the function before calling it (extern fn ...)", "Verify the call uses the same library handle as the registration", "Names are case-sensitive and must match exactly", "For dynamic discovery, use `lookup_symbol` instead"],
        example_bad: Some("let lib = native_load(\"libm.so.6\")
native_call(lib, \"sqrt\", 4.0) // sqrt was never declared"),
        example_good: Some("let lib = native_load(\"libm.so.6\")
extern fn sqrt(f64) -> f64 in lib
native_call(lib, \"sqrt\", 4.0)"),
        see_also: &["NativeSymbolNotFound", "NativeLibraryNotLoaded", "NativeArgumentCount"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_INVALID_STRING: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(138),
        long_code: "HHS_E_NATIVE_INVALID_STRING",
        short_code: "E0138",
        title: "String contains interior NUL bytes",
        short_description: "A string passed to a native function contains a NUL byte before its end, which is illegal in C strings.",
        long_description: "C strings are NUL-terminated and cannot contain interior NUL bytes — the C function would interpret the first NUL as the end of the string. The FFI marshaller checks for interior NULs before constructing a CString and refuses to silently truncate, because that would change the value the C function actually receives.

If the string genuinely needs to contain NUL bytes, you cannot pass it through a `*const c_char` parameter. Use a `*const u8` plus an explicit length parameter instead — most C APIs that handle binary data expose this form.

If the NUL bytes are accidental (e.g. you read a UTF-16 file and didn't decode it), fix the upstream conversion before calling FFI.",
        hints: &["Pass the data through a `(ptr, len)` pair, not a C string", "Strip or escape interior NUL bytes before the call", "Check that you decoded UTF-16/UCS-2 input before passing it", "If the API offers a `_n` length-aware variant, use that instead"],
        example_bad: Some("native_call(lib, \"puts\", \"hello\\0world\") // interior NUL"),
        example_good: Some("native_call(lib, \"fwrite\", bytes_ptr, bytes.len, 1, stdout)"),
        see_also: &["NativeUnsupportedType", "NativeArgumentCount", "NativeBuildError"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_LIBRARY_LOAD: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(139),
        long_code: "HHS_E_NATIVE_LIBRARY_LOAD",
        short_code: "E0139",
        title: "Failed to load native library",
        short_description: "The dynamic linker found the file but could not load it as a shared library — usually missing dependencies or wrong architecture.",
        long_description: "The file at the requested path exists, but `dlopen` (or its Windows/macOS equivalent) refused it. The wrapped OS message contains the real reason: missing transitive `.so` dependencies, wrong architecture (x86_64 .so on aarch64 host), incompatible glibc version, or insufficient permissions.

On Linux, run `ldd <path-to-lib>` to see which dependencies the dynamic linker can find — any line ending with `not found` is the missing piece. On macOS, use `otool -L`. On Windows, use Dependencies.exe or similar.

For architecture mismatches, check `file <path>` against `uname -m` on the host. Cross-compiling for the wrong target is a common cause when copying binaries between systems.",
        hints: &["Run `ldd <lib>` (Linux) or `otool -L <lib>` (macOS) to find missing deps", "Confirm the library matches the host architecture (`file` + `uname -m`)", "Set LD_LIBRARY_PATH if dependencies live in a non-standard location", "Read the wrapped dlopen message — it usually names the missing symbol"],
        example_bad: None,
        example_good: None,
        see_also: &["NativeLibraryNotFound", "NativeLibraryNotLoaded", "NativeSymbolNotFound"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_LIBRARY_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(140),
        long_code: "HHS_E_NATIVE_LIBRARY_NOT_FOUND",
        short_code: "E0140",
        title: "Native library file not found",
        short_description: "No file matching the requested library name was found in any of the FFI search paths.",
        long_description: "The FFI loader searches a list of directories (current dir, project `native/`, `LD_LIBRARY_PATH`, system library paths) for a file matching the requested name. None of those paths contained a match. This is distinct from `LibraryLoad`, which means the file was found but failed to load.

Provide an absolute path to the library, place it in one of the search directories, or extend the search path via the configured environment variable. On Linux, `LD_LIBRARY_PATH` augments the loader path; on macOS use `DYLD_LIBRARY_PATH`; on Windows, `PATH` doubles as the DLL search path.

If you are loading a system library, make sure the corresponding development package is installed — many distributions split runtime and development bits, and the unversioned `.so` symlink only ships in the dev package.",
        hints: &["Use an absolute path to the library to bypass search rules", "Set LD_LIBRARY_PATH (Linux) / DYLD_LIBRARY_PATH (macOS) / PATH (Windows)", "Install the `-dev`/`-devel` package for system libraries", "Check the searched paths listed in the error message"],
        example_bad: Some("native_load(\"libfoo\") // not on any search path"),
        example_good: Some("native_load(\"/usr/local/lib/libfoo.so.1\")"),
        see_also: &["NativeLibraryLoad", "NativeLibraryNotLoaded", "NativeSymbolNotFound"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_LIBRARY_NOT_LOADED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(141),
        long_code: "HHS_E_NATIVE_LIBRARY_NOT_LOADED",
        short_code: "E0141",
        title: "Native library handle not loaded",
        short_description: "An operation referenced a library handle that was never loaded or has already been unloaded.",
        long_description: "FFI calls go through a library handle. This error means the handle in question is not currently associated with a loaded shared object — either you never called the load function, you used a stale handle from a previous session, or the handle was explicitly unloaded.

Load the library first and capture its handle, then thread that handle through the rest of your FFI calls. If you cache handles across long-lived sessions (REPLs, hot-reload), be aware that an unload invalidates all outstanding handles to the same library.

If you are designing a library wrapper, lazy-load on first use rather than expecting callers to load explicitly — that pattern eliminates this error class entirely.",
        hints: &["Call `native_load` before any operation on the library", "Do not reuse handles across `unload`/`load` cycles", "Wrap libraries in lazy loaders that load on first use", "Check that the handle variable is in scope at the call site"],
        example_bad: None,
        example_good: None,
        see_also: &["NativeLibraryLoad", "NativeLibraryNotFound", "NativeFunctionNotFound"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_SYMBOL_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(142),
        long_code: "HHS_E_NATIVE_SYMBOL_NOT_FOUND",
        short_code: "E0142",
        title: "Symbol not found in library",
        short_description: "The dynamic linker could not find a symbol with this name in the loaded library.",
        long_description: "The library is loaded but `dlsym` returned no entry for the symbol you requested. This is distinct from `FunctionNotFound`: the binding was declared on the HudHudScript side, but the underlying `.so` does not export a matching symbol.

Common causes: a typo in the symbol name; C++ name mangling (you need the mangled symbol or an `extern \"C\"` wrapper); the symbol is hidden by `-fvisibility=hidden`; you loaded the wrong library version; the symbol exists in a sub-library that was not linked.

Use `nm -D <lib>` (Linux) or `nm -gU <lib>` (macOS) to list exported symbols and confirm the spelling. For C++ libraries, run the result through `c++filt` to demangle.",
        hints: &["Run `nm -D <lib>` to list exported symbols", "For C++ libraries, expose `extern \"C\"` wrappers or use mangled names", "Check `-fvisibility=hidden` is not stripping the symbol", "Verify you loaded the library version that contains this symbol"],
        example_bad: None,
        example_good: None,
        see_also: &["NativeFunctionNotFound", "NativeLibraryLoad", "NativeLibraryNotLoaded"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_TOO_MANY_ARGUMENTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(143),
        long_code: "HHS_E_NATIVE_TOO_MANY_ARGUMENTS",
        short_code: "E0143",
        title: "Too many arguments to native function",
        short_description: "The function declares more arguments than the FFI bridge supports in a single call.",
        long_description: "HudHudScript's FFI bridge supports up to a fixed maximum number of arguments per call (driven by the underlying libffi cif buffer). The function you registered exceeds that limit, so the binding cannot be built.

In practice this affects only pathologically wide C APIs — most real functions take fewer than a dozen arguments. If you hit this with a real-world function, the right fix is almost always to introduce a struct that bundles the arguments, and pass a pointer to the struct instead of N separate scalars. C APIs commonly do this themselves (`struct stat *buf`).

The error message shows the maximum supported arity for your build of HudHudScript. If you control the C side, refactoring to a struct is straightforward; if you do not, write a small C shim that takes a struct and forwards the call.",
        hints: &["Introduce a struct parameter to bundle related arguments", "Write a C shim that takes a struct and forwards to the real function", "Check the error message for the supported maximum arity", "Most APIs with too many args are bugs in the API; refactor if you own it"],
        example_bad: None,
        example_good: None,
        see_also: &["NativeArgumentCount", "NativeBuildError", "NativeUnsupportedType"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub const NATIVE_UNSUPPORTED_TYPE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(144),
        long_code: "HHS_E_NATIVE_UNSUPPORTED_TYPE",
        short_code: "E0144",
        title: "Unsupported FFI type",
        short_description: "The FFI bridge does not know how to marshal one of the types in this binding's signature.",
        long_description: "HudHudScript's FFI supports a fixed set of primitive types (i8/i16/i32/i64, u8/u16/u32/u64, f32, f64, pointers, opaque handles, C strings) plus pointer-to-anything. A type in your signature falls outside that set — typically a pass-by-value struct, a long double, a vector type, or a language-specific type like Rust's `String`.

Replace the unsupported type with a pointer plus length, an opaque handle, or a primitive your binding can convert to/from. For pass-by-value structs, the standard idiom is to allocate the struct on the HudHudScript side and pass a pointer to it; the C function fills it in.

If the C API genuinely returns a struct by value, write a small C shim that calls it and stores the result through an out-parameter pointer.",
        hints: &["Replace structs-by-value with pointer + struct on the script side", "Use `*const u8` + length for byte arrays instead of opaque slices", "Write a C shim for value-returning struct APIs", "Long double, vectors, and complex types are not supported — convert them"],
        example_bad: Some("extern fn make_point() -> Point // returns struct by value"),
        example_good: Some("extern fn make_point(out: *mut Point) -> i32"),
        see_also: &["NativeBuildError", "NativeArgumentCount", "NativeInvalidString"],
        since_version: "0.4.47",
        category: ErrorCategory::Native,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    NATIVE_ARGUMENT_COUNT,
    NATIVE_BUILD_ERROR,
    NATIVE_FUNCTION_NOT_FOUND,
    NATIVE_INVALID_STRING,
    NATIVE_LIBRARY_LOAD,
    NATIVE_LIBRARY_NOT_FOUND,
    NATIVE_LIBRARY_NOT_LOADED,
    NATIVE_SYMBOL_NOT_FOUND,
    NATIVE_TOO_MANY_ARGUMENTS,
    NATIVE_UNSUPPORTED_TYPE,
];

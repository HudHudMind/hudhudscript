//! Public API tests for hudhudscript-native

use hudhudscript_native::{
    config::parse_native_config, types::positional_to_named, BuildType, NativeBuilder,
    NativeCallable, NativeConfig, NativeDependency, NativeError, NativeFunction, NativeLoader,
    NativeRuntime, NativeType, NativeValue,
};
use std::path::PathBuf;

// ── NativeType ──────────────────────────────────────────────────────

#[test]
fn test_native_type_size_of_void() {
    assert_eq!(NativeType::Void.size_of(), 0);
}
#[test]
fn test_native_type_size_of_int32() {
    assert_eq!(NativeType::Int32.size_of(), 4);
}
#[test]
fn test_native_type_size_of_int64() {
    assert_eq!(NativeType::Int64.size_of(), 8);
}
#[test]
fn test_native_type_size_of_float64() {
    assert_eq!(NativeType::Float64.size_of(), 8);
}
#[test]
fn test_native_type_size_of_bool() {
    assert_eq!(NativeType::Bool.size_of(), 1);
}
#[test]
fn test_native_type_size_of_string() {
    assert!(NativeType::String.size_of() > 0);
}
#[test]
fn test_native_type_size_of_pointer() {
    assert!(NativeType::Pointer.size_of() > 0);
}
#[test]
fn test_native_type_size_of_array() {
    assert!(NativeType::Array.size_of() > 0);
}

#[test]
fn test_native_type_equality() {
    assert_eq!(NativeType::Int32, NativeType::Int32);
    assert_ne!(NativeType::Int32, NativeType::Int64);
}

#[test]
fn test_native_type_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NativeType::Int32);
    set.insert(NativeType::Float64);
    assert_eq!(set.len(), 2);
}

// ── NativeValue ─────────────────────────────────────────────────────

#[test]
fn test_native_value_type_void() {
    assert_eq!(NativeValue::Void.native_type(), NativeType::Void);
}
#[test]
fn test_native_value_type_int32() {
    assert_eq!(NativeValue::Int32(42).native_type(), NativeType::Int32);
}
#[test]
fn test_native_value_type_int64() {
    assert_eq!(NativeValue::Int64(99).native_type(), NativeType::Int64);
}
#[test]
fn test_native_value_type_float64() {
    assert_eq!(
        NativeValue::Float64(3.14).native_type(),
        NativeType::Float64
    );
}
#[test]
fn test_native_value_type_string() {
    assert_eq!(
        NativeValue::String("hi".into()).native_type(),
        NativeType::String
    );
}
#[test]
fn test_native_value_type_bool() {
    assert_eq!(NativeValue::Bool(true).native_type(), NativeType::Bool);
}
#[test]
fn test_native_value_type_null() {
    assert_eq!(NativeValue::Null.native_type(), NativeType::Pointer);
}
#[test]
fn test_native_value_type_array() {
    assert_eq!(NativeValue::Array(vec![]).native_type(), NativeType::Array);
}

#[test]
fn test_from_number_int32() {
    assert!(matches!(
        NativeValue::from_number(42.0, NativeType::Int32),
        NativeValue::Int32(42)
    ));
}
#[test]
fn test_from_number_int64() {
    assert!(matches!(
        NativeValue::from_number(42.0, NativeType::Int64),
        NativeValue::Int64(42)
    ));
}
#[test]
fn test_from_number_float64() {
    assert!(matches!(
        NativeValue::from_number(3.14, NativeType::Float64),
        NativeValue::Float64(_)
    ));
}
#[test]
fn test_from_number_bool_true() {
    assert!(matches!(
        NativeValue::from_number(1.0, NativeType::Bool),
        NativeValue::Bool(true)
    ));
}
#[test]
fn test_from_number_bool_false() {
    assert!(matches!(
        NativeValue::from_number(0.0, NativeType::Bool),
        NativeValue::Bool(false)
    ));
}
#[test]
fn test_from_number_fallback() {
    assert!(matches!(
        NativeValue::from_number(42.0, NativeType::String),
        NativeValue::Float64(_)
    ));
}

#[test]
fn test_from_string() {
    assert!(
        matches!(NativeValue::from_string("hello".into()), NativeValue::String(ref s) if s == "hello")
    );
}

#[test]
fn test_from_bool_bool() {
    assert!(matches!(
        NativeValue::from_bool(true, NativeType::Bool),
        NativeValue::Bool(true)
    ));
}
#[test]
fn test_from_bool_int32() {
    assert!(matches!(
        NativeValue::from_bool(true, NativeType::Int32),
        NativeValue::Int32(1)
    ));
}

#[test]
fn test_to_f64() {
    assert_eq!(NativeValue::Int32(10).to_f64(), Some(10.0));
    assert_eq!(NativeValue::Int64(20).to_f64(), Some(20.0));
    assert_eq!(NativeValue::Float64(3.14).to_f64(), Some(3.14));
    assert_eq!(NativeValue::Bool(true).to_f64(), Some(1.0));
    assert_eq!(NativeValue::Bool(false).to_f64(), Some(0.0));
    assert_eq!(NativeValue::String("hi".into()).to_f64(), None);
    assert_eq!(NativeValue::Null.to_f64(), None);
}

#[test]
fn test_to_string_value() {
    assert_eq!(
        NativeValue::String("hi".into()).to_string_value(),
        Some("hi".into())
    );
    assert_eq!(NativeValue::Int32(42).to_string_value(), None);
}

#[test]
fn test_to_bool() {
    assert_eq!(NativeValue::Bool(true).to_bool(), Some(true));
    assert_eq!(NativeValue::Int32(0).to_bool(), Some(false));
    assert_eq!(NativeValue::Int64(0).to_bool(), Some(false));
    assert_eq!(NativeValue::Float64(1.0).to_bool(), None);
}

#[test]
fn test_is_null_or_void() {
    assert!(NativeValue::Void.is_null_or_void());
    assert!(NativeValue::Null.is_null_or_void());
    assert!(!NativeValue::Int32(0).is_null_or_void());
}

#[test]
fn test_to_c_string_ok() {
    assert!(NativeValue::String("hello".into()).to_c_string().is_some());
}
#[test]
fn test_to_c_string_not_string() {
    assert!(NativeValue::Int32(42).to_c_string().is_none());
}
#[test]
fn test_to_c_string_interior_nul() {
    assert!(NativeValue::String("has\0nul".into())
        .to_c_string()
        .is_none());
}

#[test]
fn test_positional_to_named() {
    let map = positional_to_named(
        &[NativeValue::Int32(1), NativeValue::String("hi".into())],
        &["x", "y"],
    );
    assert_eq!(map.len(), 2);
    assert!(map.contains_key("x"));
}

// ── NativeConfig ────────────────────────────────────────────────────

#[test]
fn test_native_config_default() {
    assert!(NativeConfig::default().native_dependencies.is_empty());
}

#[test]
fn test_parse_native_config_basic() {
    let config =
        parse_native_config("[native-dependencies.mylib]\nname = \"mylib\"\npath = \"./vendor\"")
            .unwrap();
    assert_eq!(config.native_dependencies.len(), 1);
}

#[test]
fn test_parse_native_config_invalid() {
    assert!(parse_native_config("not valid toml {{{").is_err());
}

#[test]
fn test_parse_native_config_empty() {
    assert!(parse_native_config("")
        .unwrap()
        .native_dependencies
        .is_empty());
}

// ── BuildType ───────────────────────────────────────────────────────

#[test]
fn test_build_type_default() {
    assert_eq!(BuildType::default(), BuildType::Release);
}
#[test]
fn test_build_type_cmake_debug() {
    assert_eq!(BuildType::Debug.as_cmake_str(), "Debug");
}
#[test]
fn test_build_type_cmake_release() {
    assert_eq!(BuildType::Release.as_cmake_str(), "Release");
}
#[test]
fn test_build_type_cmake_rwdi() {
    assert_eq!(BuildType::RelWithDebInfo.as_cmake_str(), "RelWithDebInfo");
}
#[test]
fn test_build_type_inequality() {
    assert_ne!(BuildType::Debug, BuildType::Release);
}

// ── NativeDependency ────────────────────────────────────────────────

#[test]
fn test_effective_lib_name_no_override() {
    let dep = NativeDependency {
        name: "mylib".into(),
        path: None,
        conan: None,
        components: vec![],
        lib_name: None,
        build_type: BuildType::Release,
    };
    assert_eq!(dep.effective_lib_name(), "mylib");
}

#[test]
fn test_effective_lib_name_with_override() {
    let dep = NativeDependency {
        name: "boost".into(),
        path: None,
        conan: None,
        components: vec![],
        lib_name: Some("boost_system".into()),
        build_type: BuildType::Release,
    };
    assert_eq!(dep.effective_lib_name(), "boost_system");
}

#[test]
fn test_shared_lib_filename_linux() {
    let dep = NativeDependency {
        name: "test".into(),
        path: None,
        conan: None,
        components: vec![],
        lib_name: None,
        build_type: BuildType::Release,
    };
    if cfg!(target_os = "linux") {
        assert_eq!(dep.shared_lib_filename(), "libtest.so");
    }
}

// ── NativeBuilder ───────────────────────────────────────────────────

#[test]
fn test_native_builder_new() {
    let b = NativeBuilder::new(PathBuf::from("/project"), BuildType::Debug);
    assert_eq!(b.project_dir, PathBuf::from("/project"));
    assert!(b.build_dir.to_string_lossy().contains(".hudpackages"));
}

#[test]
fn test_native_builder_output_lib_dir() {
    assert!(NativeBuilder::new(PathBuf::from("/p"), BuildType::Release)
        .output_lib_dir()
        .to_string_lossy()
        .contains("native/lib"));
}

// ── NativeFunction ──────────────────────────────────────────────────

#[test]
fn test_native_function_metadata() {
    let f = NativeFunction {
        name: "add".into(),
        param_types: vec![NativeType::Int32, NativeType::Int32],
        return_type: NativeType::Int32,
    };
    assert_eq!(f.name, "add");
    assert_eq!(f.param_types.len(), 2);
}

// ── NativeLoader ────────────────────────────────────────────────────

#[test]
fn test_native_loader_new() {
    assert!(!NativeLoader::new().is_loaded("x"));
}
#[test]
fn test_native_loader_default() {
    assert!(!NativeLoader::default().is_loaded("x"));
}
#[test]
fn test_native_loader_search_paths() {
    assert!(NativeLoader::new().search_paths().len() >= 2);
}

#[test]
fn test_native_loader_add_search_path() {
    let mut l = NativeLoader::new();
    let n = l.search_paths().len();
    l.add_search_path(PathBuf::from("/custom/lib"));
    assert_eq!(l.search_paths().len(), n + 1);
    l.add_search_path(PathBuf::from("/custom/lib"));
    assert_eq!(l.search_paths().len(), n + 1);
}

#[test]
fn test_native_loader_load_not_found() {
    assert!(NativeLoader::new().load_library("nonexistent_xyz").is_err());
}
#[test]
fn test_native_loader_get_library_none() {
    assert!(NativeLoader::new().get_library("nope").is_none());
}
#[test]
fn test_native_loader_register_not_loaded() {
    assert!(NativeLoader::new()
        .register_function(
            "x",
            NativeFunction {
                name: "f".into(),
                param_types: vec![],
                return_type: NativeType::Void
            }
        )
        .is_err());
}
#[test]
fn test_native_loader_call_not_loaded() {
    assert!(NativeLoader::new().call_function("x", "f", &[]).is_err());
}

// ── NativeRuntime ───────────────────────────────────────────────────

#[test]
fn test_native_runtime_new() {
    let _ = format!("{:?}", NativeRuntime::new());
}
#[test]
fn test_native_runtime_default() {
    assert!(!NativeRuntime::default().loader().is_loaded("x"));
}
#[test]
fn test_native_runtime_not_available() {
    assert!(!NativeRuntime::new().is_native_available("x", "f"));
}
#[test]
fn test_native_runtime_loader_mut() {
    NativeRuntime::new()
        .loader_mut()
        .add_search_path(PathBuf::from("/e"));
}

// ── NativeError ─────────────────────────────────────────────────────

#[test]
fn test_err_library_load() {
    assert!(NativeError::LibraryLoad {
        path: "/l".into(),
        reason: "r".into()
    }
    .to_string()
    .contains("/l"));
}
#[test]
fn test_err_library_not_found() {
    assert!(NativeError::LibraryNotFound {
        name: "x".into(),
        search_paths: vec![]
    }
    .to_string()
    .contains("not found"));
}
#[test]
fn test_err_symbol_not_found() {
    assert!(NativeError::SymbolNotFound {
        symbol: "f".into(),
        library: "l".into(),
        reason: "r".into()
    }
    .to_string()
    .contains("f"));
}
#[test]
fn test_err_argument_count() {
    assert!(NativeError::ArgumentCount {
        function: "f".into(),
        expected: 2,
        got: 3
    }
    .to_string()
    .contains("2"));
}
#[test]
fn test_err_too_many_args() {
    assert!(NativeError::TooManyArguments {
        function: "f".into(),
        max: 4
    }
    .to_string()
    .contains("4"));
}
#[test]
fn test_err_invalid_string() {
    assert!(NativeError::InvalidString { value: "x".into() }
        .to_string()
        .contains("NUL"));
}
#[test]
fn test_err_unsupported_type() {
    assert!(NativeError::UnsupportedType {
        type_name: "A".into(),
        context: "c".into()
    }
    .to_string()
    .contains("A"));
}
#[test]
fn test_err_build_error() {
    assert!(NativeError::BuildError {
        message: "cmake".into()
    }
    .to_string()
    .contains("cmake"));
}
#[test]
fn test_err_function_not_found() {
    assert!(NativeError::FunctionNotFound {
        function: "f".into(),
        library: "l".into()
    }
    .to_string()
    .contains("not registered"));
}
#[test]
fn test_err_library_not_loaded() {
    assert!(NativeError::LibraryNotLoaded { name: "x".into() }
        .to_string()
        .contains("not loaded"));
}

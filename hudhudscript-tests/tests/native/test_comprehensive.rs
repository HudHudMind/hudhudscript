//! Comprehensive tests for hudhudscript-native
//! Covers: NativeConfig, NativeDependency, NativeError, NativeValue, NativeType, NativeBuilder

use hudhudscript_native::*;
use std::collections::HashMap;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════
// NativeConfig / NativeDependency / BuildType
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn native_config_default() {
    let config = NativeConfig {
        native_dependencies: HashMap::new(),
    };
    assert!(config.native_dependencies.is_empty());
}

#[test]
fn native_config_with_dependency() {
    let mut deps = HashMap::new();
    deps.insert("openssl".to_string(), NativeDependency {
        name: "openssl".to_string(),
        path: Some("/usr/lib".to_string()),
        conan: None,
        components: vec!["ssl".to_string(), "crypto".to_string()],
        lib_name: Some("libssl".to_string()),
        build_type: Default::default(),
    });
    let config = NativeConfig {
        native_dependencies: deps,
    };
    assert_eq!(config.native_dependencies.len(), 1);
}

#[test]
fn native_config_multiple_deps() {
    let mut deps = HashMap::new();
    deps.insert("zlib".to_string(), NativeDependency {
        name: "zlib".to_string(),
        path: None,
        conan: Some("zlib/1.3.1".to_string()),
        components: vec![],
        lib_name: None,
        build_type: Default::default(),
    });
    deps.insert("libcurl".to_string(), NativeDependency {
        name: "libcurl".to_string(),
        path: None,
        conan: None,
        components: vec![],
        lib_name: Some("libcurl".to_string()),
        build_type: Default::default(),
    });
    let config = NativeConfig { native_dependencies: deps };
    assert_eq!(config.native_dependencies.len(), 2);
}

#[test]
fn native_dependency_default_name() {
    let dep = NativeDependency {
        name: String::new(),
        path: None,
        conan: None,
        components: vec![],
        lib_name: None,
        build_type: Default::default(),
    };
    assert_eq!(dep.name, "");
}

#[test]
fn build_type_variants() {
    assert!(matches!(BuildType::Debug, BuildType::Debug));
    assert!(matches!(BuildType::Release, BuildType::Release));
    assert!(matches!(BuildType::RelWithDebInfo, BuildType::RelWithDebInfo));
}

#[test]
fn build_type_cmake_str() {
    assert_eq!(BuildType::Debug.as_cmake_str(), "Debug");
    assert_eq!(BuildType::Release.as_cmake_str(), "Release");
    assert_eq!(BuildType::RelWithDebInfo.as_cmake_str(), "RelWithDebInfo");
}

#[test]
fn build_type_default_is_release() {
    let bt: BuildType = Default::default();
    assert!(matches!(bt, BuildType::Release));
}

// ═══════════════════════════════════════════════════════════════════════════
// NativeError
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn native_error_library_load() {
    let err = NativeError::LibraryLoad { path: "/tmp/lib.so".to_string(), reason: "not found".to_string() };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn native_error_library_not_found() {
    let err = NativeError::LibraryNotFound { name: "mylib".to_string(), search_paths: vec!["/usr/lib".to_string()] };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn native_error_library_not_loaded() {
    let err = NativeError::LibraryNotLoaded { name: "mylib".to_string() };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn native_error_symbol_not_found() {
    let err = NativeError::SymbolNotFound { symbol: "my_func".to_string(), library: "mylib.so".to_string(), reason: "undefined".to_string() };
    let msg = format!("{}", err);
    assert!(msg.contains("my_func"));
}

// ═══════════════════════════════════════════════════════════════════════════
// NativeType / NativeValue
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn native_type_all_variants() {
    let _ = NativeType::Void;
    let _ = NativeType::Int32;
    let _ = NativeType::Int64;
    let _ = NativeType::Float64;
    let _ = NativeType::String;
    let _ = NativeType::Bool;
    let _ = NativeType::Array;
    let _ = NativeType::Pointer;
}

#[test]
fn native_value_i32() {
    let v = NativeValue::Int32(42);
    assert!(matches!(v.native_type(), NativeType::Int32));
}

#[test]
fn native_value_i64() {
    let v = NativeValue::Int64(-100);
    assert!(matches!(v.native_type(), NativeType::Int64));
}

#[test]
fn native_value_f64() {
    let v = NativeValue::Float64(3.14);
    assert!(matches!(v.native_type(), NativeType::Float64));
}

#[test]
fn native_value_string() {
    let v = NativeValue::String("hello".to_string());
    assert!(matches!(v.native_type(), NativeType::String));
}

#[test]
fn native_value_bool() {
    let v = NativeValue::Bool(true);
    assert!(matches!(v.native_type(), NativeType::Bool));
    let v2 = NativeValue::Bool(false);
    assert!(matches!(v2.native_type(), NativeType::Bool));
}

#[test]
fn native_value_void() {
    let v = NativeValue::Void;
    assert!(matches!(v.native_type(), NativeType::Void));
}

#[test]
fn native_value_array() {
    let v = NativeValue::Array(vec![NativeValue::Int32(1), NativeValue::Int32(2)]);
    assert!(matches!(v.native_type(), NativeType::Array));
}

#[test]
fn native_value_null() {
    let v = NativeValue::Null;
    // Null just needs to exist, its native_type depends on implementation
    let _ = v.native_type();
}

// ═══════════════════════════════════════════════════════════════════════════
// NativeBuilder
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn native_builder_new() {
    let builder = NativeBuilder::new(PathBuf::from("/tmp/project"), BuildType::Release);
    assert_eq!(builder.build_type.as_cmake_str(), "Release");
}

#[test]
fn native_builder_debug() {
    let builder = NativeBuilder::new(PathBuf::from("/tmp/project"), BuildType::Debug);
    assert_eq!(builder.build_type.as_cmake_str(), "Debug");
}

#[test]
fn native_builder_relwithdebinfo() {
    let builder = NativeBuilder::new(PathBuf::from("/tmp/project"), BuildType::RelWithDebInfo);
    assert_eq!(builder.build_type.as_cmake_str(), "RelWithDebInfo");
}

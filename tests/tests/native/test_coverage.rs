//! Additional coverage tests for hudhudscript-native.
//!
//! Targets untested paths in config, types, loader, runtime, builder, and error modules.

use hudhudscript_native::{
    config::parse_native_config, types::positional_to_named, BuildType, NativeBuilder,
    NativeCallable, NativeConfig, NativeDependency, NativeError, NativeFunction, NativeLoader,
    NativeRuntime, NativeType, NativeValue,
};
use std::path::PathBuf;

// ── NativeValue: from_bool fallback branch ─────────────────────────

#[test]
fn test_from_bool_fallback_to_bool_on_non_int_target() {
    // When target is not Int32 or Bool, from_bool should return Bool variant
    let val = NativeValue::from_bool(true, NativeType::Float64);
    assert!(matches!(val, NativeValue::Bool(true)));

    let val2 = NativeValue::from_bool(false, NativeType::String);
    assert!(matches!(val2, NativeValue::Bool(false)));

    let val3 = NativeValue::from_bool(true, NativeType::Pointer);
    assert!(matches!(val3, NativeValue::Bool(true)));
}

// ── NativeValue: from_number edge cases ────────────────────────────

#[test]
fn test_from_number_fallback_for_void_and_pointer() {
    // Void, Array, Pointer all hit the _ => Float64 fallback
    let void_val = NativeValue::from_number(7.0, NativeType::Void);
    assert!(matches!(void_val, NativeValue::Float64(f) if (f - 7.0).abs() < f64::EPSILON));

    let ptr_val = NativeValue::from_number(99.0, NativeType::Pointer);
    assert!(matches!(ptr_val, NativeValue::Float64(f) if (f - 99.0).abs() < f64::EPSILON));

    let arr_val = NativeValue::from_number(1.0, NativeType::Array);
    assert!(matches!(arr_val, NativeValue::Float64(f) if (f - 1.0).abs() < f64::EPSILON));
}

// ── NativeValue: Pointer variant native_type ───────────────────────

#[test]
fn test_native_value_pointer_native_type() {
    let ptr = NativeValue::Pointer(std::ptr::null_mut());
    assert_eq!(ptr.native_type(), NativeType::Pointer);
}

// ── NativeValue: to_f64 returns None for non-numeric types ─────────

#[test]
fn test_to_f64_returns_none_for_void_and_array() {
    assert_eq!(NativeValue::Void.to_f64(), None);
    assert_eq!(
        NativeValue::Array(vec![NativeValue::Int32(1)]).to_f64(),
        None
    );
    assert_eq!(NativeValue::Pointer(std::ptr::null_mut()).to_f64(), None);
}

// ── NativeValue: to_bool edge cases ────────────────────────────────

#[test]
fn test_to_bool_nonzero_int32_and_int64() {
    assert_eq!(NativeValue::Int32(42).to_bool(), Some(true));
    assert_eq!(NativeValue::Int64(-1).to_bool(), Some(true));
    assert_eq!(NativeValue::Int64(0).to_bool(), Some(false));
    // Non-boolean-convertible types
    assert_eq!(NativeValue::String("true".into()).to_bool(), None);
    assert_eq!(NativeValue::Void.to_bool(), None);
    assert_eq!(NativeValue::Null.to_bool(), None);
    assert_eq!(NativeValue::Array(vec![]).to_bool(), None);
    assert_eq!(NativeValue::Pointer(std::ptr::null_mut()).to_bool(), None);
}

// ── NativeValue: to_string_value returns None for all non-String ───

#[test]
fn test_to_string_value_none_for_all_non_string() {
    assert_eq!(NativeValue::Void.to_string_value(), None);
    assert_eq!(NativeValue::Bool(true).to_string_value(), None);
    assert_eq!(NativeValue::Int64(100).to_string_value(), None);
    assert_eq!(NativeValue::Float64(1.5).to_string_value(), None);
    assert_eq!(NativeValue::Null.to_string_value(), None);
    assert_eq!(NativeValue::Array(vec![]).to_string_value(), None);
    assert_eq!(
        NativeValue::Pointer(std::ptr::null_mut()).to_string_value(),
        None
    );
}

// ── NativeValue: is_null_or_void negative cases ────────────────────

#[test]
fn test_is_null_or_void_false_for_all_value_types() {
    assert!(!NativeValue::Int64(0).is_null_or_void());
    assert!(!NativeValue::Float64(0.0).is_null_or_void());
    assert!(!NativeValue::Bool(false).is_null_or_void());
    assert!(!NativeValue::String("".into()).is_null_or_void());
    assert!(!NativeValue::Array(vec![]).is_null_or_void());
    assert!(!NativeValue::Pointer(std::ptr::null_mut()).is_null_or_void());
}

// ── NativeValue: to_c_string edge cases ────────────────────────────

#[test]
fn test_to_c_string_empty_string() {
    let val = NativeValue::String(String::new());
    let cstr = val.to_c_string();
    assert!(cstr.is_some());
    assert_eq!(cstr.unwrap().to_str().unwrap(), "");
}

#[test]
fn test_to_c_string_returns_none_for_all_non_string_variants() {
    assert!(NativeValue::Void.to_c_string().is_none());
    assert!(NativeValue::Bool(true).to_c_string().is_none());
    assert!(NativeValue::Int64(5).to_c_string().is_none());
    assert!(NativeValue::Float64(1.0).to_c_string().is_none());
    assert!(NativeValue::Null.to_c_string().is_none());
    assert!(NativeValue::Array(vec![]).to_c_string().is_none());
    assert!(NativeValue::Pointer(std::ptr::null_mut())
        .to_c_string()
        .is_none());
}

// ── NativeValue: unsafe from_c_str with null pointer ───────────────

#[test]
fn test_from_c_str_null_returns_null() {
    let val = unsafe { NativeValue::from_c_str(std::ptr::null()) };
    assert!(val.is_null_or_void());
    assert!(matches!(val, NativeValue::Null));
}

#[test]
fn test_from_c_str_valid_pointer() {
    let cstring = std::ffi::CString::new("hello from C").unwrap();
    let val = unsafe { NativeValue::from_c_str(cstring.as_ptr()) };
    assert_eq!(val.to_string_value(), Some("hello from C".to_string()));
}

// ── NativeValue: Clone and Debug ───────────────────────────────────

#[test]
fn test_native_value_clone_all_variants() {
    let values = vec![
        NativeValue::Void,
        NativeValue::Int32(42),
        NativeValue::Int64(100),
        NativeValue::Float64(3.14),
        NativeValue::String("test".into()),
        NativeValue::Bool(true),
        NativeValue::Array(vec![NativeValue::Int32(1), NativeValue::Bool(false)]),
        NativeValue::Pointer(std::ptr::null_mut()),
        NativeValue::Null,
    ];

    for val in &values {
        let cloned = val.clone();
        // Verify the clone has the same type
        assert_eq!(cloned.native_type(), val.native_type());
    }
}

#[test]
fn test_native_value_debug_format() {
    let val = NativeValue::Int32(42);
    let dbg = format!("{:?}", val);
    assert!(dbg.contains("Int32"));
    assert!(dbg.contains("42"));

    let arr = NativeValue::Array(vec![NativeValue::Bool(true)]);
    let dbg_arr = format!("{:?}", arr);
    assert!(dbg_arr.contains("Array"));
}

// ── positional_to_named: edge cases ────────────────────────────────

#[test]
fn test_positional_to_named_empty() {
    let map = positional_to_named(&[], &[]);
    assert!(map.is_empty());
}

#[test]
fn test_positional_to_named_more_names_than_values() {
    // zip stops at the shorter iterator
    let map = positional_to_named(&[NativeValue::Int32(1)], &["a", "b", "c"]);
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("a"));
    assert!(!map.contains_key("b"));
}

#[test]
fn test_positional_to_named_more_values_than_names() {
    let map = positional_to_named(
        &[
            NativeValue::Int32(1),
            NativeValue::Int32(2),
            NativeValue::Int32(3),
        ],
        &["x"],
    );
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("x"));
}

// ── NativeType: size_of consistency ────────────────────────────────

#[test]
fn test_native_type_string_size_equals_pointer_size() {
    // String is represented as *const c_char, which is pointer-sized
    assert_eq!(NativeType::String.size_of(), NativeType::Pointer.size_of());
}

#[test]
fn test_native_type_array_size_is_pointer_plus_usize() {
    let expected = std::mem::size_of::<*const std::ffi::c_void>() + std::mem::size_of::<usize>();
    assert_eq!(NativeType::Array.size_of(), expected);
}

// ── NativeConfig: parse with full dependency fields ────────────────

#[test]
fn test_parse_config_with_conan_and_components() {
    let toml_str = r#"
[native-dependencies.opencv]
name = "opencv"
conan = "opencv/4.9.0"
components = ["core", "imgproc", "highgui"]
build_type = "Debug"

[native-dependencies.boost]
name = "boost"
path = "./vendor/boost"
lib_name = "boost_system"
"#;
    let config = parse_native_config(toml_str).unwrap();
    assert_eq!(config.native_dependencies.len(), 2);

    let opencv = &config.native_dependencies["opencv"];
    assert_eq!(opencv.conan.as_deref(), Some("opencv/4.9.0"));
    assert_eq!(opencv.components.len(), 3);
    assert_eq!(opencv.build_type, BuildType::Debug);
    assert!(opencv.path.is_none());

    let boost = &config.native_dependencies["boost"];
    assert_eq!(boost.path.as_deref(), Some("./vendor/boost"));
    assert_eq!(boost.lib_name.as_deref(), Some("boost_system"));
    assert!(boost.conan.is_none());
}

// ── NativeDependency: shared_lib_filename with lib_name override ───

#[test]
fn test_shared_lib_filename_with_lib_name_override() {
    let dep = NativeDependency {
        name: "mylib".into(),
        path: None,
        conan: None,
        components: vec![],
        lib_name: Some("custom_name".into()),
        build_type: BuildType::Release,
    };
    let filename = dep.shared_lib_filename();
    // Should use lib_name, not name
    if cfg!(target_os = "linux") {
        assert_eq!(filename, "libcustom_name.so");
    } else if cfg!(target_os = "macos") {
        assert_eq!(filename, "libcustom_name.dylib");
    } else if cfg!(target_os = "windows") {
        assert_eq!(filename, "custom_name.dll");
    }
    // The filename should always contain "custom_name"
    assert!(filename.contains("custom_name"));
}

// ── NativeLoader: load_library_from_path with nonexistent path ─────

#[test]
fn test_loader_load_library_from_path_nonexistent() {
    let mut loader = NativeLoader::new();
    let result =
        loader.load_library_from_path("fake", std::path::Path::new("/nonexistent/libfake.so"));
    assert!(result.is_err());
}

#[test]
fn test_loader_get_library_mut_returns_none_for_unloaded() {
    let mut loader = NativeLoader::new();
    assert!(loader.get_library_mut("nonexistent").is_none());
}

#[test]
fn test_loader_debug_format() {
    let loader = NativeLoader::new();
    let dbg = format!("{:?}", loader);
    assert!(dbg.contains("NativeLoader"));
    assert!(dbg.contains("search_paths"));
}

// ── NativeRuntime: with_loader constructor ─────────────────────────

#[test]
fn test_native_runtime_with_loader() {
    let mut loader = NativeLoader::new();
    loader.add_search_path(PathBuf::from("/custom/path"));
    let runtime = NativeRuntime::with_loader(loader);
    // Verify the custom search path made it through
    assert!(runtime
        .loader()
        .search_paths()
        .iter()
        .any(|p| p == &PathBuf::from("/custom/path")));
}

#[test]
fn test_native_runtime_register_function_on_unloaded_lib() {
    let mut runtime = NativeRuntime::new();
    let result = runtime.register_function(
        "nonexistent_lib",
        "some_func",
        vec![NativeType::Int32],
        NativeType::Void,
    );
    assert!(result.is_err());
}

#[test]
fn test_native_runtime_call_native_missing_library() {
    let mut runtime = NativeRuntime::new();
    let result = runtime.call_native("nonexistent_xyz_lib", "func", vec![]);
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("not found"));
}

// ── NativeBuilder: different build types ───────────────────────────

#[test]
fn test_native_builder_all_build_types() {
    for bt in [
        BuildType::Debug,
        BuildType::Release,
        BuildType::RelWithDebInfo,
    ] {
        let builder = NativeBuilder::new(PathBuf::from("/project"), bt);
        assert_eq!(builder.build_type.as_cmake_str(), bt.as_cmake_str());
        assert_eq!(builder.project_dir, PathBuf::from("/project"));
        // build_dir should be under project_dir
        assert!(builder.build_dir.starts_with("/project"));
    }
}

#[test]
fn test_native_builder_build_cmake_nonexistent_dir() {
    let builder = NativeBuilder::new(
        PathBuf::from("/tmp/hudhud_test_nonexistent_project"),
        BuildType::Release,
    );
    // cmake configure will fail because the source dir doesn't exist
    let result = builder.build_cmake(std::path::Path::new("/nonexistent/cmake/dir"));
    assert!(result.is_err());
}

// ── NativeError: Debug formatting ──────────────────────────────────

#[test]
fn test_native_error_debug_all_variants() {
    let errors: Vec<NativeError> = vec![
        NativeError::LibraryLoad {
            path: "/p".into(),
            reason: "r".into(),
        },
        NativeError::LibraryNotFound {
            name: "n".into(),
            search_paths: vec!["a".into(), "b".into()],
        },
        NativeError::LibraryNotLoaded { name: "n".into() },
        NativeError::SymbolNotFound {
            symbol: "s".into(),
            library: "l".into(),
            reason: "r".into(),
        },
        NativeError::FunctionNotFound {
            function: "f".into(),
            library: "l".into(),
        },
        NativeError::ArgumentCount {
            function: "f".into(),
            expected: 2,
            got: 3,
        },
        NativeError::TooManyArguments {
            function: "f".into(),
            max: 4,
        },
        NativeError::InvalidString { value: "v".into() },
        NativeError::UnsupportedType {
            type_name: "t".into(),
            context: "c".into(),
        },
        NativeError::BuildError {
            message: "m".into(),
        },
    ];

    for err in &errors {
        let dbg = format!("{:?}", err);
        assert!(!dbg.is_empty(), "Debug output should not be empty");
        // Also test Display via to_string
        let display = err.to_string();
        assert!(!display.is_empty(), "Display output should not be empty");
    }
}

// ── NativeFunction: Clone ──────────────────────────────────────────

#[test]
fn test_native_function_clone_and_debug() {
    let func = NativeFunction {
        name: "add".into(),
        param_types: vec![NativeType::Int32, NativeType::Int32],
        return_type: NativeType::Int64,
    };
    let cloned = func.clone();
    assert_eq!(cloned.name, "add");
    assert_eq!(cloned.param_types.len(), 2);
    assert_eq!(cloned.return_type, NativeType::Int64);

    let dbg = format!("{:?}", func);
    assert!(dbg.contains("add"));
}

// ── NativeType: Clone, Copy, Debug, Hash ───────────────────────────

#[test]
fn test_native_type_clone_copy_debug() {
    let t = NativeType::Float64;
    let t2 = t; // Copy
    let t3 = t.clone(); // Clone
    assert_eq!(t, t2);
    assert_eq!(t, t3);

    let dbg = format!("{:?}", t);
    assert!(dbg.contains("Float64"));
}

#[test]
fn test_native_type_hash_all_variants() {
    use std::collections::HashSet;
    let all_types = vec![
        NativeType::Void,
        NativeType::Int32,
        NativeType::Int64,
        NativeType::Float64,
        NativeType::String,
        NativeType::Bool,
        NativeType::Array,
        NativeType::Pointer,
    ];
    let set: HashSet<NativeType> = all_types.into_iter().collect();
    assert_eq!(set.len(), 8, "All 8 NativeType variants should be distinct");
}

// ── NativeConfig: serialization roundtrip ──────────────────────────

#[test]
fn test_native_config_serde_roundtrip() {
    let toml_str = r#"
[native-dependencies.mylib]
name = "mylib"
path = "./vendor"
components = ["core"]
build_type = "RelWithDebInfo"
"#;
    let config: NativeConfig = parse_native_config(toml_str).unwrap();
    let dep = &config.native_dependencies["mylib"];
    assert_eq!(dep.name, "mylib");
    assert_eq!(dep.path.as_deref(), Some("./vendor"));
    assert_eq!(dep.components, vec!["core"]);
    assert_eq!(dep.build_type, BuildType::RelWithDebInfo);

    // Serialize back to TOML and re-parse
    let serialized = toml::to_string(&config).unwrap();
    let reparsed: NativeConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(reparsed.native_dependencies.len(), 1);
    let redep = &reparsed.native_dependencies["mylib"];
    assert_eq!(redep.name, "mylib");
    assert_eq!(redep.build_type, BuildType::RelWithDebInfo);
}

// ── NativeValue: Send + Sync ───────────────────────────────────────

#[test]
fn test_native_value_is_send_and_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<NativeValue>();
    assert_sync::<NativeValue>();
}

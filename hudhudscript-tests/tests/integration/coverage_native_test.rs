//! Coverage tests for hudhudscript-native crate

// ------------------------------------------------------------
// Basic import tests - just verify everything compiles
// ------------------------------------------------------------

#[test]
fn test_native_modules_can_be_imported() {
    // If these imports succeed, test passes
    use hudhudscript_native::{
        builder, config, error, ffi, loader, runtime, types, NativeBuilder, NativeCallable,
        NativeConfig, NativeDependency, NativeError, NativeFunction, NativeLibrary, NativeLoader,
        NativeRuntime, NativeType, NativeValue,
    };

    // Use the types to prove they exist (compile-time only)
    fn assert_types_exist() {
        let _: Option<NativeBuilder> = None;
        let _: Option<NativeConfig> = None;
        let _: Option<NativeDependency> = None;
        let _: Option<NativeError> = None;
        let _: Option<NativeFunction> = None;
        let _: Option<NativeLibrary> = None;
        let _: Option<NativeLoader> = None;
        let _: Option<Box<dyn NativeCallable>> = None;
        let _: Option<NativeRuntime> = None;
        let _: Option<NativeType> = None;
        let _: Option<NativeValue> = None;

        // Use enum variants to prove modules exist
        let _build_type = config::BuildType::Debug;
        let _native_type = types::NativeType::Int32;
    }

    assert_types_exist();
}

#[test]
fn test_native_builder_type() {
    use hudhudscript_native::NativeBuilder;
    use std::mem;

    // Just verify the type exists
    let _size = mem::size_of::<NativeBuilder>();
    // If it compiles, test passes
}

#[test]
fn test_native_runtime_new() {
    use hudhudscript_native::NativeRuntime;

    // Create a runtime instance
    let runtime = NativeRuntime::new();
    // Should not panic and should implement Debug
    let _debug = format!("{:?}", runtime);
}

#[test]
fn test_native_loader_new() {
    use hudhudscript_native::NativeLoader;

    // Create a loader instance
    let loader = NativeLoader::new();
    // Should not panic and should implement Debug
    let _debug = format!("{:?}", loader);
}

#[test]
fn test_native_type_enum() {
    use hudhudscript_native::NativeType;

    // Test that some enum variants exist
    match NativeType::Int32 {
        NativeType::Int32 => {}
        _ => panic!("Expected Int32 variant"),
    }

    // Check other variants if they exist using pattern matching
    match NativeType::Bool {
        NativeType::Bool => {}
        _ => panic!("Expected Bool variant"),
    }
    match NativeType::String {
        NativeType::String => {}
        _ => panic!("Expected String variant"),
    }
    match NativeType::Void {
        NativeType::Void => {}
        _ => panic!("Expected Void variant"),
    }
}

#[test]
fn test_native_value_enum() {
    use hudhudscript_native::NativeValue;

    // Create some values
    let int_val = NativeValue::Int32(42);
    let bool_val = NativeValue::Bool(true);
    let string_val = NativeValue::String("test".to_string());

    match int_val {
        NativeValue::Int32(42) => {}
        _ => panic!("Expected Int32(42)"),
    }

    match bool_val {
        NativeValue::Bool(true) => {}
        _ => panic!("Expected Bool(true)"),
    }

    match string_val {
        NativeValue::String(s) => assert_eq!(s, "test"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_native_error_enum() {
    use hudhudscript_native::NativeError;

    // Create an error
    let error = NativeError::LibraryLoad {
        path: "test.so".to_string(),
        reason: "not found".to_string(),
    };

    match error {
        NativeError::LibraryLoad { path, reason } => {
            assert_eq!(path, "test.so");
            assert_eq!(reason, "not found");
        }
        _ => panic!("Expected LibraryLoad variant"),
    }
}

#[test]
fn test_native_function_struct() {
    use hudhudscript_native::{NativeFunction, NativeType};

    // Create a function definition
    let func = NativeFunction {
        name: "add".to_string(),
        param_types: vec![NativeType::Int32, NativeType::Int32],
        return_type: NativeType::Int32,
    };

    assert_eq!(func.name, "add");
    assert_eq!(func.param_types.len(), 2);
    assert!(matches!(func.return_type, NativeType::Int32));
}

#[test]
fn test_native_callable_trait() {
    use hudhudscript_native::{NativeCallable, NativeValue};

    // Create a mock implementation
    struct MockCallable;

    impl NativeCallable for MockCallable {
        fn call_native(
            &mut self,
            library: &str,
            function: &str,
            args: Vec<NativeValue>,
        ) -> Result<NativeValue, String> {
            assert_eq!(library, "test");
            assert_eq!(function, "add");
            assert_eq!(args.len(), 2);
            Ok(NativeValue::Int32(42))
        }

        fn is_native_available(&self, library: &str, function: &str) -> bool {
            library == "test" && function == "add"
        }
    }

    let mut mock = MockCallable;
    let result = mock.call_native(
        "test",
        "add",
        vec![NativeValue::Int32(1), NativeValue::Int32(2)],
    );
    assert!(result.is_ok());
    assert!(mock.is_native_available("test", "add"));
    assert!(!mock.is_native_available("test", "sub"));
}

#[test]
fn test_config_build_type() {
    use hudhudscript_native::config::BuildType;

    let debug = BuildType::Debug;
    let release = BuildType::Release;

    match debug {
        BuildType::Debug => {}
        _ => panic!("Expected Debug"),
    }

    match release {
        BuildType::Release => {}
        _ => panic!("Expected Release"),
    }
}

#[test]
fn test_debug_implementations() {
    use hudhudscript_native::{NativeFunction, NativeRuntime, NativeType, NativeValue};

    // Test that Debug is implemented for key types
    let runtime = NativeRuntime::new();
    let _debug_runtime = format!("{:?}", runtime);

    let func = NativeFunction {
        name: "test".to_string(),
        param_types: vec![NativeType::Int32],
        return_type: NativeType::Void,
    };
    let _debug_func = format!("{:?}", func);

    let _debug_type = format!("{:?}", NativeType::Int32);
    let _debug_value = format!("{:?}", NativeValue::Int32(42));
}

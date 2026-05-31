use hudhudscript_bytecode::Value16;

/// Helper: call the shared temp method with interpreter Value
fn call_temp(method: &str, args: &[Value16]) -> Result<Value16, hudhudscript_errors::Error> {
    hudhudscript_shared_builtins::temp::dispatch(method, args)
}

#[test]
fn test_temp_file_creates() {
    let result = call_temp("file", &[]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("path"));
        if let Some(p) = obj.get("path").and_then(|v| v.as_str()) {
            assert!(!p.is_empty());
        }
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_temp_dir_creates() {
    let result = call_temp("dir", &[]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("path"));
        if let Some(p) = obj.get("path").and_then(|v| v.as_str()) {
            let path = std::path::Path::new(p);
            assert!(path.exists());
            assert!(path.is_dir());
        }
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_temp_path_returns_string() {
    let result = call_temp("path", &[]).unwrap();
    if let Some(s) = result.as_str() {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string, got {:?}", result);
    }
}

#[test]
fn test_temp_file_with_prefix() {
    let result = call_temp("file", &[Value16::string("myprefix".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("path"));
    } else {
        panic!("Expected object");
    }
}

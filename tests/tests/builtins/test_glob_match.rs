use hudhudscript_bytecode::Value16;

fn glob_find(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::glob_ops::dispatch("find", args)
}
fn glob_match(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::glob_ops::dispatch("match", args)
}

#[test]
fn test_glob_match_true() {
    let result = glob_match(&[
        Value16::string("hello.txt".to_string()),
        Value16::string("*.txt".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_glob_match_false() {
    let result = glob_match(&[
        Value16::string("hello.rs".to_string()),
        Value16::string("*.txt".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_glob_match_complex() {
    let result = glob_match(&[
        Value16::string("src/main.rs".to_string()),
        Value16::string("src/*.rs".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_glob_find_returns_array() {
    // Search for Cargo.toml in current workspace — should find at least one
    let result = glob_find(&[Value16::string("Cargo.toml".to_string())]);
    // This may or may not find files depending on cwd, so just check it doesn't error
    assert!(result.is_ok());
    if let Ok(v) = result {
        if v.as_array().is_some() {
            // ok
        } else {
            panic!("Expected array");
        }
    }
}

use hudhudscript_bytecode::Value16;

// Thin wrapper that pins the generic `V` to `Value` so the test bodies below
// stay byte-identical to the interpreter-era originals (Kural 1).
fn call_string_method(
    s: &str,
    method: &str,
    args: &[Value16],
) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::string::call_string_method(s, method, args, false)
}

#[test]
fn test_to_upper() {
    let result = call_string_method("hello", "toUpperCase", &[]).unwrap();
    assert_eq!(result, Value16::string("HELLO".to_string()));
}

#[test]
fn test_to_lower() {
    let result = call_string_method("HELLO", "toLowerCase", &[]).unwrap();
    assert_eq!(result, Value16::string("hello".to_string()));
}

#[test]
fn test_trim() {
    let result = call_string_method("  hello  ", "trim", &[]).unwrap();
    assert_eq!(result, Value16::string("hello".to_string()));
}

#[test]
fn test_contains() {
    let result = call_string_method(
        "hello world",
        "contains",
        &[Value16::string("world".to_string())],
    )
    .unwrap();
    assert_eq!(result, Value16::boolean(true));

    let result = call_string_method(
        "hello world",
        "contains",
        &[Value16::string("xyz".to_string())],
    )
    .unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_starts_with() {
    let result =
        call_string_method("hello", "startsWith", &[Value16::string("hel".to_string())]).unwrap();
    assert_eq!(result, Value16::boolean(true));

    let result =
        call_string_method("hello", "startsWith", &[Value16::string("xyz".to_string())]).unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_ends_with() {
    let result =
        call_string_method("hello", "endsWith", &[Value16::string("llo".to_string())]).unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_index_of() {
    let result = call_string_method(
        "hello world",
        "indexOf",
        &[Value16::string("world".to_string())],
    )
    .unwrap();
    assert_eq!(result, Value16::number(6.0));

    let result = call_string_method(
        "hello world",
        "indexOf",
        &[Value16::string("xyz".to_string())],
    )
    .unwrap();
    assert_eq!(result, Value16::number(-1.0));
}

#[test]
fn test_replace() {
    let result = call_string_method(
        "hello world",
        "replace",
        &[
            Value16::string("world".to_string()),
            Value16::string("rust".to_string()),
        ],
    )
    .unwrap();
    assert_eq!(result, Value16::string("hello rust".to_string()));
}

#[test]
fn test_split() {
    let result = call_string_method("a,b,c", "split", &[Value16::string(",".to_string())]).unwrap();
    assert_eq!(
        result,
        Value16::array(vec![
            Value16::string("a".to_string()),
            Value16::string("b".to_string()),
            Value16::string("c".to_string()),
        ])
    );
}

#[test]
fn test_substring() {
    let result = call_string_method(
        "hello",
        "substring",
        &[Value16::number(1.0), Value16::number(4.0)],
    )
    .unwrap();
    assert_eq!(result, Value16::string("ell".to_string()));
}

#[test]
fn test_slice_alias() {
    let result = call_string_method(
        "hello",
        "slice",
        &[Value16::number(0.0), Value16::number(3.0)],
    )
    .unwrap();
    assert_eq!(result, Value16::string("hel".to_string()));
}

#[test]
fn test_length() {
    let result = call_string_method("hello", "length", &[]).unwrap();
    assert_eq!(result, Value16::number(5.0));
}

#[test]
fn test_repeat() {
    let result = call_string_method("ab", "repeat", &[Value16::number(3.0)]).unwrap();
    assert_eq!(result, Value16::string("ababab".to_string()));
}

#[test]
fn test_unknown_method() {
    let result = call_string_method("hello", "nonExistent", &[]);
    assert!(result.is_err());
}

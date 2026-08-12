use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::regex_ops::RegexMethodId;
use hudhudscript_shared_builtins::string::call_string_method;

fn regex_test(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::Test, args)
}
fn regex_match(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::Match, args)
}
fn regex_find_all(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::FindAll, args)
}
fn regex_replace(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::Replace, args)
}
fn regex_replace_all(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::ReplaceAll, args)
}
fn regex_split(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::Split, args)
}
fn regex_escape(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::regex_ops::dispatch(RegexMethodId::Escape, args)
}
fn string_match(s: &str, args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    call_string_method(s, "match", args, false)
}
fn string_match_all(s: &str, args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    call_string_method(s, "match_all", args, false)
}
fn string_replace_regex(s: &str, args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    call_string_method(s, "replace_regex", args, false)
}

#[test]
fn test_regex_test() {
    let result = regex_test(&[
        Value16::string(r"\d+".to_string()),
        Value16::string("abc123def".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_regex_test_no_match() {
    let result = regex_test(&[
        Value16::string(r"\d+".to_string()),
        Value16::string("abcdef".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_regex_match() {
    let result = regex_match(&[
        Value16::string(r"(\d+)".to_string()),
        Value16::string("abc123def456".to_string()),
    ])
    .unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("matched"), Some(&Value16::boolean(true)));
        assert_eq!(obj.get("value"), Some(&Value16::string("123".to_string())));
        assert_eq!(obj.get("index"), Some(&Value16::number(3.0)));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_regex_find_all() {
    let result = regex_find_all(&[
        Value16::string(r"\d+".to_string()),
        Value16::string("a1b22c333".to_string()),
    ])
    .unwrap();
    if let Some(arr) = result.as_array() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_regex_replace() {
    let result = regex_replace(&[
        Value16::string(r"\d+".to_string()),
        Value16::string("abc123def".to_string()),
        Value16::string("NUM".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::string("abcNUMdef".to_string()));
}

#[test]
fn test_regex_replace_all() {
    let result = regex_replace_all(&[
        Value16::string(r"\d+".to_string()),
        Value16::string("a1b2c3".to_string()),
        Value16::string("N".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::string("aNbNcN".to_string()));
}

#[test]
fn test_regex_split() {
    let result = regex_split(&[
        Value16::string(r"[,;]".to_string()),
        Value16::string("a,b;c,d".to_string()),
    ])
    .unwrap();
    if let Some(arr) = result.as_array() {
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0], Value16::string("a".to_string()));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_regex_named_captures() {
    // Named captures are extracted via capture groups (positional).
    // The shared regex implementation returns groups as an array, not named object.
    let result = regex_match(&[
        Value16::string(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})".to_string()),
        Value16::string("Date: 2024-01-15".to_string()),
    ])
    .unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("matched"), Some(&Value16::boolean(true)),);
        assert_eq!(
            obj.get("value"),
            Some(&Value16::string("2024-01-15".to_string())),
        );
        // Groups are returned as a positional array: [year, month, day]
        if let Some(groups) = obj.get("groups").and_then(|v| v.as_array()) {
            assert_eq!(groups.len(), 3);
            assert_eq!(groups[0], Value16::string("2024".to_string()));
            assert_eq!(groups[1], Value16::string("01".to_string()));
            assert_eq!(groups[2], Value16::string("15".to_string()));
        } else {
            panic!("Expected groups array, got {:?}", obj.get("groups"));
        }
    } else {
        panic!("Expected object, got {:?}", result);
    }
}

#[test]
fn test_regex_case_insensitive() {
    let result = regex_test(&[
        Value16::string("hello".to_string()),
        Value16::string("HELLO WORLD".to_string()),
        Value16::string("i".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_regex_escape() {
    let result = regex_escape(&[Value16::string("a.b+c".to_string())]).unwrap();
    if let Some(escaped) = result.as_str() {
        assert!(escaped.contains(r"\."));
        assert!(escaped.contains(r"\+"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_string_match() {
    let result = string_match("hello world 42", &[Value16::string(r"\d+".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("value"), Some(&Value16::string("42".to_string())));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_string_match_all() {
    let result = string_match_all("a1b2c3", &[Value16::string(r"\d".to_string())]).unwrap();
    if let Some(arr) = result.as_array() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_string_replace_regex() {
    let result = string_replace_regex(
        "hello world",
        &[
            Value16::string(r"\w+".to_string()),
            Value16::string("X".to_string()),
        ],
    )
    .unwrap();
    assert_eq!(result, Value16::string("X X".to_string()));
}

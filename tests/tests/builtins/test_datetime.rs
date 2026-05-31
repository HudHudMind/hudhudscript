use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::date::DateMethodId;
use hudhudscript_shared_builtins::duration::DurationMethodId;

// Thin wrappers around the shared dispatchers so the test bodies below stay
// byte-for-byte identical to the interpreter-era originals (Kural 1: no
// semantic test changes — only mechanism).
fn date_now(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::Now, args)
}
fn date_parse(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::Parse, args)
}
fn date_format(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::Format, args)
}
fn date_from_timestamp(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::FromTimestamp, args)
}
fn date_diff(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::Diff, args)
}
fn date_add(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::Add, args)
}
fn date_iso(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::date::dispatch(DateMethodId::Iso, args)
}
fn duration_seconds(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::duration::dispatch(DurationMethodId::Seconds, args)
}
fn duration_hours(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::duration::dispatch(DurationMethodId::Hours, args)
}

#[test]
fn test_date_now() {
    let result = date_now(&[]).unwrap();
    if let Some(ts) = result.as_number() {
        assert!(ts > 1_700_000_000.0); // After Nov 2023
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_date_parse_iso() {
    let result = date_parse(&[Value16::string("2024-01-15T10:30:00+00:00".to_string())]).unwrap();
    if let Some(ts) = result.as_number() {
        assert!(ts > 0.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_date_parse_simple() {
    let result = date_parse(&[Value16::string("2024-01-15 10:30:00".to_string())]).unwrap();
    assert!(result.as_number().is_some());
}

#[test]
fn test_date_parse_date_only() {
    let result = date_parse(&[Value16::string("2024-01-15".to_string())]).unwrap();
    assert!(result.as_number().is_some());
}

#[test]
fn test_date_format() {
    // Parse a known date, then format it
    let ts = date_parse(&[Value16::string("2024-01-15 10:30:00".to_string())]).unwrap();
    if let Some(ts) = ts.as_number() {
        let formatted =
            date_format(&[Value16::number(ts), Value16::string("%Y-%m-%d".to_string())]).unwrap();
        assert_eq!(formatted, Value16::string("2024-01-15".to_string()));
    }
}

#[test]
fn test_date_from_timestamp() {
    let result = date_from_timestamp(&[Value16::number(0.0)]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("year"), Some(&Value16::number(1970.0)));
        assert_eq!(obj.get("month"), Some(&Value16::number(1.0)));
        assert_eq!(obj.get("day"), Some(&Value16::number(1.0)));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_date_diff() {
    let result = date_diff(&[
        Value16::number(3600.0),
        Value16::number(0.0),
        Value16::string("hours".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::number(1.0));
}

#[test]
fn test_date_add() {
    let result = date_add(&[
        Value16::number(0.0),
        Value16::number(1.0),
        Value16::string("days".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::number(86400.0));
}

#[test]
fn test_date_iso() {
    let result = date_iso(&[Value16::number(0.0)]).unwrap();
    if let Some(s) = result.as_str() {
        assert!(s.starts_with("1970-01-01"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_duration_seconds() {
    let result = duration_seconds(&[Value16::number(30.0)]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("seconds"), Some(&Value16::number(30.0)));
        assert_eq!(obj.get("millis"), Some(&Value16::number(30000.0)));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_duration_hours() {
    let result = duration_hours(&[Value16::number(2.0)]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("seconds"), Some(&Value16::number(7200.0)));
    } else {
        panic!("Expected object");
    }
}

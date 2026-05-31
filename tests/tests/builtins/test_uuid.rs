use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::hudhud_encoding::UuidMethodId;

#[test]
fn test_uuid_v4() {
    let result = UuidMethodId::V4.dispatch(&[]).unwrap();
    if let Some(s) = result.as_str() {
        assert_eq!(s.len(), 36); // UUID format: 8-4-4-4-12
        assert!(uuid::Uuid::parse_str(&s).is_ok());
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_uuid_v7() {
    let result = UuidMethodId::V7.dispatch(&[]).unwrap();
    if let Some(s) = result.as_str() {
        assert_eq!(s.len(), 36);
        let parsed = uuid::Uuid::parse_str(&s).unwrap();
        assert_eq!(parsed.get_version_num(), 7);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_uuid_parse() {
    let input = "550e8400-e29b-41d4-a716-446655440000";
    let result = UuidMethodId::Parse
        .dispatch(&[Value16::string(input.to_string())])
        .unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("value"), Some(&Value16::string(input.to_string())));
        assert_eq!(obj.get("version"), Some(&Value16::number(4.0)));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_uuid_parse_invalid() {
    let result = UuidMethodId::Parse.dispatch(&[Value16::string("not-a-uuid".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_uuid_nil() {
    let result = UuidMethodId::Nil.dispatch(&[]).unwrap();
    assert_eq!(
        result,
        Value16::string("00000000-0000-0000-0000-000000000000".to_string())
    );
}

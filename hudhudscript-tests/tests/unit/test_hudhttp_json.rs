//! Tests for hudhud-http JSON stringify control-character escaping
//! (moved from src/json.rs).

use hudhud_http::json::value_to_json_string;
use hudhudscript_bytecode::{ObjMap, Value16};

#[test]
fn stringify_escapes_control_characters_and_object_keys() {
    let original = "line one\nline two\t\u{0000}";
    let text = value_to_json_string(&Value16::string(original.to_string()));
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON string");
    assert_eq!(parsed.as_str(), Some(original));

    let mut object = ObjMap::default();
    object.insert("key\nname".to_string(), Value16::string("value\r\n".to_string()));
    let object_text = value_to_json_string(&Value16::object(object));
    let parsed_object: serde_json::Value = serde_json::from_str(&object_text).expect("valid JSON object");
    assert_eq!(parsed_object["key\nname"], "value\r\n");
}

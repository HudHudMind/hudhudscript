use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::csv_ops::CsvMethodId;
use hudhudscript_shared_builtins::ini_ops::IniMethodId;
use hudhudscript_shared_builtins::toml_ops::TomlMethodId;
use hudhudscript_shared_builtins::yaml_ops::YamlMethodId;

fn toml_parse(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    TomlMethodId::Parse.dispatch(args)
}
fn toml_stringify(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    TomlMethodId::Stringify.dispatch(args)
}
fn yaml_parse(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    YamlMethodId::Parse.dispatch(args)
}
fn yaml_stringify(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    YamlMethodId::Stringify.dispatch(args)
}
fn csv_parse(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    CsvMethodId::Parse.dispatch(args)
}
fn csv_stringify(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    CsvMethodId::Stringify.dispatch(args)
}
fn ini_parse(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    IniMethodId::Parse.dispatch(args)
}
fn ini_stringify(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    IniMethodId::Stringify.dispatch(args)
}

#[test]
fn test_toml_roundtrip() {
    let input = "[server]\nport = 8080\nhost = \"localhost\"\n";
    let parsed = toml_parse(&[Value16::string(input.to_string())]).unwrap();
    if let Some(obj) = &parsed.as_object() {
        if let Some(server) = obj.get("server").and_then(|v| v.as_object()) {
            assert_eq!(server.get("port"), Some(&Value16::number(8080.0)));
            assert_eq!(
                server.get("host"),
                Some(&Value16::string("localhost".to_string()))
            );
        } else {
            panic!("Expected server section");
        }
    } else {
        panic!("Expected object");
    }

    let stringified = toml_stringify(&[parsed]).unwrap();
    assert!(stringified.as_str().is_some());
}

#[test]
fn test_yaml_roundtrip() {
    let input = "name: hello\nport: 8080\n";
    let parsed = yaml_parse(&[Value16::string(input.to_string())]).unwrap();
    if let Some(obj) = &parsed.as_object() {
        assert_eq!(obj.get("name"), Some(&Value16::string("hello".to_string())));
        assert_eq!(obj.get("port"), Some(&Value16::number(8080.0)));
    } else {
        panic!("Expected object");
    }

    let stringified = yaml_stringify(&[parsed]).unwrap();
    assert!(stringified.as_str().is_some());
}

#[test]
fn test_csv_roundtrip() {
    let input = "name,age\nAli,30\nVeli,25\n";
    let parsed = csv_parse(&[Value16::string(input.to_string())]).unwrap();
    if let Some(rows) = &parsed.as_array() {
        assert_eq!(rows.len(), 2);
        if let Some(row) = &rows[0].as_object() {
            assert_eq!(row.get("name"), Some(&Value16::string("Ali".to_string())));
            assert_eq!(row.get("age"), Some(&Value16::number(30.0)));
        }
    } else {
        panic!("Expected array");
    }

    let stringified = csv_stringify(&[parsed]).unwrap();
    assert!(stringified.as_str().is_some());
}

#[test]
fn test_ini_roundtrip() {
    let input = "[database]\nhost = localhost\nport = 5432\n";
    let parsed = ini_parse(&[Value16::string(input.to_string())]).unwrap();
    if let Some(obj) = &parsed.as_object() {
        if let Some(db) = obj.get("database").and_then(|v| v.as_object()) {
            assert_eq!(
                db.get("host"),
                Some(&Value16::string("localhost".to_string()))
            );
            assert_eq!(db.get("port"), Some(&Value16::number(5432.0)));
        } else {
            panic!("Expected database section");
        }
    } else {
        panic!("Expected object");
    }

    let stringified = ini_stringify(&[parsed]).unwrap();
    assert!(stringified.as_str().is_some());
}

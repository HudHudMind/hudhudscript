//! Integration tests for v0.4.38 Batch 1: Serialization & Encoding
//!
//! Tests TOML, YAML, CSV, INI parse/stringify, Base64/Hex/URL encode/decode,
//! and UUID generation in both interpreter and VM paths.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(code: &str) -> Interpreter {
    let ast = parse(code).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
    interp
}

fn run_and_get(code: &str, var: &str) -> hudhudscript_bytecode::Value16 {
    let interp = run(code);
    interp
        .get_variable(var)
        .expect(&format!("variable '{}' not found", var))
}

// ─── TOML ────────────────────────────────────────────────────────────────────

#[test]
fn test_toml_parse() {
    let code = r#"
        var text = "[server]\nport = 8080\nhost = \"localhost\"";
        var obj = TOML.parse(text);
        var port = obj.server.port;
        var host = obj.server.host;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("port").unwrap(),
        hudhudscript_bytecode::Value16::number(8080.0)
    );
    assert_eq!(
        interp.get_variable("host").unwrap(),
        hudhudscript_bytecode::Value16::string("localhost".to_string())
    );
}

#[test]
fn test_toml_stringify() {
    let code = r#"
        var obj = { name: "test", version: 1 };
        var text = TOML.stringify(obj);
    "#;
    let val = run_and_get(code, "text");
    if let Some(s) = val.as_str() {
        assert!(s.contains("name"));
        assert!(s.contains("test"));
    } else {
        panic!("Expected string");
    }
}

// ─── YAML ────────────────────────────────────────────────────────────────────

#[test]
fn test_yaml_parse() {
    let code = r#"
        var text = "name: hello\nport: 8080";
        var obj = YAML.parse(text);
        var name = obj.name;
        var port = obj.port;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("name").unwrap(),
        hudhudscript_bytecode::Value16::string("hello".to_string())
    );
    assert_eq!(
        interp.get_variable("port").unwrap(),
        hudhudscript_bytecode::Value16::number(8080.0)
    );
}

#[test]
fn test_yaml_stringify() {
    let code = r#"
        var obj = { key: "value", num: 42 };
        var text = YAML.stringify(obj);
    "#;
    let val = run_and_get(code, "text");
    if let Some(s) = val.as_str() {
        assert!(s.contains("key"));
        assert!(s.contains("value"));
    } else {
        panic!("Expected string");
    }
}

// ─── CSV ─────────────────────────────────────────────────────────────────────

#[test]
fn test_csv_parse() {
    let code = r#"
        var text = "name,age\nAli,30\nVeli,25";
        var rows = CSV.parse(text);
        var first_name = rows[0].name;
        var first_age = rows[0].age;
        var count = len(rows);
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("first_name").unwrap(),
        hudhudscript_bytecode::Value16::string("Ali".to_string())
    );
    assert_eq!(
        interp.get_variable("first_age").unwrap(),
        hudhudscript_bytecode::Value16::number(30.0)
    );
    assert_eq!(
        interp.get_variable("count").unwrap(),
        hudhudscript_bytecode::Value16::number(2.0)
    );
}

#[test]
fn test_csv_stringify() {
    let code = r#"
        var data = [{ name: "Ali", age: 30 }, { name: "Veli", age: 25 }];
        var text = CSV.stringify(data);
    "#;
    let val = run_and_get(code, "text");
    if let Some(s) = val.as_str() {
        assert!(s.contains("Ali"));
        assert!(s.contains("Veli"));
        assert!(s.contains("30"));
    } else {
        panic!("Expected string");
    }
}

// ─── INI ─────────────────────────────────────────────────────────────────────

#[test]
fn test_ini_parse() {
    let code = r#"
        var text = "[database]\nhost = localhost\nport = 5432";
        var obj = INI.parse(text);
        var host = obj.database.host;
        var port = obj.database.port;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("host").unwrap(),
        hudhudscript_bytecode::Value16::string("localhost".to_string())
    );
    assert_eq!(
        interp.get_variable("port").unwrap(),
        hudhudscript_bytecode::Value16::number(5432.0)
    );
}

#[test]
fn test_ini_stringify() {
    let code = r#"
        var obj = { database: { host: "localhost", port: 5432 } };
        var text = INI.stringify(obj);
    "#;
    let val = run_and_get(code, "text");
    if let Some(s) = val.as_str() {
        assert!(s.contains("[database]"));
        assert!(s.contains("host"));
        assert!(s.contains("localhost"));
    } else {
        panic!("Expected string");
    }
}

// ─── Base64 ──────────────────────────────────────────────────────────────────

#[test]
fn test_base64_encode_decode() {
    let code = r#"
        var encoded = Base64.encode("Hello World");
        var decoded = Base64.decode(encoded);
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("encoded").unwrap(),
        hudhudscript_bytecode::Value16::string("SGVsbG8gV29ybGQ=".to_string())
    );
    assert_eq!(
        interp.get_variable("decoded").unwrap(),
        hudhudscript_bytecode::Value16::string("Hello World".to_string())
    );
}

// ─── Hex ─────────────────────────────────────────────────────────────────────

#[test]
fn test_hex_encode_decode() {
    let code = r#"
        var encoded = Hex.encode("Hello");
        var decoded = Hex.decode(encoded);
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("encoded").unwrap(),
        hudhudscript_bytecode::Value16::string("48656c6c6f".to_string())
    );
    assert_eq!(
        interp.get_variable("decoded").unwrap(),
        hudhudscript_bytecode::Value16::string("Hello".to_string())
    );
}

// ─── URL encoding ────────────────────────────────────────────────────────────

#[test]
fn test_url_encode_decode() {
    let code = r#"
        var encoded = URL.encode("hello world&foo=bar");
        var decoded = URL.decode(encoded);
    "#;
    let interp = run(code);
    let encoded = interp.get_variable("encoded").unwrap();
    if let Some(s) = encoded.as_str() {
        assert!(s.contains("%20") || s.contains("+"));
    }
    assert_eq!(
        interp.get_variable("decoded").unwrap(),
        hudhudscript_bytecode::Value16::string("hello world&foo=bar".to_string())
    );
}

// ─── UUID ────────────────────────────────────────────────────────────────────

#[test]
fn test_uuid_v4_generation() {
    let code = r#"
        var id = uuid.v4();
    "#;
    let val = run_and_get(code, "id");
    if let Some(s) = val.as_str() {
        assert_eq!(s.len(), 36, "UUID should be 36 chars: {}", s);
        assert_eq!(s.chars().filter(|c| *c == '-').count(), 4);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_uuid_v7_generation() {
    let code = r#"
        var id = uuid.v7();
    "#;
    let val = run_and_get(code, "id");
    if let Some(s) = val.as_str() {
        assert_eq!(s.len(), 36);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_uuid_nil() {
    let code = r#"
        var id = uuid.nil();
    "#;
    let val = run_and_get(code, "id");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("00000000-0000-0000-0000-000000000000".to_string())
    );
}

#[test]
fn test_uuid_parse() {
    let code = r#"
        var parsed = uuid.parse("550e8400-e29b-41d4-a716-446655440000");
        var version = parsed.version;
        var value = parsed.value;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("version").unwrap(),
        hudhudscript_bytecode::Value16::number(4.0)
    );
    assert_eq!(
        interp.get_variable("value").unwrap(),
        hudhudscript_bytecode::Value16::string("550e8400-e29b-41d4-a716-446655440000".to_string())
    );
}

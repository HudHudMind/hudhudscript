//! VM parity tests for v0.4.38 Batch 1: Serialization & Encoding

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn vm_run_and_get(code: &str, var: &str) -> (hudhudscript_vm::VM, hudhudscript_bytecode::Value16) {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
    let val = vm
        .get_variable(var)
        .cloned()
        .map(|v| v)
        .unwrap_or_else(|| panic!("variable \'{}\' not found", var));
    (vm, val)
}

fn assert_string(
    (_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16),
    expected: &str,
) {
    if let Some(s) = val.as_string() {
        assert_eq!(s, expected, "Expected '{}', got '{}'", expected, s)
    } else {
        panic!("Expected String(\"{}\"), got {:?}", expected, val);
    }
}

fn unwrap_string((_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16)) -> String {
    if let Some(s) = val.as_string() {
        s
    } else {
        panic!("Expected String, got {:?}", val);
    }
}

// ─── Base64 ──────────────────────────────────────────────────────────────────

#[test]
fn test_vm_base64_encode() {
    assert_string(
        vm_run_and_get(r#"var x = Base64.encode("Hello");"#, "x"),
        "SGVsbG8=",
    );
}

#[test]
fn test_vm_base64_decode() {
    assert_string(
        vm_run_and_get(r#"var x = Base64.decode("SGVsbG8=");"#, "x"),
        "Hello",
    );
}

// ─── Hex ─────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_hex_encode() {
    assert_string(vm_run_and_get(r#"var x = Hex.encode("Hi");"#, "x"), "4869");
}

#[test]
fn test_vm_hex_decode() {
    assert_string(vm_run_and_get(r#"var x = Hex.decode("4869");"#, "x"), "Hi");
}

// ─── URL ─────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_url_encode() {
    let s = unwrap_string(vm_run_and_get(r#"var x = URL.encode("hello world");"#, "x"));
    assert!(s.contains("%20"), "Expected URL-encoded, got '{}'", s);
}

#[test]
fn test_vm_url_decode() {
    assert_string(
        vm_run_and_get(r#"var x = URL.decode("hello%20world");"#, "x"),
        "hello world",
    );
}

// ─── UUID ────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_uuid_nil() {
    assert_string(
        vm_run_and_get(r#"var x = uuid.nil();"#, "x"),
        "00000000-0000-0000-0000-000000000000",
    );
}

#[test]
fn test_vm_uuid_v4() {
    let s = unwrap_string(vm_run_and_get(r#"var x = uuid.v4();"#, "x"));
    assert_eq!(s.len(), 36, "UUID should be 36 chars: {}", s);
}

// ─── TOML ────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_toml_parse() {
    assert_string(
        vm_run_and_get(
            r#"var x = TOML.parse("key = \"value\""); var y = x.key;"#,
            "y",
        ),
        "value",
    );
}

// ─── YAML ────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_yaml_parse() {
    assert_string(
        vm_run_and_get(r#"var x = YAML.parse("name: test"); var y = x.name;"#, "y"),
        "test",
    );
}

// ─── INI ─────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_ini_parse() {
    assert_string(
        vm_run_and_get(
            r#"var x = INI.parse("[db]\nhost = localhost"); var y = x.db.host;"#,
            "y",
        ),
        "localhost",
    );
}

// ─── CSV ─────────────────────────────────────────────────────────────────────

#[test]
fn test_vm_csv_parse() {
    assert_string(
        vm_run_and_get(
            r#"var x = CSV.parse("name,age\nAli,30"); var y = x[0].name;"#,
            "y",
        ),
        "Ali",
    );
}

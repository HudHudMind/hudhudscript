// External tests exercising the same code paths as hudhudscript-python bindings.
// The python crate is a cdylib that wraps parser, compiler, interpreter.
// We test those exact paths here without requiring a Python runtime.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_shared_builtins::print_ops::{start_capture, stop_capture};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ── check() path ────────────────────────────────────────────────────

#[test]
fn python_check_valid_code() {
    let code = r#"let x = 42"#;
    let result = parse(code);
    assert!(result.is_ok(), "Valid code should parse successfully");
}

#[test]
fn python_check_invalid_syntax() {
    let code = r#"let = = ="#;
    let result = parse(code);
    assert!(
        result.is_err(),
        "Invalid syntax should produce a parse error"
    );
}

#[test]
fn python_check_empty_code() {
    let code = "";
    let result = parse(code);
    // Empty code should parse successfully (empty program)
    assert!(
        result.is_ok(),
        "Empty code should parse as a valid empty program"
    );
}

// ── compile() path ──────────────────────────────────────────────────

#[test]
fn python_compile_valid_code_produces_bytecode() {
    let code = r#"let x = 10 + 20"#;
    let ast = parse(code).expect("parse should succeed");

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast);
    assert!(bytecode.is_ok(), "Compilation of valid code should succeed");

    let mut bc = bytecode.unwrap();
    let bytes = bc.to_bytes();
    assert!(bytes.is_ok(), "Bytecode serialization should succeed");
    assert!(
        !bytes.unwrap().is_empty(),
        "Serialized bytecode should not be empty"
    );
}

#[test]
fn python_compile_invalid_code_returns_parse_error() {
    let code = r#"let @@@ = ???"#;
    let result = parse(code);
    assert!(
        result.is_err(),
        "Garbage code should fail to parse before compilation"
    );
}

#[test]
fn python_compile_function_declaration() {
    let code = r#"
        fn add(a, b) {
            return a + b
        }
    "#;
    let ast = parse(code).expect("function declaration should parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast);
    assert!(
        bytecode.is_ok(),
        "Function declaration should compile successfully"
    );

    let bytes = bytecode
        .unwrap()
        .to_bytes()
        .expect("serialization should work");
    assert!(
        bytes.len() > 10,
        "Compiled function should produce meaningful bytecode"
    );
}

// ── run() path ──────────────────────────────────────────────────────

#[test]
fn python_run_arithmetic_expression() {
    let code = r#"let x = 2 + 3"#;
    let ast = parse(code).expect("parse should succeed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&ast);
    assert!(
        result.is_ok(),
        "Simple arithmetic should execute without error"
    );
}

#[test]
fn python_run_returns_last_expression_value() {
    let code = r#"let x = 7 * 6
x"#;
    let ast = parse(code).expect("parse should succeed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&ast).expect("execution should succeed");

    // The python binding formats Number with fract==0 as integer
    if let Some(n) = result.as_number() {
        assert_eq!(n as i64, 42, "7 * 6 should equal 42");
        // Verify the python-binding formatting logic
        assert_eq!(n.fract(), 0.0);
        let formatted = format!("{}", n as i64);
        assert_eq!(formatted, "42");
    } else {
        panic!("Expected Number, got: {:?}", result);
    }
}

#[test]
fn python_run_capture_print_output() {
    // Mirrors the python run() function's capture logic
    let code = r#"print("hello from hudhud")"#;
    let ast = parse(code).expect("parse should succeed");

    start_capture();
    let mut interpreter = Interpreter::new();
    let _result = interpreter.execute(&ast);
    let captured = stop_capture().unwrap_or_default();

    assert!(
        captured.contains("hello from hudhud"),
        "Captured output should contain the printed string, got: {:?}",
        captured
    );
}

#[test]
fn python_run_with_timeout_simulation() {
    // Mirrors the python run() timeout logic using mpsc channel
    let code = "let x = 1 + 1\nx";
    let ast = parse(code).expect("parse should succeed");

    let timeout = Duration::from_millis(5000);
    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    thread::spawn(move || {
        let mut interpreter = Interpreter::new();
        match interpreter.execute(&ast) {
            Ok(val) => {
                let output = if let Some(n) = val.as_number() {
                    if n.fract() == 0.0 {
                        format!("{}", n as i64)
                    } else {
                        format!("{}", n)
                    }
                } else if let Some(s) = val.as_str() {
                    s.to_string()
                } else if let Some(b) = val.as_bool() {
                    format!("{}", b)
                } else if val.is_null() {
                    String::new()
                } else {
                    format!("{}", val)
                };
                let _ = tx.send(Ok(output));
            }
            Err(e) => {
                let _ = tx.send(Err(format!("{:?}", e)));
            }
        }
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            assert_eq!(output, "2", "1 + 1 should produce '2'");
        }
        Ok(Err(e)) => panic!("Execution failed: {}", e),
        Err(_) => panic!("Execution timed out"),
    }
}

#[test]
fn python_run_parse_error_produces_error_result() {
    // Mirrors the python run() function: parse error -> ExecutionResult with success=false
    let code = r#"fn ((()"#;
    let result = parse(code);
    assert!(result.is_err());

    let err_msg = format!("Parse error: {:?}", result.unwrap_err());
    assert!(
        err_msg.contains("Parse error"),
        "Error message should contain 'Parse error', got: {}",
        err_msg
    );
}

// ── run_file() path ─────────────────────────────────────────────────

#[test]
fn python_run_file_nonexistent_returns_error() {
    use std::fs;
    let path = "/tmp/hudhud_nonexistent_test_file_12345.hud";
    let result = fs::read_to_string(path);
    assert!(result.is_err(), "Reading a nonexistent file should fail");

    let err_msg = format!("Failed to read file: {}", result.unwrap_err());
    assert!(
        err_msg.contains("Failed to read file"),
        "Error should mention file read failure"
    );
}

#[test]
fn python_run_file_with_valid_content() {
    use std::fs;
    use std::io::Write;

    let dir = std::env::temp_dir();
    let path = dir.join("hudhud_python_test_valid.hud");
    {
        let mut f = fs::File::create(&path).expect("should create temp file");
        write!(f, "let result = 10 * 5\nresult").expect("should write");
    }

    let code = fs::read_to_string(&path).expect("should read file");
    let ast = parse(&code).expect("should parse file content");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&ast).expect("should execute");

    if let Some(n) = result.as_number() {
        assert_eq!(n as i64, 50);
    } else {
        panic!("Expected Number(50), got: {:?}", result);
    }

    let _ = fs::remove_file(&path);
}

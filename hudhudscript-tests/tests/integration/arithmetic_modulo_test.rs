use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhud_script_tests::vm_interpreter::{runtime_codes, RuntimeError};
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn eval_expr(source: &str) -> Result<Value16, RuntimeError> {
    let statements =
        parse(source).map_err(|e| runtime_codes::custom(format!("Parse error: {:?}", e)))?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements)
}

#[test]
fn modulo_by_zero_throws_error() {
    let result = eval_expr("5 % 0");
    match result {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("zero")
                    || msg.contains("Zero")
                    || msg.contains("divis")
                    || msg.contains("DivisionByZero"),
                "Expected division by zero error, got: {}",
                msg
            );
        }
        Ok(val) => panic!("Expected DivisionByZero error, got value: {:?}", val),
    }
}

#[test]
fn modulo_zero_by_zero_throws_error() {
    let result = eval_expr("0 % 0");
    match result {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("zero")
                    || msg.contains("Zero")
                    || msg.contains("divis")
                    || msg.contains("DivisionByZero"),
                "Expected division by zero error, got: {}",
                msg
            );
        }
        Ok(val) => panic!("Expected DivisionByZero error, got value: {:?}", val),
    }
}

#[test]
fn modulo_negative_by_zero_throws_error() {
    let result = eval_expr("-3 % 0");
    match result {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("zero")
                    || msg.contains("Zero")
                    || msg.contains("divis")
                    || msg.contains("DivisionByZero"),
                "Expected division by zero error, got: {}",
                msg
            );
        }
        Ok(val) => panic!("Expected DivisionByZero error, got value: {:?}", val),
    }
}

#[test]
fn modulo_normal_operation() {
    let result = eval_expr("10 % 3");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value16::number(1.0));
}

#[test]
fn modulo_exact_division() {
    let result = eval_expr("9 % 3");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value16::number(0.0));
}

#[test]
fn modulo_negative_dividend() {
    let result = eval_expr("-7 % 3");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let val = result.unwrap();
    if let Some(n) = val.as_number() {
        // In Rust, -7 % 3 = -1 (truncated toward zero)
        // But different languages have different conventions
        // Just verify it's a valid result
        assert!(n.abs() < 3.0, "Remainder should be less than divisor");
        // Verify: dividend == divisor * quotient + remainder
        let quotient: f64 = (-7.0) / 3.0;
        let expected = -7.0 - (3.0 * quotient.trunc());
        assert!(
            (n - expected).abs() < 0.0001,
            "Expected ~{}, got {}",
            expected,
            n
        );
    } else {
        panic!("Expected number")
    }
}

#[test]
fn modulo_negative_divisor() {
    let result = eval_expr("7 % -3");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let val = result.unwrap();
    if let Some(n) = val.as_number() {
        assert!(n.abs() < 3.0, "Remainder should be less than divisor");
    } else {
        panic!("Expected number")
    }
}

#[test]
fn modulo_both_negative() {
    let result = eval_expr("-7 % -3");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn modulo_decimal_numbers() {
    let result = eval_expr("5.5 % 2.0");
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let val = result.unwrap();
    if let Some(n) = val.as_number() {
        // 5.5 / 2.0 = 2.75, so remainder should be around 1.5
        let expected = 5.5 % 2.0; // Rust behavior
        assert!(
            (n - expected).abs() < 0.0001,
            "Expected ~{}, got {}",
            expected,
            n
        );
    } else {
        panic!("Expected number")
    }
}

#[test]
fn modulo_type_error_on_string() {
    let result = eval_expr("\"hello\" % 2");
    assert!(result.is_err(), "Expected type error");
}

#[test]
fn modulo_type_error_on_array() {
    let result = eval_expr("[1,2,3] % 2");
    assert!(result.is_err(), "Expected type error");
}

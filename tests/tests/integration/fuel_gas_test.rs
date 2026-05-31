//! Tests for Issue #94: Bounded Execution (Fuel/Gas Model) — VM-backed
//! after the interpreter retirement.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run_with_fuel(src: &str, fuel: u64) -> Result<Value16, String> {
    let ast = parse(src).expect("parse error");
    let mut interp = Interpreter::new().with_fuel(fuel);
    interp.execute(&ast).map_err(|e| e.to_string())
}

fn run_unlimited(src: &str) -> Value16 {
    let ast = parse(src).expect("parse error");
    let mut interp = Interpreter::new();
    interp.execute(&ast).expect("runtime error")
}

// ── Basic fuel consumption ────────────────────────────────────────────────────

#[test]
fn test_single_stmt_within_fuel() {
    let result = run_with_fuel("let x = 1;", 10);
    assert!(result.is_ok(), "should succeed with plenty of fuel");
}

#[test]
fn test_out_of_gas_single_stmt() {
    let result = run_with_fuel("let x = 1;", 0);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("gas") || err.contains("limit"),
        "expected OutOfGas error, got: {}",
        err
    );
}

#[test]
fn test_multiple_stmts_within_fuel() {
    let src = "let a = 1; let b = 2; let c = 3;";
    let result = run_with_fuel(src, 100);
    assert!(result.is_ok());
}

#[test]
fn test_out_of_gas_mid_program() {
    let src = "let a = 1; let b = 2; let c = 3;";
    let result = run_with_fuel(src, 2);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("gas") || err.contains("limit"),
        "expected OutOfGas, got: {}",
        err
    );
}

// ── Loop runaway prevention ───────────────────────────────────────────────────

#[test]
fn test_loop_stopped_by_fuel() {
    let src = r#"
        let i = 0;
        while (i < 1000000) {
            i = i + 1;
        }
    "#;
    let result = run_with_fuel(src, 50);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("gas") || err.contains("limit"),
        "expected OutOfGas, got: {}",
        err
    );
}

#[test]
fn test_loop_completes_within_fuel() {
    let src = r#"
        let i = 0;
        while (i < 3) {
            i = i + 1;
        }
    "#;
    let result = run_with_fuel(src, 200);
    assert!(result.is_ok(), "small loop should complete: {:?}", result);
}

// ── Unlimited mode (no fuel set) ─────────────────────────────────────────────

#[test]
fn test_unlimited_mode_no_error() {
    let val = run_unlimited("let x = 42;");
    let _ = val;
}

// ── with_fuel builder and remaining_fuel ─────────────────────────────────────

#[test]
fn test_remaining_fuel_decreases() {
    let ast = parse("let x = 1; let y = 2;").expect("parse error");
    let mut interp = Interpreter::new().with_fuel(100);
    assert_eq!(interp.remaining_fuel(), Some(100));
    interp.execute(&ast).expect("runtime error");
    let remaining = interp.remaining_fuel().unwrap();
    assert!(remaining < 100, "fuel should have been consumed");
}

#[test]
fn test_reset_fuel() {
    let ast = parse("let x = 1;").expect("parse error");
    let mut interp = Interpreter::new().with_fuel(50);
    interp.execute(&ast).expect("runtime error");
    let after_first = interp.remaining_fuel().unwrap();
    interp.reset_fuel();
    assert_eq!(
        interp.remaining_fuel(),
        Some(50),
        "fuel should reset to limit"
    );
    assert!(after_first < 50);
}

#[test]
fn test_no_fuel_set_remaining_is_none() {
    let interp = Interpreter::new();
    assert_eq!(interp.remaining_fuel(), None);
}

//! EXCEPTION.md 9.1-9.7 acceptance tests — permanent VM integration tests
//!
//! Tests compile and execute real HudHudScript through parser -> compiler -> VM.
//! Also tests exception() constructor, istisna alias, and throw normalization.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> Result<(), String> {
    let mut vm = VM::with_locale(hudhudscript_vm::OutputLocale::Default);
    let ast = parse(source).map_err(|e| format!("parse: {}", e))?;
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("compile: {}", e))?;
    vm.execute(&bytecode).map_err(|e| format!("{}", e))
}

fn run_expect_err(source: &str, expected: &str) {
    match run(source) {
        Ok(()) => panic!("expected error containing '{}', got success", expected),
        Err(e) => assert!(
            e.contains(expected),
            "error '{}' did not contain '{}'",
            e,
            expected
        ),
    }
}

// ─── 9.1: throw string -> catch has exception fields ──────────

#[test]
fn test_9_1_throw_string_catch_fields() {
    run(
        r#"try { throw "boom"; } catch (e) { let c = e.code; let t = e.title; let d = e.description; }"#,
    ).unwrap();
}

// ─── 9.2: exception throw -> preserved ────────────────────────

#[test]
fn test_9_2_exception_throw_preserved() {
    run(
        r#"let ex = exception("E_CUSTOM", "custom", "bad input"); try { throw ex; } catch (e) { let c = e.code; let t = e.title; let d = e.description; }"#,
    ).unwrap();
}

// ─── 9.3: try/finally normal ──────────────────────────────────

#[test]
fn test_9_3_try_finally_normal() {
    run(
        r#"let r = ""; try { r = r + "try"; } finally { r = r + ":finally"; } let ok = r + "|done";"#,
    ).unwrap();
}

// ─── 9.4: try/catch/finally throw ─────────────────────────────

#[test]
fn test_9_4_try_catch_finally_throw() {
    run(r#"
        let r = "";
        try {
            r = r + "try";
            throw "x";
            r = r + "try_end";
        } catch (err) {
            r = r + ":catch:" + err.description;
        } finally {
            r = r + ":finally";
        }
        let ok = r + "|done";
    "#)
    .unwrap();
}

// ─── 9.5: try/finally uncaught throw ──────────────────────────

#[test]
fn test_9_5_try_finally_uncaught_throw() {
    run_expect_err(
        r#"try { throw "boom"; } finally { let x = 1; }"#,
        "Uncaught exception",
    );
}

// ─── 9.6: catch rethrow nested ────────────────────────────────

#[test]
fn test_9_6_catch_rethrow_nested() {
    run(r#"
        let r = "";
        try {
            try {
                throw "inner";
            } catch (e) {
                r = r + "caught:" + e.description;
                throw e;
            } finally {
                r = r + ":inner_finally";
            }
        } catch (outer) {
            r = r + ":outer:" + outer.description;
        }
        let ok = r + "|done";
    "#)
    .unwrap();
}

// ─── 9.7: runtime error catch -> exception object ─────────────

#[test]
fn test_9_7_runtime_error_catch_has_fields() {
    run(
        r#"try { let n = null; let v = n.field; } catch (e) { let c = e.code; let t = e.title; let d = e.description; }"#,
    ).unwrap();
}

#[test]
fn test_9_7_runtime_error_catch_not_string() {
    // Before fix: catch received a string, e.code would fail.
    // After fix: catch receives exception object with .code field.
    run(r#"try { let n = null; let v = n.field; } catch (e) { let c = e.code; }"#).unwrap();
}

// ─── throw normalization ──────────────────────────────────────

#[test]
fn test_throw_null_normalizes() {
    run(r#"try { throw null; } catch (e) { let c = e.code; }"#).unwrap();
}

#[test]
fn test_throw_number_normalizes() {
    run(r#"try { throw 42; } catch (e) { let c = e.code; let d = e.description; }"#).unwrap();
}

// ─── exception() constructor ──────────────────────────────────

#[test]
fn test_exception_constructor_full() {
    run(r#"let e = exception("E_TEST", "test_title", "test_desc", "val"); let c = e.code; let t = e.title; let d = e.description;"#).unwrap();
}

#[test]
fn test_exception_constructor_minimal() {
    run(r#"let e = exception("E_MIN", "minimal"); let c = e.code; let t = e.title;"#).unwrap();
}

// ─── istisna alias ─────────────────────────────────────────────

#[test]
fn test_istisna_alias() {
    run(r#"let e = istisna("E_TR", "baslik", "aciklama"); let c = e.code; let t = e.title;"#)
        .unwrap();
}

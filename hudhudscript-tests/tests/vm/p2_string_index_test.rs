//! P2-A1a regression: ASCII string index fast path.
//! Verifies ASCII direct path and non-ASCII fallback produce correct results.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn str_index(src: &str, idx: &str) -> String {
    let script = format!("let x = \"{}\"; let r = x[{}]; r;", src, idx);
    let ast = parse(&script).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm.get_global("r")
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

// ── ASCII path ──────────────────────────────────────────────

#[test]
fn p2_ascii_index_first() {
    assert_eq!(str_index("abc", "0"), "a");
}

#[test]
fn p2_ascii_index_middle() {
    assert_eq!(str_index("abc", "1"), "b");
}

#[test]
fn p2_ascii_index_last() {
    assert_eq!(str_index("abc", "2"), "c");
}

// ── Non-ASCII fallback ──────────────────────────────────────

#[test]
fn p2_nonascii_index_first() {
    assert_eq!(str_index("ğa", "0"), "ğ");
}

#[test]
fn p2_nonascii_index_second() {
    assert_eq!(str_index("ğa", "1"), "a");
}

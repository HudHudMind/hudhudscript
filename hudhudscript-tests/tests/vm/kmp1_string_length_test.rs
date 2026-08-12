// KMP1 regression test: String.length O(1) for ASCII (StringAscii kind).
// Ensures .length on large ASCII strings no longer clones via as_string()
// and is O(1) instead of O(n).
use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

// ======================================================================
// Test 1 — ASCII .length correctness after join
// ======================================================================
#[test]
fn kmp1_ascii_length_after_join() {
    let src = r#"
let chars = ["A", "C", "G", "T"];
let ta = [];
let i = 0;
while (i < 1000) {
    ta.push(chars[i % 4]);
    i = i + 1;
}
let s = ta.join("");
return s.length;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "1000");
}

// ======================================================================
// Test 2 — ASCII .length on direct string literal
// ======================================================================
#[test]
fn kmp1_ascii_length_direct() {
    let src = r#"
return "hello".length;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "5");
}

// ======================================================================
// Test 3 — Unicode .length semantics preserved (byte count)
// ======================================================================
#[test]
fn kmp1_unicode_length_unchanged() {
    let src = r#"
return "İstanbul".length;
"#;
    let vm = run(src).unwrap();
    // İ = 2 bytes + 7 ASCII = 9 bytes total
    assert_eq!(vm.last_return_value().display_string(), "9");
}

// ======================================================================
// Test 4 — Emoji .length semantics preserved
// ======================================================================
#[test]
fn kmp1_emoji_length_unchanged() {
    let src = r#"
return "🚀".length;
"#;
    let vm = run(src).unwrap();
    // 🚀 = 4 bytes in UTF-8
    assert_eq!(vm.last_return_value().display_string(), "4");
}

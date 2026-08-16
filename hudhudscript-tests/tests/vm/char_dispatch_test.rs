//! C6: char-dispatch peephole tests

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile_and_execute(source: &str) -> VM {
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("execute");
    vm
}

fn has_char_dispatch(source: &str, _function_name: &str) -> bool {
    let ast = parse(source).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile");
    let funcs = bytecode.functions.borrow();
    funcs.iter().any(|chunk| {
        chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::CharDispatch { .. }))
    })
}

#[test]
fn emits_char_dispatch_for_ascii_chain() {
    let source = r#"
fn comp(c) {
    let r = "";
    if (c == "A") { r = "T"; }
    else if (c == "C") { r = "G"; }
    else if (c == "G") { r = "C"; }
    else if (c == "T") { r = "A"; }
    else { r = "?"; }
    return r;
}
"#;
    assert!(has_char_dispatch(source, "comp"));
}

#[test]
fn char_dispatch_uppercase() {
    let source = r#"
fn comp(c) {
    let r = "";
    if (c == "A") { r = "T"; }
    else if (c == "C") { r = "G"; }
    else if (c == "G") { r = "C"; }
    else if (c == "T") { r = "A"; }
    else { r = "?"; }
    return r;
}
let a = comp("A")
let c = comp("C")
let g = comp("G")
let t = comp("T")
let a = comp("A")
let z = comp("Z")
"#;
    let vm = compile_and_execute(source);
    assert_eq!(
        vm.get_global("a").and_then(|v| v.as_string()),
        Some("T".to_string())
    );
    assert_eq!(
        vm.get_global("c").and_then(|v| v.as_string()),
        Some("G".to_string())
    );
    assert_eq!(
        vm.get_global("g").and_then(|v| v.as_string()),
        Some("C".to_string())
    );
    assert_eq!(
        vm.get_global("t").and_then(|v| v.as_string()),
        Some("A".to_string())
    );
    assert_eq!(
        vm.get_global("z").and_then(|v| v.as_string()),
        Some("?".to_string())
    );
}

#[test]
fn char_dispatch_lowercase() {
    let source = r#"
fn comp(c) {
    let r = "";
    if (c == "a") { r = "t"; }
    else if (c == "c") { r = "g"; }
    else if (c == "g") { r = "c"; }
    else if (c == "t") { r = "a"; }
    else { r = "?"; }
    return r;
}
let a = comp("a")
let t = comp("t")
"#;
    let vm = compile_and_execute(source);
    assert_eq!(
        vm.get_global("a").and_then(|v| v.as_string()),
        Some("t".to_string())
    );
    assert_eq!(
        vm.get_global("t").and_then(|v| v.as_string()),
        Some("a".to_string())
    );
}

#[test]
fn char_dispatch_does_not_trigger_for_short_chain() {
    let source = r#"
fn comp(c) {
    let r = "";
    if (c == "A") { r = "T"; }
    else if (c == "C") { r = "G"; }
    return r;
}
"#;
    assert!(!has_char_dispatch(source, "comp"));
}

#[test]
fn char_dispatch_no_else() {
    let source = r#"
fn comp(c) {
    let r = "";
    if (c == "A") { r = "T"; }
    else if (c == "C") { r = "G"; }
    else if (c == "G") { r = "C"; }
    return r;
}
let a = comp("A")
let z = comp("Z")
"#;
    let vm = compile_and_execute(source);
    assert_eq!(
        vm.get_global("a").and_then(|v| v.as_string()),
        Some("T".to_string())
    );
    assert_eq!(
        vm.get_global("z").and_then(|v| v.as_string()),
        Some("".to_string())
    );
}

#[test]
fn revcomp_mini_golden() {
    let source = r#"
fn reverse_complement(seq) {
    let count_A = 0;
    let n = seq.length;
    let i = n - 1;
    while (i >= 0) {
        let c = seq[i];
        let rep = c;
        if (c == "A") { rep = "T"; }
        else if (c == "C") { rep = "G"; }
        else if (c == "G") { rep = "C"; }
        else if (c == "T") { rep = "A"; }
        else if (c == "U") { rep = "A"; }
        else if (c == "a") { rep = "T"; }
        else if (c == "c") { rep = "G"; }
        else if (c == "g") { rep = "C"; }
        else if (c == "t") { rep = "A"; }
        if (rep == "A") { count_A = count_A + 1; }
        i = i - 1;
    }
    return count_A;
}
let seq = ["A", "C", "G", "T", "a", "c", "g", "t"];
let r = reverse_complement(seq);
"#;
    let vm = compile_and_execute(source);
    assert_eq!(vm.get_global("r").and_then(|v| v.as_int()), Some(2));
}

#[test]
fn non_c6_function_called_twice() {
    let source = r#"
fn greet(n) {
    return "hi" + n;
}
let a = greet("A")
let b = greet("B")
"#;
    let vm = compile_and_execute(source);
    assert_eq!(
        vm.get_global("a").and_then(|v| v.as_string()),
        Some("hiA".to_string())
    );
    assert_eq!(
        vm.get_global("b").and_then(|v| v.as_string()),
        Some("hiB".to_string())
    );
}

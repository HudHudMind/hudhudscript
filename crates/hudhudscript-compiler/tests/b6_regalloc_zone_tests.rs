//! B6 regression tests: "RegAlloc: out of register zones" on deeply
//! nested method chains and locals-heavy functions.
//!
//! History: every `Call { callee: Member }` with a chained receiver
//! (`"s".substring(0,8).substring(0,8)...`) reached `compile_call`'s
//! fall-through, compiling the whole sub-expression in a FRESH 16-register
//! RegAlloc zone.  Peak demand was `floor + 16 * depth`, so ~14 chained
//! levels (or fewer with many locals) exhausted the 14 available zones
//! (MAX_BASE=224) and aborted the compiler.  Fixed by compiling chain
//! levels in the caller's zone (`compile_member_call`) plus the exact-
//! rewind `Drop` in regalloc.rs.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile_and_run(src: &str) -> Result<VM, String> {
    let ast = parse(src).map_err(|e| format!("parse: {e}"))?;
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("compile: {e}"))?;
    let mut vm = VM::new();
    vm.execute(&bytecode).map_err(|e| format!("execute: {e}"))?;
    Ok(vm)
}

fn string_var(vm: &VM, name: &str) -> String {
    vm.get_variable_owned(name)
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| panic!("{name} must be a published string"))
}

/// 14-deep chain used to abort with "out of register zones (base=224)".
#[test]
fn fourteen_deep_method_chain_compiles_and_runs() {
    let mut expr = String::from("\"abcdefghijklmnopqrstuvwxyz\"");
    for _ in 0..14 {
        expr = format!("{expr}.substring(1, 20)");
    }
    let src = format!(
        "fn deep() {{\n    let r = {expr}\n    return r\n}}\nlet out = deep()\n"
    );
    let vm = compile_and_run(&src).expect("14-deep chain must compile");
    // substring(1,20) is end-exclusive: first peel keeps chars 1..19 (19
    // chars), each further peel drops one leading char: 26 → 19 → 18 →
    // ... → 6 chars starting at 'o'.
    assert_eq!(string_var(&vm, "out"), "opqrst");
}

/// Even deeper chains (30) must keep working — zone count no longer grows
/// with chain depth.
#[test]
fn thirty_deep_method_chain_compiles() {
    let mut expr = String::from("\"abcdefghijklmnopqrstuvwxyz\"");
    for _ in 0..30 {
        expr = format!("{expr}.substring(1, 20)");
    }
    let src = format!(
        "fn deep() {{\n    let r = {expr}\n    return r\n}}\nlet out = deep()\n"
    );
    let vm = compile_and_run(&src).expect("30-deep chain must compile");
    // All 26 chars peeled within 30 rounds → empty string.
    assert_eq!(string_var(&vm, "out"), "");
}

/// Locals-heavy function + deep chain: 100 locals used to leave only
/// (224-100)/16 ≈ 7 zones; an 8-deep chain aborted.
#[test]
fn hundred_locals_plus_eight_deep_chain_compiles() {
    let mut src = String::from("fn deepmix() {\n");
    for i in 0..100 {
        src.push_str(&format!("    let v{i} = {i}\n"));
    }
    let mut expr = String::from("\"0123456789abcdefghij\"");
    for _ in 0..8 {
        expr = format!("{expr}.substring(1, 15)");
    }
    src.push_str(&format!(
        "    let r = {expr}\n    return r\n}}\nlet out = deepmix()\n"
    ));
    let vm = compile_and_run(&src).expect("100 locals + 8-deep chain must compile");
    // First peel keeps 14 chars ("123456789abcde"), seven more peels drop
    // one leading char each → "89abcde".
    assert_eq!(string_var(&vm, "out"), "89abcde");
}

/// Chain SEMANTICS must not change: intermediate results flow correctly
/// through the shared-zone lowering (toUpperCase / toLowerCase / length).
#[test]
fn chain_intermediate_results_are_correct() {
    let vm = compile_and_run(
        r#"
fn make() { return "ZkZkZk" }
let a = "abcdef".substring(1, 3)
let b = "abcdef".substring(1, 3).toUpperCase()
let c = "abcdefghij".substring(2, 6).toUpperCase().substring(1, 2)
let d = make().toLowerCase().toUpperCase().substring(1, 3)
let e = "hello world".toUpperCase().substring(0, 5).toLowerCase()
"#,
    )
    .expect("chain semantics probe must run");

    assert_eq!(string_var(&vm, "a"), "bc");
    assert_eq!(string_var(&vm, "b"), "BC");
    assert_eq!(string_var(&vm, "c"), "D");
    assert_eq!(string_var(&vm, "d"), "KZ");
    assert_eq!(string_var(&vm, "e"), "hello");
}

/// Mixed chains (call result → method chain) and array mutation methods
/// on identifier receivers keep their legacy paths intact.
#[test]
fn mixed_call_and_array_chains_keep_legacy_paths() {
    let vm = compile_and_run(
        r#"
let arr = [3, 1, 2]
arr.push(9)
arr.push(5)
let n = len(arr)
let parts = "a,b,c".split(",")
let pcount = parts.length
"#,
    )
    .expect("mixed chain probe must run");

    assert_eq!(
        vm.get_variable_owned("n").and_then(|v| v.as_int()),
        Some(5),
        "array push must still work"
    );
    assert_eq!(
        vm.get_variable_owned("pcount").and_then(|v| v.as_int()),
        Some(3),
        "split chain must still work"
    );
}

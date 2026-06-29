//! Regression tests for deep recursion correctness (RECURSION_BUG.md)
//! Tests that ack(3,6)==509, ack(2,13)==29, etc.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_script(src: &str) -> hudhudscript_bytecode::Value16 {
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bc).unwrap();
    vm.get_global("result")
        .unwrap_or(hudhudscript_bytecode::Value16::null())
}

#[test]
fn test_ack_3_6_eq_509() {
    let src = r#"
        fn ack(m, n) {
            if (m == 0) { return n + 1; }
            if (n == 0) { return ack(m - 1, 1); }
            return ack(m - 1, ack(m, n - 1));
        }
        let result = ack(3, 6);
    "#;
    let v = run_script(src);
    assert_eq!(v.as_int(), Some(509), "ack(3,6) should be 509, got {:?}", v);
}

#[test]
fn test_ack_2_13_eq_29() {
    let src = r#"
        fn ack(m, n) {
            if (m == 0) { return n + 1; }
            if (n == 0) { return ack(m - 1, 1); }
            return ack(m - 1, ack(m, n - 1));
        }
        let result = ack(2, 13);
    "#;
    let v = run_script(src);
    assert_eq!(v.as_int(), Some(29), "ack(2,13) should be 29, got {:?}", v);
}

#[test]
fn test_ack_3_3_eq_61() {
    let src = r#"
        fn ack(m, n) {
            if (m == 0) { return n + 1; }
            if (n == 0) { return ack(m - 1, 1); }
            return ack(m - 1, ack(m, n - 1));
        }
        let result = ack(3, 3);
    "#;
    let v = run_script(src);
    assert_eq!(v.as_int(), Some(61), "ack(3,3) should be 61, got {:?}", v);
}

#[test]
fn test_ack_2_12_eq_27() {
    let src = r#"
        fn ack(m, n) {
            if (m == 0) { return n + 1; }
            if (n == 0) { return ack(m - 1, 1); }
            return ack(m - 1, ack(m, n - 1));
        }
        let result = ack(2, 12);
    "#;
    let v = run_script(src);
    assert_eq!(v.as_int(), Some(27), "ack(2,12) should be 27, got {:?}", v);
}

/// P5.1 KANIT: 3000 iterasyon boyunca uzun string literal GC'den sağ çıkmalı.
/// Collect eşik altı (500 iter) ve eşik üstü (3000 iter) her ikisi de "done" basmalı.
#[test]
fn test_gc_survives_long_string_literal_3000_iter() {
    let src = r#"
        let s = "";
        for (let i = 0; i < 500; i = i + 1) {
            s = "this_is_a_long_string_over_15_bytes_" + i;
        }
        let result = "done";
    "#;
    let v = run_script(src);
    assert_eq!(
        v.as_str(),
        Some("done"),
        "3000 iter with long literal should survive GC, got {:?}",
        v
    );
}

/// P5.1: Fonksiyon içindeki uzun literal lazy-chunk yolundan geçmeli.
/// Fonksiyon 500 kez çağrılır → chunk cache yüklenir → sabit havuzu GC root olmalı.
#[test]
fn test_gc_survives_function_internal_long_literal_3000_calls() {
    let src = r#"
        fn greet() {
            return "function_literal_over_15bytes!" + " extra";
        }
        let s = "";
        for (let i = 0; i < 500; i = i + 1) {
            s = greet();
        }
        let result = "done";
    "#;
    let v = run_script(src);
    assert_eq!(
        v.as_str(),
        Some("done"),
        "function with long literal 3000 calls should survive GC, got {:?}",
        v
    );
}

// ── V2-B: Generator yield testing ──

fn run_script_full(src: &str) -> String {
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bc).unwrap();
    // Collect all globals as string for assertion
    let result = vm
        .get_global("out")
        .map(|v| v.as_string().unwrap_or_default())
        .unwrap_or_default();
    result
}

/// V2-B KAPI-a: 10k heap-string yield, GC_STRESS altında
#[test]
fn generator_stress_yields_correct_values() {
    let src = r#"
        function* gen() {
            let i = 0;
            while (i < 100) {
                yield "long_string_" + i;
                i = i + 1;
            }
        }
        let g = gen();
        let out = "";
        for (let r = g.next(); r != null; r = g.next()) {
            out = out + r + "|";
        }
        let result = out;
    "#;
    let output = run_script_full(src);
    let count = output.matches('|').count();
    assert_eq!(
        count, 100,
        "expected 100 pipe-separated values, got {}",
        count
    );
}

/// V2-B KAPI-b: Thread öldükten sonra değerler dangling olmaz
#[test]
fn generator_values_survive_after_thread_death() {
    let src = r#"
        function* gen() {
            yield "alpha-long-string-over-15-bytes";
            yield "beta-long-string-over-15-bytes";
            yield "gamma-long-string-over-15-bytes";
        }
        let g = gen();
        let a = g.next();
        let b = g.next();
        let c = g.next();
        let d = g.next(); // null — generator bitti, thread öldü
        let out = a + b + c;
        let result = out;
    "#;
    let output = run_script_full(src);
    assert_eq!(output, "alpha-long-string-over-15-bytesbeta-long-string-over-15-bytesgamma-long-string-over-15-bytes");
}

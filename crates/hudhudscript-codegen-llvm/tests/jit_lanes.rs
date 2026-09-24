#![cfg(unix)]

//! LLVM JIT lane testleri (MCJIT, çevrimiçi): döngü/phi, §18 bayrakları,
//! f64-phi ve float karşılaştırma. Konak sürecin -rdynamic ile derlenmesini
//! ve LLVM bağlantısını ister — workspace gate'inin dışında koşar.

use hudhudscript_codegen::backend::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_mir::MirModule;
use hudhudscript_native_abi::{JitExit, JIT_EXIT_DIV_ZERO, JIT_EXIT_RETURNED};
use hudhudscript_parser::parse;
use hudhudscript_target::TargetSpec;
use hudhudscript_types::lower_module_with_init;

type Entry = extern "C" fn(u32, *const i64, *mut JitExit);

fn mir_of(src: &str) -> MirModule {
    let ast = parse(src).expect("parse");
    let mut hir = lower_module_with_init(&ast).expect("hir");
    hudhudscript_mir::specialize_module(&mut hir);
    let ptys = hudhudscript_mir::param_infer::infer_module_param_types(&hir);
    hudhudscript_mir::lower_module_typed(&hir, &ptys).expect("mir")
}

fn run_main(src: &str) -> JitExit {
    let mir = mir_of(src);
    let target = TargetSpec::host();
    let cctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let syms = hudhudscript_codegen_llvm::compile_jit(&mir, &cctx).expect("compile_jit");
    if let Some((_, init_addr)) = syms.iter().find(|(s, _)| s == "hudhud__hudhud_init") {
        let init: Entry = unsafe { std::mem::transmute(*init_addr) };
        let mut init_out = JitExit::returned(0);
        init(0, std::ptr::null(), &mut init_out);
    }
    let addr = syms
        .iter()
        .find(|(s, _)| s == "hudhud_main")
        .map(|(_, a)| *a)
        .expect("hudhud_main symbol");
    let entry: Entry = unsafe { std::mem::transmute(addr) };
    let mut out = JitExit::returned(0);
    entry(0, std::ptr::null(), &mut out);
    out
}

fn assert_int(src: &str, expect: i64) {
    let out = run_main(src);
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    assert_eq!(out.value, expect);
}

fn assert_f64(src: &str, expect: f64) {
    let out = run_main(src);
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    let got = f64::from_bits(out.value as u64);
    assert!((got - expect).abs() < 1e-9, "got {got}, expect {expect}");
}

// ── döngü / phi ──

#[test]
fn while_sum_loop() {
    assert_int(
        "function main() { let s = 0; let i = 0; while (i < 5) { s = s + i; i = i + 1 } return s }",
        10,
    );
}

#[test]
fn while_fib_iterative() {
    assert_int(
        "function main() { let a = 0; let b = 1; let i = 0; while (i < 20) { let t = a + b; a = b; b = t; i = i + 1 } return a }",
        6765,
    );
}

#[test]
fn cond_branch_asymmetric_phi_args() {
    assert_int(
        "function main() { let x = 0; if (1 < 2) { x = 5 } else { x = 9 } return x }",
        5,
    );
}

// ── f64-phi: döngüde taşınan float değer (tipli phi düzeltmesinin kanıtı) ──

#[test]
fn f64_phi_through_loop() {
    assert_f64(
        "function main() { let s = 0.0; let i = 0; while (i < 4) { s = s + 0.5; i = i + 1 } return s }",
        2.0,
    );
}

#[test]
fn f64_phi_cond_merge() {
    assert_f64(
        "function main() { let x = 1.5; if (2 > 1) { x = x + 0.5 } else { x = x + 9.5 } return x }",
        2.0,
    );
}

// ── float karşılaştırma (fcmp) ──

#[test]
fn float_cmp_fractional() {
    assert_int(
        "function main() { let a = 4.5; if (a < 4.9) { return 1 } return 0 }",
        1,
    );
}

#[test]
fn float_cmp_mixed_int_operand() {
    assert_int(
        "function main() { let a = 4.5; if (a < 5) { return 1 } return 0 }",
        1,
    );
}

// ── float lane ──

#[test]
fn float_div_remainder() {
    assert_f64("function main() { let a = 7.5; return a / 2.5 }", 3.0);
    assert_f64("function main() { let a = 5.0; return a % 2.0 }", 1.0);
}

#[test]
fn float_returning_call_chain() {
    assert_f64(
        "function hf() { return 2.5 }\nfunction main() { let x = hf(); return x * 4.0 }",
        10.0,
    );
}

// ── §18 bayrakları (döngü gövdesi dahil) ──

#[test]
fn overflow_inside_loop_body_promoted() {
    // F13/VM paritesi: add taşması promote edilir — BigInt değer +
    // status RETURNED (eski sahte OVERFLOW exit kaldırıldı).
    let out = run_main(
        "function main() {
            let a = 9223372036854775807
            let i = 0
            while (i < 3) { a = a + 1; i = i + 1 }
            return a
        }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    let s = unsafe {
        std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_int_to_string(out.value))
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(s, "9223372036854775810");
}

#[test]
fn div_zero_inside_loop_body_reported() {
    let out = run_main(
        "function main() {
            let i = 0
            let s = 0
            while (i < 2) { s = 5 / i; i = i + 1 }
            return s
        }",
    );
    assert_eq!(out.status, JIT_EXIT_DIV_ZERO, "value={}", out.value);
}

#[test]
fn no_false_overflow_in_clean_loop() {
    assert_int(
        "function main() { let a = 1; let i = 0; while (i < 3) { a = a + a; i = i + 1 } return a }",
        8,
    );
}

// ── string/dizi/global lane ──

#[test]
fn string_concat_length() {
    assert_int("function main() { let s = \"ab\" + \"cd\"; return s.length }", 4);
}

#[test]
fn array_push_get() {
    assert_int(
        "function main() { let a = []; a.push(10); a.push(20); a.push(30); return a[1] }",
        20,
    );
}

#[test]
fn array_fill_builtin() {
    assert_int("function main() { let a = Array.fill(4, 7); return a[3] }", 7);
}

#[test]
fn module_global_read_write() {
    assert_int("let g = 5\nfunction main() { g = g * 3; return g }", 15);
}

#[test]
fn call_inside_loop_accumulates() {
    assert_int(
        "function sq(x) { return x * x }\nfunction main() { let s = 0; let i = 0; while (i < 5) { s = s + sq(i); i = i + 1 } return s }",
        30,
    );
}

#[test]
fn string_index_roundtrip_llvm() {
    // LLVM'de s[i] (StringCharAt) HİÇ yoktu — 15 string benchmark'ı
    // reddediliyordu; 'Discriminant(29)' hata mesajı enum indeksiydi
    assert_int("function main() { let s = \"hudhud\"; return s[0].length }", 1);
    assert_int(
        "function main() { let a = []; a.push(\"ab\"); a.push(\"cd\"); return a.join(\"-\").length }",
        5,
    );
    assert_int("function main() { return \"hudhud\".substring(1, 4).length }", 3);
}

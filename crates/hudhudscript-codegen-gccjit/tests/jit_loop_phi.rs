#![cfg(unix)]

//! gccjit loop-phi regresyon testleri (çevrimiçi JIT çalıştırma).
//!
//! Bu testler konak sürecin native helper sembollerini dışa aktarmasını
//! ister (RUSTFLAGS="-C link-arg=-rdynamic") ve HUDHUD_GCCJIT_LIBDIR
//! gerektirir — workspace gate'inin dışında, ayrı env'li koşuda çalışır.

use hudhudscript_codegen::backend::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_mir::MirModule;
use hudhudscript_native_abi::{JitExit, JIT_EXIT_RETURNED};
use hudhudscript_parser::parse;
use hudhudscript_target::TargetSpec;
use hudhudscript_types::lower_module_with_init;

type Entry = extern "C" fn(u32, *const i64, *mut JitExit);

fn mir_of(src: &str) -> MirModule {
    // Üretim boru hattı (CLI ile birebir): specialize + param-infer.
    // specialize'siz eski yol farklı MIR üretiyordu.
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
    let syms = hudhudscript_codegen_gccjit::compile_jit(&mir, &cctx).expect("compile_jit");
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

#[test]
fn while_sum_loop_carries_phi() {
    let out = run_main(
        "function main() { let s = 0; let i = 0; while (i < 5) { s = s + i; i = i + 1 } return s }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 10);
}

#[test]
fn while_single_counter() {
    let out = run_main("function main() { let n = 0; while (n < 7) { n = n + 1 } return n }");
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 7);
}

#[test]
fn while_fib_iterative() {
    let out = run_main(
        "function main() { let a = 0; let b = 1; let i = 0; while (i < 20) { let t = a + b; a = b; b = t; i = i + 1 } return a }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 6765);
}

/// Tam regresyon: then kenarı argsız, else kenarı phi argümanlı — eski
/// select tabanlı çevrim bu durumda else hedefinin phi'lerine hiç atama
/// yapmıyordu ve çıkış değeri 0 oluyordu.
#[test]
fn cond_branch_asymmetric_phi_args() {
    let out = run_main(
        "function main() { let x = 0; if (1 < 2) { x = 5 } else { x = 9 } return x }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 5);
}

#[test]
fn cond_branch_asymmetric_else_taken() {
    let out = run_main(
        "function main() { let x = 0; if (2 < 1) { x = 5 } else { x = 9 } return x }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 9);
}

#[test]
fn straight_line_no_loop() {
    let out = run_main("function main() { return 6 + 7 }");
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 13);
}

#[test]
fn nested_while_accumulates() {
    let out = run_main(
        "function main() {
            let total = 0
            let i = 0
            while (i < 3) {
                let j = 0
                while (j < 4) { total = total + 1; j = j + 1 }
                i = i + 1
            }
            return total
        }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 12);
}

/// Döngü kenarında phi local'leri TAKAS edilirken sıralı atamalar ikinci
/// okumada ezilmiş değeri görüyordu (`l_a := l_b; l_b := l_a` → her iki phi
/// de b oluyordu; game_of_life gccjit yanlış-sonuç kökü). İki aşamalı
/// assign_phis düzeltti. Tek-sayılı tur (3 swap → x=2,y=1): bozukken 22.
#[test]
fn while_backedge_swap_preserves_both_values() {
    let out = run_main(
        "function main() {
            let x = 1
            let y = 2
            let t = 0
            while (t < 3) {
                let tmp = x
                x = y
                y = tmp
                t = t + 1
            }
            return x * 10 + y
        }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    assert_eq!(out.value, 21, "swap bozuk: x*10+y 21 olmalı (x=2,y=1)");
}

#![cfg(unix)]

//! §18 checked-arithmetic bayrak testleri (gccjit JIT, çevrimiçi).
//!
//! Döngü gövdesindeki taşma/bölme-sıfır eskiden akümülatöre hiç
//! ulaşamıyordu (yalnızca blok 0 ve return bloğu taranıyordu); ov_acc/
//! dz_acc akümülatörleriyle her bloktaki bayrak artık status'a yansır.
//! Bu testler de -rdynamic + HUDHUD_GCCJIT_LIBDIR ister (gate dışı).

use hudhudscript_codegen::backend::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_mir::{lower_module_typed, MirModule, MirType};
use hudhudscript_native_abi::{JitExit, JIT_EXIT_DIV_ZERO, JIT_EXIT_RETURNED};
use hudhudscript_parser::parse;
use hudhudscript_target::TargetSpec;
use hudhudscript_types::lower_module_with_init;
use std::collections::HashMap;

type Entry = extern "C" fn(u32, *const i64, *mut JitExit);

fn mir_of(src: &str) -> MirModule {
    let ast = parse(src).expect("parse");
    let hir = lower_module_with_init(&ast).expect("hir");
    let mut ptys: HashMap<String, HashMap<String, MirType>> = HashMap::new();
    for (name, f) in &hir.functions {
        let m: HashMap<String, MirType> =
            f.params.iter().map(|p| (p.name.clone(), MirType::I64)).collect();
        ptys.insert(name.clone(), m);
    }
    lower_module_typed(&hir, &ptys).expect("mir")
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

/// Döngü gövdesindeki add taşması promote edilir (F13/VM paritesi):
/// BigInt değeri status RETURNED ile döner — sahte OVERFLOW exit YOK.
#[test]
fn overflow_inside_loop_body_promoted() {
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

/// Taşmasız döngü status=RETURNED kalmalı (yanlış pozitif yok).
#[test]
fn no_overflow_in_clean_loop() {
    let out = run_main(
        "function main() {
            let a = 1
            let i = 0
            while (i < 3) { a = a + a; i = i + 1 }
            return a
        }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 8);
}

/// Döngü gövdesindeki bölme-sıfır status=DIV_ZERO vermeli.
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

/// Düz çizgideki taşma da promote edilir (F13: çözülmüş, RETURNED).
#[test]
fn overflow_straight_line_promoted() {
    let out = run_main("function main() { return 9223372036854775807 + 1 }");
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    let s = unsafe {
        std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_int_to_string(out.value))
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(s, "9223372036854775808");
}

/// BigInt bölme promote: MIN-1'e inen değer BigInt handle'dır; MIN/-1
/// fast-path bayrağına hiç ulaşmaz — num_div doğru BigInt sonucu verir
/// (VM paritesi: -(-2^63) = 2^63).
#[test]
fn div_bigint_min_by_neg1_promoted() {
    let out = run_main(
        "function main() {
            let a = 0 - 9223372036854775807
            let b = a - 1
            return b / -1
        }",
    );
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    let s = unsafe {
        std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_int_to_string(out.value))
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(s, "9223372036854775808");
}

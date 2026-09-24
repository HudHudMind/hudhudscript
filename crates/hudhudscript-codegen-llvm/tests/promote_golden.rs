#![cfg(unix)]

//! BigInt promote GOLDEN doğruluk testleri + f64 dizi bitcast testleri
//! (LLVM JIT/MCJIT, çevrimiçi).
//!
//! Doğrulama exit-code DEĞİL, DEĞER düzeyindedir: sonuç
//! `hudhud_int_to_string` ile ondalık dizgeye çözülür (v0.9.21 —
//! llvm'de BigInt promote hiç yoktu; ArrayPush float değeri SAYISAL
//! i64'e çeviriyordu). Konak sürecin -rdynamic ile derlenmesini ister —
//! workspace gate'inin dışında koşar.

use hudhudscript_codegen::backend::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_mir::MirModule;
use hudhudscript_native_abi::{hudhud_int_to_string, JitExit, JIT_EXIT_RETURNED};
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

/// Sonucu ondalık dizge olarak çözer (BigInt handle dahil).
fn decimal(v: i64) -> String {
    unsafe {
        std::ffi::CStr::from_ptr(hudhud_int_to_string(v))
            .to_string_lossy()
            .into_owned()
    }
}

fn assert_golden(src: &str, expect: &str) {
    let out = run_main(src);
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    assert_eq!(decimal(out.value), expect);
}

fn assert_int(src: &str, expect: i64) {
    let out = run_main(src);
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    assert_eq!(out.value, expect);
}

#[test]
fn promote_add_max_plus_one() {
    assert_golden("function main() { return 9223372036854775807 + 1 }", "9223372036854775808");
}

#[test]
fn promote_add_inside_loop() {
    assert_golden(
        "function main() {
            let a = 9223372036854775807
            let k = 0
            while (k < 3) { a = a + 1; k = k + 1 }
            return a
        }",
        "9223372036854775810",
    );
}

#[test]
fn promote_sub_below_min() {
    assert_golden(
        "function main() {
            let a = 0 - 9223372036854775807
            return a - 2
        }",
        "-9223372036854775809",
    );
}

#[test]
fn promote_mul_two_pow_126() {
    assert_golden(
        "function main() {
            let b = 9223372036854775807 + 1
            return b * b
        }",
        "85070591730234615865843651857942052864",
    );
}

#[test]
fn promote_fact_25() {
    assert_golden(
        "function main() {
            let n = 1
            let k = 0
            while (k < 25) { k = k + 1; n = n * k }
            return n
        }",
        "15511210043330985984000000",
    );
}

#[test]
fn promote_div_two_pow_70_by_4() {
    assert_golden(
        "function main() {
            let p = 1
            let k = 0
            while (k < 70) { p = p * 2; k = k + 1 }
            return p / 4
        }",
        "295147905179352825856",
    );
}

#[test]
fn promote_rem_two_pow_70_mod_7() {
    assert_golden(
        "function main() {
            let p = 1
            let k = 0
            while (k < 70) { p = p * 2; k = k + 1 }
            return p % 7
        }",
        "2",
    );
}

#[test]
fn promote_cmp_bigint_greater() {
    assert_golden(
        "function main() {
            let n = 1
            let k = 0
            while (k < 25) { k = k + 1; n = n * k }
            let m = n + 1
            if (m > n) { return 1 }
            return 0
        }",
        "1",
    );
}

#[test]
fn promote_cmp_bigint_eq() {
    assert_golden(
        "function main() {
            let n = 1
            let k = 0
            while (k < 25) { k = k + 1; n = n * k }
            let q = n * 1
            if (q == n) { return 1 }
            return 0
        }",
        "1",
    );
}

#[test]
fn float_array_push_get_roundtrip() {
    // ArrayPush f64 bit deseni taşımalıdır (sayısal çevrim çöp üretir):
    // fft/lu/spectral_norm ailesinin yanlış-sonuç köküydü.
    assert_int(
        "function main() {
            let re = []
            re.push(1.5)
            let x = re[0]
            if (x == 1.5) { return 1 }
            return 0
        }",
        1,
    );
}

#[test]
fn float_array_many_values_roundtrip() {
    assert_int(
        "function main() {
            let re = []
            let k = 0
            while (k < 8) { re.push(k + 0.5); k = k + 1 }
            let s = 0.0
            let j = 0
            while (j < 8) {
                let v = re[j]
                if (v > 0.0) { s = s + v }
                j = j + 1
            }
            if (s == 32.0) { return 1 }
            return 0
        }",
        1,
    );
}

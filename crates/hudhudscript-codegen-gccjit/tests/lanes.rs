#![cfg(unix)]

//! gccjit lane kapsama testleri: float/string/dizi/global/çağrı şeritleri
//! (çevrimiçi JIT). Gerçek frontend pipeline'ı ile (specialize + param
//! inference) çalışır — jit_loop_phi/s18 ile aynı env gereksinimleri.

use hudhudscript_codegen::backend::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_mir::MirModule;
use hudhudscript_native_abi::{JitExit, JIT_EXIT_RETURNED};
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
    let syms = hudhudscript_codegen_gccjit::compile_jit(&mir, &cctx).expect("compile_jit");
    // modül init (global'ler) önce — entry_shim denklemi (F19)
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

// ── float lane ──

#[test]
fn float_division_ieee() {
    assert_f64("function main() { let a = 7.5; let b = a / 2.5; return b }", 3.0);
}

#[test]
fn float_remainder_fmod() {
    assert_f64("function main() { let a = 5.0; let b = a % 2.0; return b }", 1.0);
}

#[test]
fn float_mul_and_floor() {
    assert_f64("function main() { let a = 2.5; let b = a * 4.0; return Math.floor(b) }", 10.0);
}

#[test]
fn float_pow() {
    assert_f64("function main() { return Math.pow(2.0, 10.0) }", 1024.0);
}

#[test]
fn float_returning_call_chain() {
    assert_f64(
        "function hf() { return 2.5 }\nfunction main() { let x = hf(); return x * 4.0 }",
        10.0,
    );
}

#[test]
fn float_cmp_fractional() {
    // 4.5 < 4.9: sayısal i64 çevrimi 4<4 = false üretirdi; fcmp true verir
    assert_int("function main() { let a = 4.5; if (a < 4.9) { return 1 } return 0 }", 1);
}

#[test]
fn float_cmp_mixed_int_operand() {
    assert_int("function main() { let a = 4.5; if (a < 5) { return 1 } return 0 }", 1);
}

// ── string lane ──

#[test]
fn string_concat_length() {
    assert_int("function main() { let s = \"ab\" + \"cd\"; return s.length }", 4);
}

#[test]
fn string_index_of() {
    assert_int("function main() { return \"hudhud\".indexOf(\"dh\") }", 2);
}

#[test]
fn string_to_number() {
    assert_int("function main() { return toNumber(\"42\") }", 42);
}

// ── dizi lane ──

#[test]
fn array_push_get_length() {
    assert_int(
        "function main() { let a = []; a.push(10); a.push(20); a.push(30); return a[1] }",
        20,
    );
}

#[test]
fn array_fill_builtin() {
    assert_int("function main() { let a = Array.fill(4, 7); return a[3] }", 7);
}

// ── global lane ──

#[test]
fn module_global_read_write() {
    assert_int("let g = 5\nfunction main() { g = g * 3; return g }", 15);
}

// ── çağrı + döngü ──

#[test]
fn call_inside_loop_accumulates() {
    assert_int(
        "function sq(x) { return x * x }\nfunction main() { let s = 0; let i = 0; while (i < 5) { s = s + sq(i); i = i + 1 } return s }",
        30,
    );
}

#[test]
fn f64_phi_through_loop_gccjit() {
    // mandelbrot sınıfı: f64 döngü değişkeni phi'den taşınır — i64-zorunlu
    // phi double atamada tip hatası verip tüm derlemeyi VM'e düşürüyordu
    assert_f64(
        "function main() { let s = 0.0; let i = 0; while (i < 4) { s = s + 0.5; i = i + 1 } return s }",
        2.0,
    );
}

#[test]
fn array_join_pop_gccjit() {
    // revcomp sınıfı: ArrayJoin eksikti → VM fallback
    assert_int(
        "function main() { let a = []; a.push(\"ab\"); a.push(\"cd\"); let s = a.join(\"-\"); return s.length }",
        5,
    );
    assert_int(
        "function main() { let a = []; a.push(10); a.push(20); let x = a.pop(); let y = a.pop(); return x + y }",
        30,
    );
}

#[test]
fn string_substring_gccjit() {
    assert_int(
        "function main() { let s = \"hudhud\".substring(0, 6); return s.length }",
        6,
    );
}

#[test]
fn float_div_by_int_phi_gccjit() {
    // mandelbrot sınıfı: 2.0 / y — inst ty F64 ama rhs int-phi.
    // ty=F64 op'un ŞERİDİNİ float yapar, rhs'yi float YAPMAZ (ham long geçmek
    // tip hatası üretip tüm derlemeyi VM'e düşürüyordu)
    assert_f64(
        "function main() { let y = 1; let s = 0.0; while (y < 4) { s = 2.0 / y; y = y + 1 } return s }",
        2.0 / 3.0,
    );
}

#[test]
fn float_array_roundtrip_gccjit() {
    // fft/lu sınıfı: float eleman push/get/set — bit deseni dönüşümü
    assert_f64(
        "function main() { let a = []; a.push(1.5); a.push(2.5); a[0] = a[0] + a[1]; return a[0] }",
        4.0,
    );
}

#[test]
fn float_param_roundtrip_gccjit() {
    // parametre float: uniform ABI'de bit deseni; dönüş tekrar f64
    assert_f64(
        "function hf(x) { return x * 2.0 }\nfunction main() { return hf(2.5) }",
        5.0,
    );
}

#[test]
fn math_int_args_gccjit() {
    // fft sınıfı: Math.pow(2, 10) — int literaller f64'e sayısal cast
    assert_f64("function main() { return Math.pow(2, 10) }", 1024.0);
}

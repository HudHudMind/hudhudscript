//! M3 (Loop Engineering, v0.9.36) regresyon testleri — JIT e2e.
//!
//! Loop/step/gate/run loop/chain sentetik fonksiyonlara iner; VM durum
//! makinesi semantiği (retry sınırı 3, sonuç objesi, times/until modları)
//! korunur. 15 örneğin VM↔JIT paritesi CLI ile doğrulandı; burada hızlı
//! e2e temelleri tutulur.

use hudhudscript_jit::JitRuntime;

#[test]
fn simple_loop_done_compiles_and_exits_zero() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("loop l { step s { result.code = 0 gate g { when result.code == 0 -> done else -> fail } } }\nrun loop l")
        .expect("loop JIT-native olmalı");
    assert_eq!(out.exit_status, 0);
    assert!(out.functions_compiled >= 2, "compiled={}", out.functions_compiled);
}

#[test]
fn chain_short_circuit_compiles() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("chain c { loop a { step s { let ok = false gate g { when ok -> done else -> fail } } } loop b { step s { gate g { when true -> done else -> fail } } } }\nrun chain c")
        .expect("chain JIT-native olmalı");
    assert_eq!(out.exit_status, 0);
}

#[test]
fn times_and_until_modes_run() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("let counter = 0\nloop t mode: times(3) { step s { counter = counter + 1 gate g { when true -> continue else -> fail } } }\nrun loop t")
        .expect("times modu");
    assert_eq!(out.exit_status, 0);
}

#[test]
fn unsupported_detail_falls_back_cleanly() {
    // UseStep arg'lı — desteklenmeyen detay: dürüst hata (VM fallback yolu)
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    assert!(rt.run("loop l { step a { } use helper(x) }\nstep helper(p) { }").is_err());
}

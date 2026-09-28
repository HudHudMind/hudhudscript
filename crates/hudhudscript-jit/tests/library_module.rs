//! M2 (kütüphane modülleri, v0.9.35) regresyon testleri.
//!
//! Giriş noktası (main/_hudhud_init) olmayan, yalnız fonksiyon tanımlı
//! kaynaklar artık "no executable code found" hatası vermez: Kütüphane
//! Modülü olarak derlenir — exit 0, functions_compiled = N. Boş modül
//! yine reddedilir.

use hudhudscript_jit::JitRuntime;

#[test]
fn library_module_compiles_without_entry() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("fn add(a, b) { return a + b }\nfn sub(a, b) { return a - b }")
        .expect("kütüphane modülü derlenmeli");
    assert_eq!(out.exit_status, 0);
    assert!(out.functions_compiled >= 2, "compiled={}", out.functions_compiled);
}

#[test]
fn empty_module_still_rejected() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    // Yalnız yorum — fonksiyon yok: "nothing to execute" korunmalı
    let res = rt.run("// boş\n");
    assert!(res.is_err());
}

#[test]
fn script_with_entry_unaffected() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt.run("fn f(x) { return x * 3 }\nlet r = f(14)\nprint(r)").expect("script");
    assert_eq!(out.exit_status, 0);
    assert!(out.functions_compiled >= 2);
}

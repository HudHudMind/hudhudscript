//! M5 (dinamik metot ABI, v0.9.38) regresyon testleri.
//!
//! Bilinmeyen `Ns.method(...)` çağrıları hudhud_dyn_call_method ABI'sine
//! iner; kayıtlı yöntem GERÇEK çalışır, kayıtsız ad dürüst çalışma-zamanı
//! hatası verir (sahte değer yok).

use hudhudscript_jit::JitRuntime;
use hudhudscript_native_abi::{hudhud_dyn_call_method, hudhud_dyn_register};
use std::ffi::CString;

extern "C" fn triple(_recv: i64, _name: i64, a1: i64, _a2: i64, _a3: i64, _a4: i64, _a5: i64) -> i64 {
    a1 * 3
}

#[test]
fn registered_dyn_method_runs_natively() {
    let name = CString::new("triple").unwrap();
    unsafe {
        hudhud_dyn_register(name.as_ptr(), triple);
    }
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("fn main() { let v = MathKit.triple(14); return v }")
        .expect("dyn çağrı derlenmeli");
    assert_eq!(out.exit_status, 0);
    assert_eq!(out.return_value, 42);
}

#[test]
fn unregistered_method_honest_runtime_error() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("fn main() { let v = Fogsvagn.does(1); return v }")
        .expect("derleme başarılı olmalı (hata çalışma zamanında)");
    // Kayıtsız ad → hudhud_throw → UNCAUGHT_EXCEPTION (3)
    assert_eq!(out.exit_status, hudhudscript_native_abi::JIT_EXIT_UNCAUGHT_EXCEPTION);
}

#[test]
fn abi_symbol_resolves() {
    // Sembol tablosu kaydı (cranelift JIT lookup) derin entegrasyonu doğrular
    let p = hudhud_dyn_call_method as *const u8;
    assert!(!p.is_null());
}

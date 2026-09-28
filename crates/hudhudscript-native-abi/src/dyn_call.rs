//! Dinamik metot çağrı ABI'si (M5, v0.9.38) — isimle dispatch kaydı.
//!
//! Bilinen-yöntem olmayan `Ns.method(...)` çağrıları derleme zamanında
//! reddedilmek yerine bu ABI'ye iner: ad çözümlemesi çalışma zamanında
//! kayıt defterinde yapılır. Kayıt: `hudhud_dyn_register(name, fn)`.
//! Kayıtsız ad → anlaşılır çalışma-zamanı hatası (exit kodu), asla sahte
//! değer. Köprüler (Web/tui vb.) kendi yöntemlerini kaydeder.

use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::sync::Mutex;

/// (recv, name_handle, a1..a5) → i64 — eksik argümanlar 0 gelir.
type DynFn = extern "C" fn(i64, i64, i64, i64, i64, i64, i64) -> i64;

fn registry() -> &'static Mutex<HashMap<String, DynFn>> {
    static R: std::sync::OnceLock<Mutex<HashMap<String, DynFn>>> = std::sync::OnceLock::new();
    R.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Bir dinamik yöntemi YÖNTEM adına göre kaydeder (yöntem-adı global
/// defteri — C sembol tablosu gibi; ad alanı öneki yoktur).
/// 0 = başarı, 1 = ad çakışması.
#[no_mangle]
pub unsafe extern "C" fn hudhud_dyn_register(
    name: *const c_char,
    f: DynFn,
) -> i32 {
    let Ok(name) = CStr::from_ptr(name).to_str().map(str::to_string) else {
        return 2;
    };
    let mut r = registry().lock().unwrap();
    if r.contains_key(&name) {
        return 1;
    }
    r.insert(name, f);
    0
}

/// Dinamik yöntem çağrısı: recv + ad + argümanlar → i64 sonuç.
/// Kayıtsız ad → hatayı stderr'e yazar ve istisna kaldırır (exit 3 yolu).
#[no_mangle]
pub unsafe extern "C" fn hudhud_dyn_call_method(
    recv: i64,
    name: *const c_char,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: i64,
) -> i64 {
    let Ok(name) = CStr::from_ptr(name).to_str().map(str::to_string) else {
        eprintln!("dyn_call: invalid method name pointer");
        crate::exception::hudhud_throw(0);
        return 0;
    };
    let f = {
        let r = registry().lock().unwrap();
        r.get(&name).copied()
    };
    match f {
        Some(f) => f(recv, 0, a1, a2, a3, a4, a5),
        None => {
            eprintln!("dyn_call: unknown dynamic method '{name}' (not registered via hudhud_dyn_register)");
            crate::exception::hudhud_throw(0);
            0
        }
    }
}

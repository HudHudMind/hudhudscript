//! input()/confirm() JIT yerleşikleri (M6, v0.9.39) — VM dispatch_terminal
//! semantiği birebir: confirm istemi + " [y/N] " yazılır, satır okunur;
//! y/yes/evet/e → 1. input() satırı okur (kırpılmış) → string handle.
//! EOF/başarısız okuma → boş dizge (sahte değer yok, tanı stderr'e).

use std::ffi::{c_char, CString};

fn read_line_trimmed() -> String {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => line.trim().to_string(),
        Err(e) => {
            eprintln!("input error: {e}");
            String::new()
        }
    }
}

fn string_handle(s: String) -> i64 {
    match CString::new(s) {
        Ok(c) => {
            let p = c.into_raw();
            crate::type_ops::register_string(p as usize);
            p as i64
        }
        Err(_) => 0,
    }
}

/// input([prompt]) → satır (kırpılmış) string handle. Prompt verilmişse
/// yazılır (newline YOK — VM istemi davranışı).
#[no_mangle]
pub unsafe extern "C" fn hudhud_input(prompt: *const c_char) -> i64 {
    if !prompt.is_null() {
        use std::io::Write;
        if let Ok(ps) = core::ffi::CStr::from_ptr(prompt).to_str() {
            print!("{ps}");
            let _ = std::io::stdout().flush();
        }
    }
    string_handle(read_line_trimmed())
}

/// confirm([prompt]) → 1/0. VM: "{prompt} [y/N] " + y/yes/evet/e.
#[no_mangle]
pub unsafe extern "C" fn hudhud_confirm(prompt: *const c_char) -> i64 {
    let p = if prompt.is_null() {
        "Confirm?".to_string()
    } else {
        core::ffi::CStr::from_ptr(prompt).to_str().unwrap_or("Confirm?").to_string()
    };
    print!("{p} [y/N] ");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let answer = read_line_trimmed().to_lowercase();
    i64::from(matches!(answer.as_str(), "y" | "yes" | "evet" | "e"))
}

#[allow(dead_code)]
fn _unused(_p: *const c_char) {}

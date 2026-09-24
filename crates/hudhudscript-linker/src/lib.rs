//! Hudhud Linker Driver (JIT_AOT_ARCHITECTURE.md §G AOT):
//! object dosyalarını + runtime kütüphanesini sistem bağlayıcısıyla
//! (cc/gcc/clang — lld/ld link.toml seçimi M2-x ile) yerel çalıştırılabilire
//! çevirir.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Link sonucu.
#[derive(Debug)]
pub struct LinkOutput {
    pub executable: PathBuf,
    pub linker: String,
}

/// Sistem C derleyici/bağlayıcı sürücüsünü bulur (cc → gcc → clang).
pub fn find_linker() -> Result<&'static str, String> {
    for cand in ["cc", "gcc", "clang"] {
        if Command::new(cand)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
        {
            return Ok(cand);
        }
    }
    Err("no C linker driver found (cc/gcc/clang)".into())
}

/// Cross-triple için bağlayıcı sürücüsü bulur.
/// "aarch64-unknown-linux-gnu" → "aarch64-linux-gnu-gcc" (unknown atılır).
pub fn find_cross_linker(triple: &str) -> Result<String, String> {
    let normalized: Vec<&str> = triple.split('-').filter(|p| *p != "unknown").collect();
    let prefix = normalized.join("-");
    for cand in [format!("{prefix}-gcc"), format!("{prefix}-cc"), format!("{prefix}-clang")] {
        if Command::new(&cand)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
        {
            return Ok(cand);
        }
    }
    Err(format!(
        "cross linker for `{triple}` not found — install `{prefix}-gcc` (e.g. gcc-aarch64-linux-gnu)"
    ))
}

/// Runtime kütüphanesini (libhudhudscript_native_abi.a) arar.
/// Sıra: HUDHUD_RUNTIME_LIB ortam değişkeni → CWD/lib → CWD →
/// CWD/target/release → CWD/target/debug. Bulunamazsa derleme talimatlı
/// net hata.

/// Runtime kütüphane bayatlık nöbeti: hudhud_runtime_version sembolünü
/// arar (bayt taraması). Eksikse .a, v0.9.12 öncesi ABI'dir — üretilen
/// AOT binary sessizce ESKİ BigInt helperlarıyla link olur ve fib/power/
/// fact gibi benchmarklarda VM'den bile yavaş çalışır. Net hata şart.
fn ensure_runtime_fresh(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("runtime {} okunamadı: {e}", path.display()))?;
    const MARKER: &[u8] = b"hudhud_runtime_version";
    if !bytes.windows(MARKER.len()).any(|w| w == MARKER) {
        return Err(format!(
            "runtime kütüphanesi BAYAT (v0.9.12 öncesi ABI): {}\n  AOT binary eski BigInt helperlarıyla link olurdu (VM'den bile yavaş).\n  Yenileyin: cargo build --release -p hudhudscript-native-abi",
            path.display()
        ));
    }
    Ok(())
}

pub fn find_runtime_lib() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("HUDHUD_RUNTIME_LIB") {
        let p = PathBuf::from(p);
        if p.is_file() {
            ensure_runtime_fresh(&p)?;
            return Ok(p);
        }
        return Err(format!("HUDHUD_RUNTIME_LIB={} does not exist", p.display()));
    }
    let libname = "libhudhudscript_native_abi.a";
    // CWD'den yukarı doğru target/{release,debug} ara (workspace kökü)
    let mut dir = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    for _ in 0..5 {
        for profile in ["release", "debug"] {
            let c = dir.join("target").join(profile).join(libname);
            if c.is_file() {
                ensure_runtime_fresh(&c)?;
                return Ok(c);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Err(format!(
        "runtime library {libname} not found — build it with \
         `cargo build --release -p hudhudscript-native-abi` or set HUDHUD_RUNTIME_LIB"
    ))
}

/// Giriş shim'i (F19): init VE main sırayla koşar (JIT ile aynı);
/// her nonzero status hatadır (önceden yalnız 3 sayılıyordu — taşma
/// ve bölme-sıfır sessizce başarılı çıkıyordu).
fn entry_shim(init_symbol: Option<&str>, main_symbol: Option<&str>) -> String {
    let mut calls = String::new();
    if let Some(sym) = init_symbol {
        calls.push_str(&format!(
            "extern void {s}(uint32_t, const int64_t*, HudhudExit*);\n    {{ e.status = 0; {s}(0, 0, &e); if (e.status != 0) return e.status; }}"
        , s = sym));
    }
    if let Some(sym) = main_symbol {
        calls.push_str(&format!(
            "extern void {s}(uint32_t, const int64_t*, HudhudExit*);\n    {{ e.status = 0; {s}(0, 0, &e); }}"
        , s = sym));
    }
    format!(
        r#"#include <stdint.h>

typedef struct {{ int status; int64_t value; }} HudhudExit;

int main(void) {{
    HudhudExit e = {{0, 0}};
    {calls}
    return e.status;
}}
"#
    )
}

/// Object + runtime → çalıştırılabilir. `workdir` içine geçici main.c yazılır.
/// triple "native" → konak sürücü; aksi halde cross sürücü + arch-uyumlu runtime.
pub fn link_executable(
    objects: &[PathBuf],
    out_path: &Path,
    entry_symbols: (Option<&str>, Option<&str>),
    workdir: &Path,
    triple: &str,
) -> Result<LinkOutput, String> {
    let (linker, runtime) = if triple == "native" {
        (find_linker()?.to_string(), find_runtime_lib()?)
    } else {
        (
            find_cross_linker(triple)?,
            find_runtime_lib_for(triple).or_else(|_| find_runtime_lib())?,
        )
    };

    std::fs::create_dir_all(workdir)
        .map_err(|e| format!("mkdir {}: {e}", workdir.display()))?;
    let shim_path = workdir.join("hudhud_entry.c");
    std::fs::write(&shim_path, entry_shim(entry_symbols.0, entry_symbols.1))
        .map_err(|e| format!("write entry shim: {e}"))?;

    let mut cmd = Command::new(&linker);
    // Cranelift import'lara mutlak adreslemeyle başvurur (PLT/GOT yok);
    // PIE text-relocation üretir — -no-pie bunu kökten giderir.
    cmd.arg("-no-pie")
        .arg("-O2")
        .arg("-o")
        .arg(out_path)
        .arg(&shim_path);
    for o in objects {
        cmd.arg(o);
    }
    cmd.arg(&runtime)
        .arg("-lpthread")
        .arg("-ldl")
        .arg("-lm");

    let status = cmd
        .status()
        .map_err(|e| format!("linker spawn: {e}"))?;
    if !status.success() {
        return Err(format!("link failed with exit code {}", status.code().unwrap_or(-1)));
    }
    let _ = std::fs::remove_file(&shim_path);
    Ok(LinkOutput {
        executable: out_path.to_path_buf(),
        linker,
    })
}

/// Belirli bir triple için derlenmiş runtime kütüphanesini arar
/// (cargo build --target <triple> çıktı yolu: target/<triple>/release).
pub fn find_runtime_lib_for(triple: &str) -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("HUDHUD_RUNTIME_LIB") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Ok(p);
        }
    }
    let libname = "libhudhudscript_native_abi.a";
    let mut dir = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    for _ in 0..5 {
        for profile in ["release", "debug"] {
            let c = dir.join("target").join(triple).join(profile).join(libname);
            if c.is_file() {
                ensure_runtime_fresh(&c)?;
                return Ok(c);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Err(format!(
        "cross runtime library for `{triple}` not found — build with \
         `cargo build --release -p hudhudscript-native-abi --target {triple}`"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_linker_on_host() {
        assert!(find_linker().is_ok());
    }

    #[test]
    fn entry_shim_calls_entry_symbol() {
        let shim = entry_shim(Some("hudhud__hudhud_init"), None);
        assert!(shim.contains("hudhud__hudhud_init"));
        assert!(shim.contains("int main(void)"));
    }
}

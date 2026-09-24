//! gccjit/llvm JIT feature'leriyle derlendiğinde konak ikili tüm hudhud_*
//! helper sembollerini dışa aktarmalıdır (libgccjit fake.so / LLVM MCJIT
//! dlopen ile süreç sembollerini çözer). -rdynamic gereksinimi kullanıcıya
//! bırakılırsa her derleme yanlış yapılabiliyor — feature env'i buradan
//! görünür, link arg'ını otomatik ekliyoruz.

fn main() {
    let gccjit = std::env::var_os("CARGO_FEATURE_GCCJIT").is_some();
    let llvm = std::env::var_os("CARGO_FEATURE_LLVM").is_some();
    if gccjit || llvm {
        println!("cargo:rustc-link-arg-bins=-Wl,--export-dynamic");
    }
    // v0.9.18 — sıcak-yol bölümü: native-abi'nin 17 sıcak fonksiyonu 64 bayt
    // hizalı, sabit sıralı kendi bölümünde; genel .text değişimleri yerini
    // kaydıramaz (0.9.16 GLOBAL_SLOTS yerleşim kayması bir daha oynayamaz).
    // GNU linker script sadece Linux ELF hedefleri için geçerlidir (Windows
    // MSVC -Wl,-T tanımaz ve LNK1181 hatası verir, macOS ld farklıdır).
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let local_ld = manifest_dir.join("hudhud_hot.ld");
        let root_ld = manifest_dir.join("../../scripts/link/hudhud_hot.ld");

        let ld = if local_ld.exists() {
            Some(local_ld)
        } else if root_ld.exists() {
            Some(root_ld)
        } else {
            None
        };

        if let Some(ld_path) = ld {
            println!("cargo:rustc-link-arg-bins=-Wl,-T,{}", ld_path.display());
            println!("cargo:rerun-if-changed={}", ld_path.display());
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}

//! Linux test ikililerine -Wl,--export-dynamic: MCJIT'in bellekte derlediği
//! kod, süreç genelî ad uzayından hudhud_* helper'larını çözer; ihraç
//! edilmezlerse her çevrimiçi test "undefined symbol" ile düşer. Bayrak
//! YALNIZ bu paketin test hedeflerine verilir — pyo3 gibi export-dynamic ile
//! bağlanamayacak crate'ler etkilenmez (v0.9.22: cargo test --workspace'in
//! tek komutla yeşil olması için). Windows (MSVC) hedefinde verilmez — ELF
//! bayrağıdır. llvm-sys bağlantısı kendi build script'i üzerinden
//! LLVM_SYS_140_PREFIX (bkz. .cargo/config.toml [env]) ile çözülür.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" {
        // -tests: tests/ altındaki entegrasyon testleri; unsuffixed: lib
        // unittest ikilisi (rustc-link-arg-tests onu kapsamaz — MCJIT duman
        // testi SIGSEGV düşerdi). Paketin başka bağlanan artefaktı yok.
        println!("cargo:rustc-link-arg-tests=-Wl,--export-dynamic");
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }
}

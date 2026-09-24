//! libgccjit bağlantısı (öncelik sırası):
//! 1. HUDHUD_GCCJIT_LIBDIR verilmişse o dizin (rpath dahil),
//! 2. sistemde sürümsüz libgccjit.so varsa düz -lgccjit,
//! 3. yalnız sürümlü libgccjit.so.0 varsa (dev paketi sembolik bağ kurmamış)
//!    OUT_DIR'e sembolik bağ kurup -lgccjit ile ararız — env hiç gerekmez.
//!
//! ÇOKLU KONAK/ÇOKLU MİMARİ: bu build script Windows konakta da DERLENMELİ
//! — unix'e özgü API'ler cfg(unix) arkasında; lib arama listesi Debian
//! multiarch dizinlerini (x86_64/aarch64/armv7/arm) kapsar. Sembolik bağ
//! kurulamıyorsa (ör. Windows konak) düz -lgccjit'e düşülür.
//!
//! Linux test ikililerine -Wl,--export-dynamic: libgccjit'in derleyip
//! dlopen ettiği fake.so, süreç genelî ad uzayından hudhud_* helper'larını
//! çözer; ihraç edilmezlerse her çevrimiçi test "undefined symbol" ile
//! düşer. Bayrak YALNIZ bu paketin test hedeflerine verilir — pyo3 gibi
//! export-dynamic ile bağlanamayacak crate'ler etkilenmez (v0.9.22:
//! cargo test --workspace'in tek komutla yeşil olması için).
//! -tests: tests/ entegrasyonları; unsuffixed: lib unittest ikilisi
//! (rustc-link-arg-tests onu kapsamaz). Windows (MSVC) hedefinde verilmez
//! — ELF bayrağıdır.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        return;
    }
    if let Ok(dir) = std::env::var("HUDHUD_GCCJIT_LIBDIR") {
        println!("cargo:rustc-link-lib=dylib=gccjit");
        println!("cargo:rustc-link-search=native={dir}");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");
    } else if let Some(so) = find_system_libgccjit() {
        // OUT_DIR sembolik bağını dene (unix); her durumda -lgccjit ile bağla
        let _ = symlink_impl(&so);
        println!("cargo:rustc-link-lib=dylib=gccjit");
    } else {
        println!("cargo:rustc-link-lib=dylib=gccjit");
    }
    println!("cargo:rerun-if-env-changed=HUDHUD_GCCJIT_LIBDIR");
    if target_os == "linux" {
        println!("cargo:rustc-link-arg-tests=-Wl,--export-dynamic");
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }
}

/// Önce sürümsüz libgccjit.so; yoksa sürümlü libgccjit.so.0 yolunu döner.
/// Debian multiarch dizinleri (x86_64/aarch64/armv7/arm) + genel dizinler.
fn find_system_libgccjit() -> Option<std::path::PathBuf> {
    let dirs = [
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
        "/usr/lib/arm-linux-gnueabihf",
        "/usr/lib/arm-linux-gnueabi",
        "/usr/lib64",
        "/usr/lib",
        "/usr/local/lib",
    ];
    for d in dirs {
        let so = std::path::Path::new(d).join("libgccjit.so");
        if so.exists() {
            return Some(so);
        }
    }
    for d in dirs {
        let so0 = std::path::Path::new(d).join("libgccjit.so.0");
        if so0.exists() {
            return Some(so0);
        }
    }
    None
}

/// Bulunan kütüphaneyi OUT_DIR/libgccjit.so olarak sembolik bağla
/// (-lgccjit sürümsüz adı arar; bağ kurulursa arama dizini eklenir).
/// Yalnız unix konakta; Windows konakta etkisizdir.
#[cfg(unix)]
fn symlink_impl(so: &std::path::Path) -> bool {
    let out_lib =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("libgccjit.so");
    let _ = std::fs::remove_file(&out_lib);
    if std::os::unix::fs::symlink(so, &out_lib).is_ok() {
        let parent = out_lib.parent().unwrap().display();
        println!("cargo:rustc-link-search=native={parent}");
        true
    } else {
        false
    }
}

#[cfg(not(unix))]
fn symlink_impl(_so: &std::path::Path) -> bool {
    false
}

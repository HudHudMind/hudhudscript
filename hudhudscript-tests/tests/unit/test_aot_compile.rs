//! Tests for hudhudscript-aot — source → MIR → object → link → executable.
//!
//! The gccjit/llvm end-to-end builds link against system libraries
//! (libgccjit, LLVM) and are gated behind this package's `aot-backends`
//! feature so plain `cargo test` stays hermetic.

use hudhudscript_aot::{
    build_executable, compile_object_only, refresh_runtime_lib, AotOptLevel as OptLevel,
};

// ── end-to-end executable (cranelift) ─────────────────────────────

// Bu testin runtime kütüphanesini bulması gerekir. Temiz ağaçta
// staticlib .a henüz derlenmemiş olabilir (cargo bağımlılıkları yalnız
// rlib derler) — özyeterlilik (kullanıcı onayı, v0.9.24): mevcut
// refresh_runtime_lib ile bir kez üret, tekrar ara; yine yoksa ortam
// hazırlığı değil üretim doğrulaması olduğu için net hata göster.
#[test]
fn aot_end_to_end_executable() {
    let mut rt = hudhudscript_linker::find_runtime_lib();
    if rt.is_err() {
        if let Err(e) = refresh_runtime_lib("native") {
            panic!(
                "runtime lib missing: {:?} — yenileme de başarısız: {e} — \
                 run cargo build --release -p hudhudscript-native-abi",
                hudhudscript_linker::find_runtime_lib().err()
            );
        }
        rt = hudhudscript_linker::find_runtime_lib();
    }
    let rt = match rt {
        Ok(p) => p,
        Err(e) => panic!(
            "runtime lib missing: {e:?} — \
             run cargo build --release -p hudhudscript-native-abi"
        ),
    };
    let _ = rt; // build_executable kendi find_runtime_lib çağrısını yapar
    let workdir = std::env::temp_dir().join(format!("hudhud_aot_e2e_{}", std::process::id()));
    let out = workdir.join("hudhud_aot_test_bin");
    let result = build_executable(
        "function main() { return 6 * 7 }",
        &out,
        &workdir,
        OptLevel::O2,
        "native",
        "cranelift",
    )
    .expect("aot build");
    assert!(result.executable.is_file());
    assert_eq!(result.entry_symbol, "hudhud_main");
    let status = run_aot_binary(&result.executable);
    // main() 42 döndürür → JitExit.status = RETURNED(0) → exit code 0
    assert!(status.success(), "aot binary exit: {:?}", status.code());
    let _ = std::fs::remove_file(&result.executable);
    let _ = std::fs::remove_file(&result.object);
    let _ = std::fs::remove_dir_all(&workdir);
}

/// Spawn the produced AOT executable, forcing the fork+exec path.
///
/// glibc's posix_spawn (used by `Command` when no pre_exec is configured)
/// is CLONE_VFORK-based; in a multithreaded parent — libtest runs tests on
/// parallel threads — that path has been observed to fail sporadically with
/// EFAULT (one occurrence in a full workspace gate run; the binary itself
/// was a valid ELF that ran fine afterwards). Registering a no-op pre_exec
/// makes std take fork+exec instead, removing the vfork window.
fn run_aot_binary(path: &std::path::Path) -> std::process::ExitStatus {
    let mut cmd = std::process::Command::new(path);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| Ok(()));
        }
    }
    cmd.status().expect("run aot binary")
}

// ── cross-object emission ────────────────────────────────────────

#[test]
fn cross_object_aarch64() {
    let workdir = std::env::temp_dir().join(format!("hudhud_aot_cross_{}", std::process::id()));
    std::fs::create_dir_all(&workdir).unwrap();
    let obj = workdir.join("t_arm64.o");
    let (path, _, _, _) = compile_object_only(
        "print(2 + 3)",
        &obj,
        OptLevel::O2,
        "aarch64-unknown-linux-gnu",
        "cranelift",
    )
    .expect("aarch64 object");
    let bytes = std::fs::read(&path).expect("read");
    assert_eq!(&bytes[0..4], &[0x7f, b'E', b'L', b'F']);
    // aarch64 ELF: e_machine = EM_AARCH64 (183, offset 18, little-endian)
    assert_eq!(&bytes[18..20], &[183, 0]);
    let _ = std::fs::remove_file(&obj);
    let _ = std::fs::remove_dir_all(&workdir);
}

// ── feature-gated backends (aot-backends) ────────────────────────

#[cfg(feature = "aot-backends")]
#[test]
fn gccjit_end_to_end_executable() {
    // gccjit backend: kaynak → GCC object → link → binary (libgccjit gerekir)
    let rt = hudhudscript_linker::find_runtime_lib();
    if rt.is_err() {
        panic!("runtime lib missing: {:?}", rt.err());
    }
    let workdir = std::env::temp_dir().join(format!("hudhud_gccjit_e2e_{}", std::process::id()));
    let out = workdir.join("hudhud_gccjit_test_bin");
    let result = build_executable(
        "function main() { return 6 * 7 }",
        &out,
        &workdir,
        OptLevel::O2,
        "native",
        "gccjit",
    )
    .expect("gccjit aot build");
    assert!(result.executable.is_file());
    let status = run_aot_binary(&result.executable);
    assert!(status.success(), "gccjit binary exit: {:?}", status.code());
    let _ = std::fs::remove_file(&result.executable);
    let _ = std::fs::remove_file(&result.object);
    let _ = std::fs::remove_dir_all(&workdir);
}

#[cfg(feature = "aot-backends")]
#[test]
fn llvm_end_to_end_executable() {
    // LLVM backend: kaynak → LLVM IR → object → link → binary
    let rt = hudhudscript_linker::find_runtime_lib();
    if rt.is_err() {
        panic!("runtime lib missing: {:?}", rt.err());
    }
    let workdir = std::env::temp_dir().join(format!("hudhud_llvm_e2e_{}", std::process::id()));
    let out = workdir.join("hudhud_llvm_test_bin");
    let result = build_executable(
        "function main() { return 6 * 7 }",
        &out,
        &workdir,
        OptLevel::O2,
        "native",
        "llvm",
    )
    .expect("llvm aot build");
    assert!(result.executable.is_file());
    let status = run_aot_binary(&result.executable);
    assert!(status.success(), "llvm binary exit: {:?}", status.code());
    let _ = std::fs::remove_file(&result.executable);
    let _ = std::fs::remove_file(&result.object);
    let _ = std::fs::remove_dir_all(&workdir);
}

// ── object-only emission ─────────────────────────────────────────

#[test]
fn object_only_has_entry() {
    let workdir = std::env::temp_dir().join(format!("hudhud_aot_obj_{}", std::process::id()));
    std::fs::create_dir_all(&workdir).unwrap();
    let obj = workdir.join("t.o");
    let (path, symbols, entry, _entries) =
        compile_object_only("print(2 + 3)", &obj, OptLevel::O2, "native", "cranelift").expect("obj");
    assert!(path.is_file());
    assert!(symbols.contains(&"hudhud__hudhud_init".to_string()));
    assert_eq!(entry, "hudhud__hudhud_init");
    let _ = std::fs::remove_file(&obj);
    let _ = std::fs::remove_dir_all(&workdir);
}

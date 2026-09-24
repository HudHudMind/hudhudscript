//! AOT derleyici (JIT_AOT_ARCHITECTURE.md §J M2):
//! kaynak → AST → HIR → MIR → object (Cranelift) → link → yerel çalıştırılabilir.
//!
//! Object-first: her aşama ayrı ürün üretir; `hudhud compile --emit=obj`
//! yalnız object yazar, `hudhud build --mode=aot` link'e kadar götürür.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use hudhudscript_codegen::backend::{CodegenContext, OptGoal, OptLevel};

pub use hudhudscript_codegen::backend::OptLevel as AotOptLevel;
use hudhudscript_mir::{lower_module_typed, MirModule, MirType};
use hudhudscript_parser::parse;
use hudhudscript_types::lower_module_with_init;

/// AOT build sonucu.
#[derive(Debug)]
pub struct AotResult {
    pub executable: PathBuf,
    pub object: PathBuf,
    pub symbols: Vec<String>,
    pub entry_symbol: String,
}

fn mir_from_source(source: &str) -> Result<MirModule, String> {
    let ast = parse(source).map_err(|e| format!("parse: {e}"))?;
    let mut hir = lower_module_with_init(&ast).map_err(|e| format!("HIR: {e}"))?;
    hudhudscript_mir::specialize_module(&mut hir);
    let ptys = hudhudscript_mir::param_infer::infer_module_param_types(&hir);
    let mut mir = lower_module_typed(&hir, &ptys).map_err(|e| format!("MIR lowering: {e}"))?;
    for f in mir.functions.iter_mut() {
        let (opt, _n) = hudhudscript_mir_opt::optimize(f);
        *f = opt;
    }
    Ok(mir)
}

/// Kaynağı object dosyasına derler (link yok). Dönen: (object yolu,
/// semboller, giriş sembolü, (init, main) giriş çifti — MIR'dan, satır
/// sezgiselinden DEĞİL: main gövdesindeki satırlar top-level sanılamaz).
pub fn compile_object_only(
    source: &str,
    out_path: &Path,
    opt: OptLevel,
    target_triple: &str,
    backend: &str,
) -> Result<(PathBuf, Vec<String>, String, (Option<String>, Option<String>)), String> {
    let mir = mir_from_source(source)?;
    let (init_sym, main_sym) = hudhudscript_codegen_cranelift::entry_symbols(&mir);
    if init_sym.is_none() && main_sym.is_none() {
        return Err("no executable code found (need top-level code or main())".to_string());
    }
    let entry = init_sym.clone().or(main_sym.clone())
        .ok_or_else(|| "no executable code found".to_string())?;
    let target = if target_triple == "native" {
        hudhudscript_target::TargetSpec::host()
    } else {
        hudhudscript_target::parse_triple(target_triple)
            .map_err(|e| format!("{e}"))?
    };
    let ctx = CodegenContext {
        target: &target,
        opt,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let symbols = match backend {
        #[cfg(feature = "gccjit")]
        "gccjit" => hudhudscript_codegen_gccjit::compile_to_object(&mir, &ctx, out_path)
            .map_err(|e| format!("gccjit object emission: {e}"))?,
        #[cfg(not(feature = "gccjit"))]
        "gccjit" => {
            return Err(
                "gccjit backend requires --features gccjit (libgccjit link gerektirir)".to_string(),
            )
        }
        #[cfg(feature = "llvm")]
        "llvm" => hudhudscript_codegen_llvm::compile_to_object(&mir, &ctx, out_path)
            .map_err(|e| format!("llvm object emission: {e}"))?,
        #[cfg(not(feature = "llvm"))]
        "llvm" => {
            return Err(
                "llvm backend requires --features llvm (LLVM 14 link gerektirir)".to_string(),
            )
        }
        "cranelift" | "auto" => hudhudscript_codegen_cranelift::compile_to_object(&mir, &ctx, out_path)
            .map_err(|e| format!("object emission: {e}"))?,
        other => return Err(format!(
            "unknown backend `{other}` — available: auto, cranelift, gccjit, llvm"
        )),
    };
    Ok((out_path.to_path_buf(), symbols, entry, (init_sym, main_sym)))
}

/// Kaynaktan tam yerel çalıştırılabilir üretir (object + link).
pub fn build_executable(
    source: &str,
    out_path: &Path,
    workdir: &Path,
    opt: OptLevel,
    target_triple: &str,
    backend: &str,
) -> Result<AotResult, String> {
    std::fs::create_dir_all(workdir)
        .map_err(|e| format!("mkdir {}: {e}", workdir.display()))?;
    let stem = out_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("hudhud_out");
    let object = workdir.join(format!("{stem}.o"));
    let (obj, symbols, entry, (init_sym, main_sym)) =
        compile_object_only(source, &object, opt, target_triple, backend)?;
    // F19: init VE main MIR'dan çıkar (compile_object_only döndürür) — satır
    // sezgiseli main-gövde satırlarını top-level sanıp init'siz objeye init
    // referansı yazdırıyordu (link hatası).
    let link = match hudhudscript_linker::link_executable(
        &[obj.clone()],
        out_path,
        (init_sym.as_deref(), main_sym.as_deref()),
        workdir,
        target_triple,
    ) {
        Ok(l) => l,
        Err(e) if is_stale_runtime(&e) => {
            // Bayat .a: geliştirme makinesindeyse otomatik yenile (cargo
            // PATH'te + HUDHUD_NO_AUTO_REFRESH yoksa), sonra bir kez daha dene
            refresh_runtime_lib(target_triple)?;
            hudhudscript_linker::link_executable(
                &[obj.clone()],
                out_path,
                (init_sym.as_deref(), main_sym.as_deref()),
                workdir,
                target_triple,
            )?
        }
        Err(e) => return Err(e),
    };
    Ok(AotResult {
        executable: link.executable,
        object: obj,
        symbols,
        entry_symbol: entry,
    })
}

/// Linker'in bayat-runtime hatası mı (ensure_runtime_fresh mesaj damgası)?
fn is_stale_runtime(err: &str) -> bool {
    err.contains("BAYAT")
}

/// Statik runtime kütüphanesini cargo ile yeniden derle (konak veya cross).
/// Cargo yoksa (kurulu ikili) anlaşılır hata — talimat mesajda korunur.
fn refresh_runtime_lib(target_triple: &str) -> Result<(), String> {
    if std::env::var_os("HUDHUD_NO_AUTO_REFRESH").is_some() {
        return Err("otomatik yenileme HUDHUD_NO_AUTO_REFRESH ile kapatılmış".into());
    }
    let cargo = std::env::var_os("PATH")
        .and_then(|p| {
            p.to_string_lossy()
                .split(':')
                .map(|d| std::path::PathBuf::from(d).join("cargo"))
                .find(|c| c.is_file())
        })
        .ok_or_else(|| {
            "cargo PATH'te değil — runtime'ı elle yenileyin:\n  cargo build --release -p hudhudscript-native-abi".to_string()
        })?;
    let mut cmd = std::process::Command::new(&cargo);
    cmd.args(["build", "--release", "-p", "hudhudscript-native-abi"]);
    if target_triple != "native" && !target_triple.is_empty() {
        cmd.args(["--target", target_triple]);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("cargo başlatılamadı ({}): {e}", cargo.display()))?;
    if !out.status.success() {
        return Err(format!(
            "runtime yenileme başarısız (cargo exit {}):\n{}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let status = std::process::Command::new(&result.executable)
            .status()
            .expect("run aot binary");
        // main() 42 döndürür → JitExit.status = RETURNED(0) → exit code 0
        assert!(status.success(), "aot binary exit: {:?}", status.code());
        let _ = std::fs::remove_file(&result.executable);
        let _ = std::fs::remove_file(&result.object);
        let _ = std::fs::remove_dir_all(&workdir);
    }

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

    #[cfg(feature = "gccjit")]
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
        let status = std::process::Command::new(&result.executable)
            .status()
            .expect("run gccjit binary");
        assert!(status.success(), "gccjit binary exit: {:?}", status.code());
        let _ = std::fs::remove_file(&result.executable);
        let _ = std::fs::remove_file(&result.object);
        let _ = std::fs::remove_dir_all(&workdir);
    }

    #[cfg(feature = "llvm")]
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
        let status = std::process::Command::new(&result.executable)
            .status()
            .expect("run llvm binary");
        assert!(status.success(), "llvm binary exit: {:?}", status.code());
        let _ = std::fs::remove_file(&result.executable);
        let _ = std::fs::remove_file(&result.object);
        let _ = std::fs::remove_dir_all(&workdir);
    }

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
}



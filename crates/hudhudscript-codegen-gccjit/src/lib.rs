//! gccjit backend (JIT_AOT_ARCHITECTURE.md §J M4): MIR → libgccjit →
//! object dosyası / çalıştırılabilir. ARMv7 gömülü hattının birincil
//! backend'i (cranelift 32-bit ARM üretmez).
//!
//! Ortam: `apt install libgccjit-12-dev` veya
//! `HUDHUD_GCCJIT_LIBDIR=<dir>` (libgccjit.so içeren dizin).

#[cfg(unix)]
mod abi;
#[cfg(unix)]
mod translate;

use std::path::Path;

use hudhudscript_codegen::backend::{BackendError, CodegenContext};
use hudhudscript_mir::MirModule;

#[cfg(unix)]
use hudhudscript_codegen::backend::{OptGoal, OptLevel};

/// MIR modülünü object dosyasına derler (gccjit OutputKind::Object).
#[cfg(unix)]
pub fn compile_to_object(
    module: &MirModule,
    ctx: &CodegenContext<'_>,
    out_path: &Path,
) -> Result<Vec<String>, BackendError> {
    let level = match (ctx.opt, ctx.opt_goal) {
        (OptLevel::O0, _) => gccjit::OptimizationLevel::None,
        (OptLevel::O1, _) => gccjit::OptimizationLevel::Limited,
        (OptLevel::O3, OptGoal::Speed) => gccjit::OptimizationLevel::Aggressive,
        _ => gccjit::OptimizationLevel::Standard,
    };

    let gcx = gccjit::Context::default();
    gcx.set_optimization_level(level);
    if std::env::var("HUDHUD_GCCJIT_DUMP").is_ok() {
        gcx.set_dump_code_on_compile(true);
    }
    // Not: libgccjit çALIŞTIĞI makinenin mimarisine göre derler; ARMv7
    // cross-derleme hedefte (veya hedefe derlenmiş libgccjit ile) yapılır.
    let _ = &ctx.target.triple;

    let abi = translate::build_abi(&gcx);
    let ext = translate::declare_exts(&abi);
    let module_funcs = translate::declare_module_functions(&abi, module)
        .map_err(|e: hudhudscript_codegen::backend::BackendError| e.in_function("module".to_string()))?;
    let mut symbols = Vec::new();
    for (i, f) in module.functions.iter().enumerate() {
        let symbol = translate::translate_function(&abi, module, f, i, &module_funcs, &ext)
            .map_err(|e: hudhudscript_codegen::backend::BackendError| e.in_function(f.name.to_string()))?;
        symbols.push(symbol);
    }

    gcx.compile_to_file(gccjit::OutputKind::ObjectFile, out_path.display().to_string());
    // gccjit hataları stderr'e düşer ve dosya oluşmaz — varlığı doğrula
    if !out_path.is_file() {
        return Err(BackendError::new(
            "EMIT_FAIL",
            format!("gccjit produced no object at {}", out_path.display()),
        ));
    }
    Ok(symbols)
}

#[cfg(not(unix))]
pub fn compile_to_object(
    _module: &MirModule,
    _ctx: &CodegenContext<'_>,
    _out_path: &Path,
) -> Result<Vec<String>, BackendError> {
    Err(BackendError::new(
        "UNSUPPORTED",
        "gccjit backend is only supported on Unix platforms (gccjit crate is Unix-only)",
    ))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use hudhudscript_mir::{lower_module_typed, MirModule, MirType};
    use hudhudscript_parser::parse;
    use hudhudscript_types::lower_module_with_init;
    use std::collections::HashMap;

    fn mir_of(src: &str) -> MirModule {
        let ast = parse(src).expect("parse");
        let hir = lower_module_with_init(&ast).expect("hir");
        let mut ptys: HashMap<String, HashMap<String, MirType>> = HashMap::new();
        for (name, f) in &hir.functions {
            let m: HashMap<String, MirType> =
                f.params.iter().map(|p| (p.name.clone(), MirType::I64)).collect();
            ptys.insert(name.clone(), m);
        }
        lower_module_typed(&hir, &ptys).expect("mir")
    }

    #[test]
    fn emits_valid_elf_object() {
        let mir = mir_of("print(2 + 3)");
        let target = hudhudscript_target::TargetSpec::host();
        let ctx = CodegenContext {
            target: &target,
            opt: OptLevel::O2,
            opt_goal: OptGoal::Speed,
            debug_info: false,
            abi_version: 1,
        };
        let out = std::env::temp_dir().join(format!("hudhud_gccjit_{}.o", std::process::id()));
        let symbols = compile_to_object(&mir, &ctx, &out).expect("gccjit object");
        assert!(!symbols.is_empty());
        let bytes = std::fs::read(&out).expect("read back");
        assert!(bytes.len() > 64);
        assert_eq!(&bytes[0..4], &[0x7f, b'E', b'L', b'F']);
        let _ = std::fs::remove_file(&out);
    }
}

/// JIT modu: Context::compile() ile bellekte çalıştırılabilir üretir;
/// sembol adreslerini döner. CompileResult'ın ömrü kod belleğine aittir —
/// süreç kapanana kadar yaşatılır (cranelift backend'inin mem::forget
/// örüntüsüyle aynı).
#[cfg(unix)]
pub fn compile_jit(
    module: &MirModule,
    ctx: &CodegenContext<'_>,
) -> Result<Vec<(String, usize)>, BackendError> {
    let gcx = gccjit::Context::default();
    gcx.set_optimization_level(gccjit::OptimizationLevel::Standard);
    let _ = &ctx.target.triple;
    let abi = translate::build_abi(&gcx);
    let ext = translate::declare_exts(&abi);
    let module_funcs = translate::declare_module_functions(&abi, module)
        .map_err(|e| e.in_function("module".to_string()))?;
    let mut symbols = Vec::new();
    for (i, f) in module.functions.iter().enumerate() {
        let symbol = translate::translate_function(&abi, module, f, i, &module_funcs, &ext)
            .map_err(|e: BackendError| e.in_function(f.name.to_string()))?;
        symbols.push(symbol);
    }
    let result = gcx.compile();
    let mut out = Vec::new();
    for s in &symbols {
        let p = result.get_function(s);
        if p.is_null() {
            return Err(BackendError::new("LOOKUP_FAIL", format!("gccjit: {s} missing")));
        }
        out.push((s.clone(), p as usize));
    }
    std::mem::forget(result);
    Ok(out)
}

#[cfg(not(unix))]
pub fn compile_jit(
    _module: &MirModule,
    _ctx: &CodegenContext<'_>,
) -> Result<Vec<(String, usize)>, BackendError> {
    Err(BackendError::new(
        "UNSUPPORTED",
        "gccjit backend is only supported on Unix platforms (gccjit crate is Unix-only)",
    ))
}

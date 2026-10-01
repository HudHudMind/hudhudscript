//! LLVM backend (JIT_AOT_ARCHITECTURE.md §J M5): MIR → LLVM IR → object.
//!
//! Ortam: LLVM 14 (llvm-14-dev) + statik Polly (debian: libpolly-14-dev).
//! `LLVM_SYS_140_PREFIX` bir LLVM prefix'i gösterir (libLLVM + libPolly.a +
//! başlıklar). Cross-target: TargetMachine triple'ı ile (armv7 dâhil —
//! LLVM 14 armv7 kod üretir; gccjit'ten farkı budur).

#[cfg(unix)]
mod translate;

#[cfg(unix)]
pub use translate::translate_function;

use std::path::Path;

use hudhudscript_codegen::backend::{BackendError, CodegenContext};
use hudhudscript_mir::MirModule;

#[cfg(unix)]
use hudhudscript_codegen::backend::{OptGoal, OptLevel};
#[cfg(unix)]
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
#[cfg(unix)]
use inkwell::OptimizationLevel;

/// MIR modülünü object dosyasına derler.
#[cfg(unix)]
pub fn compile_to_object(
    module: &MirModule,
    ctx: &CodegenContext<'_>,
    out_path: &Path,
) -> Result<Vec<String>, BackendError> {
    let level = match (ctx.opt, ctx.opt_goal) {
        (OptLevel::O0, _) => OptimizationLevel::None,
        (OptLevel::O1, _) => OptimizationLevel::Less,
        (OptLevel::O3, OptGoal::Speed) => OptimizationLevel::Aggressive,
        _ => OptimizationLevel::Default,
    };

    Target::initialize_all(&InitializationConfig::default());

    let context = inkwell::context::Context::create();
    let llvm_module = context.create_module("hudhud");

    let triple = if ctx.target.triple == "native" {
        TargetMachine::get_default_triple()
    } else {
        inkwell::targets::TargetTriple::create(&ctx.target.triple)
    };

    let mut symbols = Vec::new();
    for (i, f) in module.functions.iter().enumerate() {
        // Module<'ctx> — llvm_module context'ten türediği için ömürler aynı
        let symbol = translate::translate_function(&context, &llvm_module, module, f, i)
            .map_err(|e: hudhudscript_codegen::backend::BackendError| e.in_function(f.name.to_string()))?;
        symbols.push(symbol);
    }
    llvm_module.verify().map_err(|e| {
        BackendError::new("VERIFY_FAIL", format!("llvm module verification: {e}"))
    })?;

    let target = Target::from_triple(&triple)
        .map_err(|e| BackendError::new("ISA_FAIL", format!("llvm target: {e}")))?;
    let tm = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            level,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| BackendError::new("ISA_FAIL", "llvm target machine creation failed"))?;
    llvm_module.set_triple(&triple);
    llvm_module.verify().map_err(|e| {
        BackendError::new("VERIFY_FAIL", format!("llvm module verification: {e}"))
    })?;
    tm.write_to_file(&llvm_module, FileType::Object, out_path)
        .map_err(|e| BackendError::new("EMIT_FAIL", format!("llvm object emit: {e}")))?;
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
        "llvm backend is only supported on Unix platforms (LLVM 14 development packages required)",
    ))
}

// Inline tests forbidden — all tests live in hudhud-script-tests/tests/

/// JIT modu (M6): MCJIT ExecutionEngine — modül bellekte derlenir,
/// sembol adresleri döner. Harici hudhud_* sembolleri süreç sembol
/// tablosundan çözülür (konak ikili -rdynamic ile derlenmelidir).
#[cfg(unix)]
pub fn compile_jit(
    module: &MirModule,
    ctx: &CodegenContext<'_>,
) -> Result<Vec<(String, usize)>, BackendError> {
    // Statik LLVM'de native-only shim release linkte boş kalabiliyor;
    // initialize_all hedef kayıtlarını doğrudan referanslar (AOT yoluyla aynı).
    inkwell::targets::Target::initialize_all(&inkwell::targets::InitializationConfig::default());
    let context = std::mem::ManuallyDrop::new(inkwell::context::Context::create());
    let llvm_module = context.create_module("hudhud_jit");
    // MCJIT konak üçlüsünü bilmeli (AOT yolunda set edilir; JIT'te de şart)
    llvm_module.set_triple(&inkwell::targets::TargetMachine::get_default_triple());
    let mut symbols = Vec::new();
    for (i, f) in module.functions.iter().enumerate() {
        let symbol = translate::translate_function(&context, &llvm_module, module, f, i)
            .map_err(|e: BackendError| e.in_function(f.name.to_string()))?;
        symbols.push(symbol);
    }
    // MCJIT geçersiz modülde sessizce üretim yapamaz (semboller kaybolur) —
    // verify ile kök neden dürüstçe raporlanır
    if let Err(e) = llvm_module.verify() {
        return Err(BackendError::new(
            "VERIFY_FAIL",
            format!("llvm module verification: {e}"),
        ));
    }
    // MCJIT oluşturma BAŞARISIZ olsa bile LLVM modül sahipliğini kısmen
    // tüketebilir — hata yolunda da leak edilir (drop = çifte-dispose segv).
    let ee = match llvm_module.create_jit_execution_engine(level_of(ctx)) {
        Ok(ee) => ee,
        Err(e) => {
            std::mem::forget(llvm_module);
            return Err(BackendError::new("JIT_FAIL", format!("llvm ee: {e}")));
        }
    };
    let mut out = Vec::new();
    for s in &symbols {
        let addr = match ee.get_function_address(s) {
            Ok(a) => a,
            Err(e) => {
                std::mem::forget(ee);
                std::mem::forget(llvm_module);
                return Err(BackendError::new("LOOKUP_FAIL", format!("llvm: {s}: {e}")));
            }
        };
        out.push((s.clone(), addr));
    }
    std::mem::forget(ee);
    std::mem::forget(llvm_module);
    Ok(out)
}

#[cfg(not(unix))]
pub fn compile_jit(
    _module: &MirModule,
    _ctx: &CodegenContext<'_>,
) -> Result<Vec<(String, usize)>, BackendError> {
    Err(BackendError::new(
        "UNSUPPORTED",
        "llvm backend is only supported on Unix platforms (LLVM 14 development packages required)",
    ))
}

#[cfg(unix)]
fn level_of(ctx: &CodegenContext<'_>) -> OptimizationLevel {
    match (ctx.opt, ctx.opt_goal) {
        (OptLevel::O0, _) => OptimizationLevel::None,
        (OptLevel::O1, _) => OptimizationLevel::Less,
        (OptLevel::O3, OptGoal::Speed) => OptimizationLevel::Aggressive,
        _ => OptimizationLevel::Default,
    }
}

// Inline tests forbidden — all tests live in hudhud-script-tests/tests/

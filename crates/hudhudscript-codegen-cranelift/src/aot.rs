//! Cranelift AOT: MIR → yerel object dosyası (ELF/COFF/MachO).
//! JIT_AOT_ARCHITECTURE.md §J M2 — object-first; link ayrı adımda
//! (hudhudscript-linker driver).

use std::path::Path;

use cranelift::prelude::codegen::settings::{self, Configurable};
use cranelift::prelude::types::I32;
use cranelift::prelude::{AbiParam, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use hudhudscript_codegen::backend::{BackendError, CodegenContext, OptGoal, OptLevel};
use hudhudscript_mir::MirModule;

use crate::translate::translate_with_module;

/// MIR modülünü tek object dosyasına derler. Dönen değer: üretilen
/// sembol adları (hudhud_*). Harici helper sembolleri (hudhud_print_int
/// vb.) Linkage::Import olarak kalır; link aşamasında runtime
/// kütüphanesinden çözülür.
pub fn compile_to_object(
    module: &MirModule,
    ctx: &CodegenContext<'_>,
    out_path: &Path,
) -> Result<Vec<String>, BackendError> {
    // AOT: host finalize anı yok → ConstString kaydı inline (binary kendi
    // kaydeder; JIT yolu host-side register_finalized_strings kullanır)
    crate::translate::set_inline_string_reg(true);
    let result = compile_to_object_inner(module, ctx, out_path);
    crate::translate::set_inline_string_reg(false);
    result
}

fn compile_to_object_inner(
    module: &MirModule,
    ctx: &CodegenContext<'_>,
    out_path: &Path,
) -> Result<Vec<String>, BackendError> {
    let opt_level = match (ctx.opt, ctx.opt_goal) {
        (OptLevel::O0, _) => "none",
        (OptLevel::O1, _) => "fast",
        (_, OptGoal::Size) | (_, OptGoal::SizeMin) => "speed_and_size",
        _ => "speed",
    };
    let mut flags = settings::builder();
    flags
        .set("opt_level", opt_level)
        .map_err(|e| BackendError::new("ISA_FAIL", format!("opt_level: {e}")))?;
    // Hedef ISA: native → konak; triple → cross (Cranelift x64/aarch64/
    // s390x/riscv64 üretir; armv7 gccjit/LLVM backend'lerinin hattıdır)
    let isa_builder = if ctx.target.triple == "native" {
        cranelift_native::builder()
            .map_err(|e| BackendError::new("ISA_FAIL", format!("native isa: {e}")))?
    } else {
        cranelift::prelude::isa::lookup_by_name(&ctx.target.triple).map_err(|e| {
            BackendError::new(
                "ISA_FAIL",
                format!(
                    "cross target `{}` unsupported by cranelift ({e}); armv7 awaits the gccjit/llvm backends",
                    ctx.target.triple
                ),
            )
        })?
    };
    let isa = isa_builder
        .finish(settings::Flags::new(flags))
        .map_err(|e| BackendError::new("ISA_FAIL", format!("isa finish: {e}")))?;

    let builder = ObjectBuilder::new(
        isa,
        "hudhud".to_string(),
        default_libcall_names(),
    )
    .map_err(|e| BackendError::new("OBJECT_FAIL", format!("object builder: {e}")))?;
    let mut obj = ObjectModule::new(builder);
    let ptr = obj.isa().pointer_type();

    // 1) TÜM fonksiyonları ÖNCE declare et (CallStatic ileri referans;
    //    uniform giriş ABI'si: (argc: i32, args: ptr, out: ptr) → void)
    let mut func_ids: Vec<cranelift_module::FuncId> = Vec::new();
    for f in &module.functions {
        let mut sig = obj.make_signature();
        sig.params.push(AbiParam::new(I32));
        sig.params.push(AbiParam::new(ptr));
        sig.params.push(AbiParam::new(ptr));
        let symbol = format!("hudhud_{}", f.name);
        let id = obj
            .declare_function(&symbol, cranelift_module::Linkage::Export, &sig)
            .map_err(|e| {
                BackendError::new("DECLARE_FAIL", format!("declare {symbol}: {e}"))
                    .in_function(f.name.to_string())
            })?;
        func_ids.push(id);
    }

    // 2) Her fonksiyonu çevir (translate Module-trait generic'idir)
    let mut symbols = Vec::new();
    for f in &module.functions {
        let mut fc = FunctionBuilderContext::new();
        let symbol = translate_with_module(&mut obj, &mut fc, f, true, &func_ids)?;
        symbols.push(symbol);
    }

    // 3) Object'i üret ve yaz
    let product = obj.finish();
    let bytes = product
        .emit()
        .map_err(|e| BackendError::new("EMIT_FAIL", format!("object emit: {e}")))?;
    std::fs::write(out_path, bytes)
        .map_err(|e| BackendError::new("IO_FAIL", format!("write {}: {e}", out_path.display())))?;
    Ok(symbols)
}

/// AOT giriş protokolü (F19 — JIT ile aynı): üst-düzey kod VARSA
/// init koşar; init başarılıysa VE main DA varsa main koşar.
/// Dönen sembol sırası: (init, main) — linker shim ikisini sırayla çağırır.
pub fn entry_symbols(module: &MirModule) -> (Option<String>, Option<String>) {
    let init = module.functions.iter()
        .any(|f| f.name.as_ref() == "_hudhud_init")
        .then(|| "hudhud__hudhud_init".to_string());
    let main = module.functions.iter()
        .any(|f| f.name.as_ref() == "main")
        .then(|| "hudhud_main".to_string());
    (init, main)
}

/// Tek sembol (geriye dönük uyum): init > main
pub fn entry_symbol(module: &MirModule) -> Option<String> {
    let (init, main) = entry_symbols(module);
    init.or(main)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hudhudscript_mir::{lower_module_typed, MirType};
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
    fn emits_valid_object_file() {
        let mir = mir_of("print(2 + 3)");
        let target = hudhudscript_target::TargetSpec::host();
        let ctx = CodegenContext {
            target: &target,
            opt: OptLevel::O2,
            opt_goal: OptGoal::Speed,
            debug_info: false,
            abi_version: 1,
        };
        let out = std::env::temp_dir().join(format!("hudhud_aot_test_{}.o", std::process::id()));
        let symbols = compile_to_object(&mir, &ctx, &out).expect("object");
        assert!(!symbols.is_empty());
        let bytes = std::fs::read(&out).expect("read back");
        // ELF magic (Linux host) — object üretilmiş ve yazılmış
        assert!(bytes.len() > 64);
        assert_eq!(&bytes[0..4], &[0x7f, b'E', b'L', b'F']);
        assert_eq!(entry_symbol(&mir).as_deref(), Some("hudhud__hudhud_init"));
        let _ = std::fs::remove_file(&out);
    }
}

//! `CraneliftBackend` — NativeBackend implementation over cranelift-jit.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use cranelift_codegen::isa::TargetIsa;
use cranelift::prelude::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use hudhudscript_codegen::backend::{
    BackendError, CodegenContext, CompiledFunction, CompiledModule, NativeBackend, OptGoal,
};
use hudhudscript_codegen::jit::JitConfig;
use hudhudscript_codegen::capabilities::BackendCapabilities;
use hudhudscript_codegen::jit::{JitEngine, NativeAddress, NativeFunction};
use hudhudscript_mir::{MirFunction, MirModule};
use hudhudscript_target::{Architecture, TargetSpec};

/// Retrieves or builds a cached TargetIsa for host execution.
fn get_target_isa(opt_level: &str) -> Result<Arc<dyn TargetIsa>, BackendError> {
    if opt_level == "speed" {
        static ISA_SPEED: OnceLock<Result<Arc<dyn TargetIsa>, String>> = OnceLock::new();
        let cached = ISA_SPEED.get_or_init(|| {
            let mut flags = settings::builder();
            flags.set("opt_level", "speed").map_err(|e| format!("opt_level: {e}"))?;
            let isa_builder = cranelift_native::builder().map_err(|e| format!("native isa: {e}"))?;
            isa_builder.finish(settings::Flags::new(flags)).map_err(|e| format!("isa finish: {e}"))
        });
        return cached
            .as_ref()
            .map(Arc::clone)
            .map_err(|e| BackendError::new("ISA_FAIL", e.clone()));
    }
    if opt_level == "none" {
        static ISA_NONE: OnceLock<Result<Arc<dyn TargetIsa>, String>> = OnceLock::new();
        let cached = ISA_NONE.get_or_init(|| {
            let mut flags = settings::builder();
            flags.set("opt_level", "none").map_err(|e| format!("opt_level: {e}"))?;
            let isa_builder = cranelift_native::builder().map_err(|e| format!("native isa: {e}"))?;
            isa_builder.finish(settings::Flags::new(flags)).map_err(|e| format!("isa finish: {e}"))
        });
        return cached
            .as_ref()
            .map(Arc::clone)
            .map_err(|e| BackendError::new("ISA_FAIL", e.clone()));
    }
    let mut flags = settings::builder();
    flags
        .set("opt_level", opt_level)
        .map_err(|e| BackendError::new("ISA_FAIL", format!("opt_level: {e}")))?;
    let isa_builder = cranelift_native::builder()
        .map_err(|e| BackendError::new("ISA_FAIL", format!("native isa: {e}")))?;
    isa_builder
        .finish(settings::Flags::new(flags))
        .map_err(|e| BackendError::new("ISA_FAIL", format!("isa finish: {e}")))
}

fn opt_str_for(ctx: &CodegenContext<'_>) -> &'static str {
    match (ctx.opt, ctx.opt_goal) {
        (hudhudscript_codegen::backend::OptLevel::O0, _) => "none",
        (_, hudhudscript_codegen::backend::OptGoal::Size)
        | (_, hudhudscript_codegen::backend::OptGoal::SizeMin) => "speed_and_size",
        _ => "speed",
    }
}

/// Host-ISA Cranelift backend (first-light capabilities: JIT only).
pub struct CraneliftBackend {
    symbols: HashMap<String, NativeAddress>,
}

impl CraneliftBackend {
    pub fn new() -> Self {
        CraneliftBackend { symbols: HashMap::new() }
    }

    fn make_jit_module(opt_level: &str) -> Result<JITModule, BackendError> {
        let isa = get_target_isa(opt_level)?;
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        crate::symbols::register_symbols(&mut builder);
        Ok(JITModule::new(builder))
    }
}

impl Default for CraneliftBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeBackend for CraneliftBackend {
    fn name(&self) -> &'static str {
        "cranelift"
    }

    fn capabilities(&self) -> BackendCapabilities {
        // İlk ışık: yalnız JIT (host ISA). AOT object şeridi
        // hudhudscript-aot adımıyla açılır; burada dürüst reddedilir.
        BackendCapabilities {
            jit: true,
            aot: false,
            cross_compile: false,
            debug_info: false,
            unwind: false,
            exceptions: false,
            vector: false,
            atomics: false,
            gc_stack_maps: false,
        }
    }

    fn supports_target(&self, target: &TargetSpec) -> bool {
        if target.pointer_width != 8 {
            return false;
        }
        if !matches!(target.arch, Architecture::X86_64 | Architecture::Aarch64 | Architecture::Riscv64) {
            return false;
        }
        // cranelift-native yalnız konak ISA üretir: JIT için hedef=host.
        target.triple == TargetSpec::host().triple
    }

    fn compile_function(
        &mut self,
        func: &MirFunction,
        ctx: &CodegenContext<'_>,
    ) -> Result<CompiledFunction, BackendError> {
        let mut module = Self::make_jit_module(opt_str_for(ctx))?;
        let mut func_ctx = cranelift::prelude::FunctionBuilderContext::new();
        let symbol = crate::translate::translate(&mut module, &mut func_ctx, func)?;

        module
            .finalize_definitions()
            .map_err(|e| BackendError::new("FINALIZE_FAIL", format!("{e}")).in_function(func.name.to_string()))?;
        crate::translate::register_finalized_strings(&mut module);
        let func_id = match module.get_name(&symbol) {
            Some(cranelift_module::FuncOrDataId::Func(id)) => id,
            _ => {
                return Err(BackendError::new("LOOKUP_FAIL", format!("symbol {symbol} missing"))
                    .in_function(func.name.to_string()))
            }
        };
        let code_ptr = module.get_finalized_function(func_id);
        if code_ptr.is_null() {
            return Err(BackendError::new("LOOKUP_FAIL", format!("code for {symbol} missing"))
                .in_function(func.name.to_string()));
        }
        let address = code_ptr as NativeAddress;
        self.symbols.insert(symbol.clone(), address);
        // Kod belleği JITModule'a aittir; derlenen sembolün çağrılabilir
        // kalması için modül ömrü uzatılır (sahiplik modeli JitExit
        // şeridinde ModuleHost'a taşınacak).
        std::mem::forget(module);
        Ok(CompiledFunction { symbol, address: Some(address), code_size: 0 })
    }

    fn compile_module(
        &mut self,
        module: &MirModule,
        ctx: &CodegenContext<'_>,
    ) -> Result<CompiledModule, BackendError> {
        let mut jit = Self::make_jit_module(opt_str_for(ctx))?;
        let ptr = jit.isa().pointer_type();

        let mut uniform_sig = jit.make_signature();
        uniform_sig.params.push(cranelift::prelude::AbiParam::new(
            cranelift::prelude::types::I32,
        ));
        uniform_sig.params.push(cranelift::prelude::AbiParam::new(ptr));
        uniform_sig.params.push(cranelift::prelude::AbiParam::new(ptr));

        // 1) TÜM fonksiyonları ÖNCE declare et (CallStatic ileri referans)
        let mut func_ids: Vec<cranelift_module::FuncId> = Vec::with_capacity(module.functions.len());
        for f in &module.functions {
            let symbol = format!("hudhud_{}", f.name);
            let id = jit
                .declare_function(&symbol, cranelift_module::Linkage::Export, &uniform_sig)
                .map_err(|e| {
                    BackendError::new("DECLARE_FAIL", format!("declare {symbol}: {e}"))
                        .in_function(f.name.to_string())
                })?;
            func_ids.push(id);
        }

        // 2) Her fonksiyonu çevir (CallStatic → declare edilmiş FuncId'ye call)
        let mut results = Vec::with_capacity(module.functions.len());
        let mut fc = cranelift::prelude::FunctionBuilderContext::new();
        for f in module.functions.iter() {
            let symbol = crate::translate::translate_in_module(
                &mut jit,
                &mut fc,
                f,
                &func_ids,
                module,
            )?;
            results.push(CompiledFunction {
                symbol,
                address: None,
                code_size: 0,
            });
        }

        // 3) Finalize
        jit.finalize_definitions().map_err(|e| {
            BackendError::new("FINALIZE_FAIL", format!("{e}"))
        })?;
        // String sabitlerini STRING_REGISTRY'ye host tarafında bir kez kaydet
        crate::translate::register_finalized_strings(&mut jit);

        for (i, f) in module.functions.iter().enumerate() {
            #[allow(clippy::needless_range_loop)]
            let _ = i;
            let code = jit.get_finalized_function(func_ids[i]);
            if code.is_null() {
                return Err(BackendError::new("LOOKUP_FAIL", format!("code for hudhud_{} missing", f.name))
                    .in_function(f.name.to_string()));
            }
            let addr = code as NativeAddress;
            let symbol = format!("hudhud_{}", f.name);
            self.symbols.insert(symbol.clone(), addr);
            results[i].address = Some(addr);
        }

        std::mem::forget(jit);
        Ok(CompiledModule {
            functions: results,
            abi_version: ctx.abi_version,
            backend_name: "cranelift".into(),
            target_triple: ctx.target.triple.clone(),
        })
    }

    fn emit_object(
        &mut self,
        module: &MirModule,
        output: &Path,
        ctx: &CodegenContext<'_>,
    ) -> Result<(), BackendError> {
        crate::aot::compile_to_object(module, ctx, output)?;
        Ok(())
    }

    fn create_jit(&self, _config: &JitConfig) -> Result<Box<dyn JitEngine>, BackendError> {
        Ok(Box::new(CraneliftJit::new()?))
    }
}

/// JitEngine implementation owning its JITModule lifetime properly.
struct CraneliftJit {
    module: JITModule,
    func_ctx: cranelift::prelude::FunctionBuilderContext,
    symbols: HashMap<String, NativeAddress>,
    counter: u32,
}

impl CraneliftJit {
    fn new() -> Result<Self, BackendError> {
        let module = CraneliftBackend::make_jit_module("speed")?;
        Ok(CraneliftJit {
            module,
            func_ctx: cranelift::prelude::FunctionBuilderContext::new(),
            symbols: HashMap::new(),
            counter: 0,
        })
    }
}

impl JitEngine for CraneliftJit {
    fn compile(&mut self, function: &MirFunction) -> Result<NativeFunction, BackendError> {
        let symbol = crate::translate::translate(&mut self.module, &mut self.func_ctx, function)?;
        self.module.finalize_definitions().map_err(|e| {
            BackendError::new("FINALIZE_FAIL", format!("{e}")).in_function(function.name.to_string())
        })?;
        crate::translate::register_finalized_strings(&mut self.module);
        let func_id = match self.module.get_name(&symbol) {
            Some(cranelift_module::FuncOrDataId::Func(id)) => id,
            _ => {
                return Err(BackendError::new("LOOKUP_FAIL", format!("symbol {symbol} missing"))
                    .in_function(function.name.to_string()))
            }
        };
        let code_ptr = self.module.get_finalized_function(func_id);
        if code_ptr.is_null() {
            return Err(BackendError::new("LOOKUP_FAIL", format!("code for {symbol} missing"))
                .in_function(function.name.to_string()));
        }
        let address = code_ptr as NativeAddress;
        self.symbols.insert(symbol, address);
        let id_out = hudhudscript_mir::FunctionId(self.counter);
        self.counter += 1;
        Ok(NativeFunction { id: id_out, address, code_size: 0 })
    }

    fn lookup(&self, symbol: &str) -> Option<NativeAddress> {
        self.symbols.get(symbol).copied()
    }

    fn invalidate(&mut self, symbol: &str) {
        self.symbols.remove(symbol);
    }

    fn shutdown(&mut self) {
        self.symbols.clear();
    }
}

// OptGoal import'u bir sonraki şeritte kullanılacak (opt hedefi bayrakları)
#[allow(unused)]
fn _opt_goal_marker(_g: OptGoal) {}

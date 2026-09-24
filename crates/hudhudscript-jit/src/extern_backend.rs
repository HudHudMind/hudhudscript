//! gccjit/LLVM JIT modları için NativeBackend adaptörü: adres üreten
//! harici derleyici fonksiyonunu mevcut JitRuntime boru hattına bağlar.

use hudhudscript_codegen::{
    BackendCapabilities, BackendError, CodegenContext, CompiledFunction, CompiledModule,
    JitConfig, JitEngine, NativeBackend,
};
use hudhudscript_mir::{MirFunction, MirModule};
use hudhudscript_target::TargetSpec;

pub type ExternCompile =
    fn(&MirModule, &CodegenContext<'_>) -> Result<Vec<(String, usize)>, BackendError>;

pub struct ExternJitBackend {
    name: &'static str,
    compile: ExternCompile,
}

impl ExternJitBackend {
    pub fn new(name: &'static str, compile: ExternCompile) -> Self {
        Self { name, compile }
    }
}

impl NativeBackend for ExternJitBackend {
    fn name(&self) -> &'static str {
        self.name
    }

    fn capabilities(&self) -> BackendCapabilities {
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
        target.triple == "native" || target.triple == TargetSpec::host().triple
    }

    fn compile_function(
        &mut self,
        _func: &MirFunction,
        _ctx: &CodegenContext<'_>,
    ) -> Result<CompiledFunction, BackendError> {
        Err(BackendError::new("UNSUPPORTED", "extern JIT backends compile whole modules"))
    }

    fn compile_module(
        &mut self,
        module: &MirModule,
        ctx: &CodegenContext<'_>,
    ) -> Result<CompiledModule, BackendError> {
        let funcs = (self.compile)(module, ctx)?;
        Ok(CompiledModule {
            functions: funcs
                .into_iter()
                .map(|(symbol, address)| CompiledFunction { symbol, address: Some(address), code_size: 0 })
                .collect(),
            abi_version: ctx.abi_version,
            backend_name: self.name.to_string(),
            target_triple: ctx.target.triple.clone(),
        })
    }

    fn emit_object(
        &mut self,
        _module: &MirModule,
        _output: &std::path::Path,
        _ctx: &CodegenContext<'_>,
    ) -> Result<(), BackendError> {
        Err(BackendError::new(
            "UNSUPPORTED",
            "object emission lives in hudhudscript-aot (--backend … build)",
        ))
    }

    fn create_jit(&self, _config: &JitConfig) -> Result<Box<dyn JitEngine>, BackendError> {
        Err(BackendError::new("UNSUPPORTED", "eager module JIT — tiering arrives with M8"))
    }
}

//! JIT runtime: orchestrates the compile-and-run pipeline.

use hudhudscript_codegen::backend::{CodegenContext, NativeBackend, OptGoal, OptLevel};
use hudhudscript_mir::lower_module_typed;
use hudhudscript_native_abi::JitExit;
use hudhudscript_parser::parse;
use hudhudscript_target::TargetSpec;
use hudhudscript_types::lower_module_with_init;

/// Result of a JIT run.
#[derive(Debug)]
pub struct JitRunResult {
    pub exit_status: i32,
    pub return_value: i64,
    pub functions_compiled: usize,
}

/// Eager JIT runtime (v1: compile everything, run main()).
pub struct JitRuntime {
    backend: Box<dyn NativeBackend>,
}

impl JitRuntime {
    /// Varsayılan backend seçimi (auto → derlemeye giren tek backend).
    pub fn new() -> Result<Self, String> {
        Self::with_backend("auto")
    }

    /// Adla backend seçer. Bilinmeyen ad → kullanılabilirların listelendiği hata.
    pub fn with_backend(name: &str) -> Result<Self, String> {
        match name {
            "auto" | "cranelift" => {
                #[cfg(feature = "cranelift")]
                {
                    Ok(Self {
                        backend: Box::new(hudhudscript_codegen_cranelift::CraneliftBackend::new()),
                    })
                }
                #[cfg(not(feature = "cranelift"))]
                {
                    Err("no JIT backend compiled in (rebuild with --features cranelift)".to_string())
                }
            }
            #[cfg(feature = "gccjit")]
            "gccjit" => Ok(Self {
                backend: Box::new(crate::extern_backend::ExternJitBackend::new(
                    "gccjit",
                    hudhudscript_codegen_gccjit::compile_jit,
                )),
            }),
            #[cfg(not(feature = "gccjit"))]
            "gccjit" => Err("gccjit backend requires --features gccjit (libgccjit link gerektirir; konak ikili -rdynamic ile derlenmeli)".to_string()),
            #[cfg(feature = "llvm")]
            "llvm" => Ok(Self {
                backend: Box::new(crate::extern_backend::ExternJitBackend::new(
                    "llvm",
                    hudhudscript_codegen_llvm::compile_jit,
                )),
            }),
            #[cfg(not(feature = "llvm"))]
            "llvm" => Err("llvm backend requires --features llvm (LLVM 14 link gerektirir; konak ikili -rdynamic ile derlenmeli)".to_string()),
            other => Err(format!(
                "unknown backend `{other}` — available: auto, cranelift, gccjit, llvm"
            )),
        }
    }

    /// Parse → HIR → MIR → compile → execute.
    /// Scripting language semantics: top-level code runs directly (wrapped
    /// in a synthetic _hudhud_init function). No main() requirement.
    pub fn run(&mut self, source: &str) -> Result<JitRunResult, String> {
        let ast = parse(source).map_err(|e| format!("parse: {e}"))?;
        crate::precheck::quick_precheck(&ast).map_err(|e| format!("precheck: {e}"))?;
        let mut hir_module = lower_module_with_init(&ast).map_err(|e| format!("HIR: {e}"))?;

        if hir_module.functions.is_empty() {
            return Err("nothing to execute".to_string());
        }

        hudhudscript_mir::specialize_module(&mut hir_module);

        // Param tipleri kullanımdan çıkarılır (dizi/obje handle'ları Ref olur;
        // ABI'de hepsi i64 geçer — tip yalnızca lane içi dispatch içindir)
        let all_ptys = hudhudscript_mir::param_infer::infer_module_param_types(&hir_module);

        let mut mir_module = lower_module_typed(&hir_module, &all_ptys)
            .map_err(|e| format!("MIR lowering: {e}"))?;
        // MIR optimizer: §18'e saygılı const-fold (taşma/bölme asla katlanmaz)
        for f in mir_module.functions.iter_mut() {
            let (opt, _n) = hudhudscript_mir_opt::optimize(f);
            *f = opt;
        }

        if std::env::var("HUDHUD_JIT_TRACE").is_ok() {
            eprintln!("[jit:trace] Successfully lowered {} functions to native MIR", mir_module.functions.len());
        }

        let target = TargetSpec::host();
        let ctx = CodegenContext {
            target: &target,
            opt: OptLevel::O2,
            opt_goal: OptGoal::Speed,
            debug_info: false,
            abi_version: 1,
        };

        let compiled = self.backend
            .compile_module(&mir_module, &ctx)
            .map_err(|e| format!("compile: {e}"))?;

        let n = compiled.functions.len();

        let init_addr = compiled.functions.iter()
            .find(|f| f.symbol == "hudhud__hudhud_init")
            .and_then(|f| f.address);
        let main_addr = compiled.functions.iter()
            .find(|f| f.symbol == "hudhud_main")
            .and_then(|f| f.address);

        let out = match (init_addr, main_addr) {
            (Some(ia), Some(ma)) => {
                let init_out = unsafe { call_uniform(ia, &[]) };
                if init_out.status != 0 {
                    init_out
                } else {
                    unsafe { call_uniform(ma, &[]) }
                }
            }
            (Some(ia), None) => unsafe { call_uniform(ia, &[]) },
            (None, Some(ma)) => unsafe { call_uniform(ma, &[]) },
            (None, None) => return Err("no executable code found".to_string()),
        };
        let exit_status = if hudhudscript_native_abi::has_active_exception() {
            hudhudscript_native_abi::JIT_EXIT_UNCAUGHT_EXCEPTION
        } else {
            out.status
        };
        Ok(JitRunResult {
            exit_status,
            return_value: out.value,
            functions_compiled: n,
        })
    }
}

/// Uniform native entry: (argc, args*, JitExit*)
type NativeEntry = unsafe extern "C" fn(u32, *const i64, *mut JitExit);

unsafe fn call_uniform(addr: usize, args: &[i64]) -> JitExit {
    let f: NativeEntry = std::mem::transmute(addr);
    let mut out = JitExit::returned(0);
    f(args.len() as u32, args.as_ptr(), &mut out);
    out
}

impl Default for JitRuntime {
    fn default() -> Self {
        Self::new().expect("JIT runtime creation should not fail with cranelift feature")
    }
}


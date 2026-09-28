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
    /// Göreli importların çözümleme tabanı (M4) — CLI dosya dizinini verir.
    base_dir: Option<std::path::PathBuf>,
    opt_level: OptLevel,
    opt_goal: OptGoal,
    mir_opt_rounds: usize,
    policy: String,
    hot_threshold: usize,
    loop_threshold: usize,
    cache_enabled: bool,
    code_cache_mb: usize,
    verify_with_vm: bool,
}

impl JitRuntime {
    /// Varsayılan backend seçimi (auto → derlemeye giren tek backend).
    pub fn new() -> Result<Self, String> {
        Self::with_backend("auto")
    }

    /// Adla backend seçer. Bilinmeyen ad → kullanılabilirların listelendiği hata.
    pub fn with_backend(name: &str) -> Result<Self, String> {
        let backend: Box<dyn NativeBackend> = match name {
            "auto" | "cranelift" => {
                #[cfg(feature = "cranelift")]
                {
                    Box::new(hudhudscript_codegen_cranelift::CraneliftBackend::new())
                }
                #[cfg(not(feature = "cranelift"))]
                {
                    return Err("no JIT backend compiled in (rebuild with --features cranelift)".to_string());
                }
            }
            #[cfg(feature = "gccjit")]
            "gccjit" => Box::new(crate::extern_backend::ExternJitBackend::new(
                "gccjit",
                hudhudscript_codegen_gccjit::compile_jit,
            )),
            #[cfg(not(feature = "gccjit"))]
            "gccjit" => return Err("gccjit backend requires --features gccjit (libgccjit link gerektirir; konak ikili -rdynamic ile derlenmeli)".to_string()),
            #[cfg(feature = "llvm")]
            "llvm" => Box::new(crate::extern_backend::ExternJitBackend::new(
                "llvm",
                hudhudscript_codegen_llvm::compile_jit,
            )),
            #[cfg(not(feature = "llvm"))]
            "llvm" => return Err("llvm backend requires --features llvm (LLVM 14 link gerektirir; konak ikili -rdynamic ile derlenmeli)".to_string()),
            other => return Err(format!(
                "unknown backend `{other}` — available: auto, cranelift, gccjit, llvm"
            )),
        };
        Ok(Self {
            backend,
            base_dir: None,
            opt_level: OptLevel::O2,
            opt_goal: OptGoal::Speed,
            mir_opt_rounds: 4,
            policy: "eager".to_string(),
            hot_threshold: 1000,
            loop_threshold: 10000,
            cache_enabled: true,
            code_cache_mb: 64,
            verify_with_vm: false,
        })
    }

    /// Import tabanı (M4): göreli `import ... from "./x.hud"` çözümleri için.
    pub fn set_base_dir(&mut self, dir: impl Into<std::path::PathBuf>) {
        self.base_dir = Some(dir.into());
    }

    /// Sets the backend optimization level (O0, O1, O2, O3).
    pub fn set_opt_level(&mut self, opt: OptLevel) {
        self.opt_level = opt;
    }

    /// Sets the backend optimization level from string ("o0", "o1", "o2", "o3", "none", "speed").
    pub fn set_opt_level_str(&mut self, s: &str) -> bool {
        let opt = match s.to_ascii_lowercase().as_str() {
            "0" | "o0" | "none" => OptLevel::O0,
            "1" | "o1" | "less" => OptLevel::O1,
            "2" | "o2" | "default" | "speed" => OptLevel::O2,
            "3" | "o3" | "aggressive" => OptLevel::O3,
            _ => return false,
        };
        self.opt_level = opt;
        true
    }

    /// Sets the optimization goal (Speed, Size, SizeMin).
    pub fn set_opt_goal(&mut self, goal: OptGoal) {
        self.opt_goal = goal;
    }

    /// Sets the optimization goal from string ("speed", "size", "sizemin").
    pub fn set_opt_goal_str(&mut self, s: &str) -> bool {
        let goal = match s.to_ascii_lowercase().as_str() {
            "speed" => OptGoal::Speed,
            "size" => OptGoal::Size,
            "sizemin" | "size_min" => OptGoal::SizeMin,
            _ => return false,
        };
        self.opt_goal = goal;
        true
    }

    /// Sets maximum fixed-point rounds for the MIR optimizer.
    pub fn set_mir_opt_rounds(&mut self, rounds: usize) {
        self.mir_opt_rounds = rounds;
    }

    /// Sets the JIT compilation policy ("hot", "lazy", "eager").
    pub fn set_policy(&mut self, policy: impl Into<String>) {
        self.policy = policy.into();
    }

    /// Sets the hot threshold invocation count for JIT compilation.
    pub fn set_hot_threshold(&mut self, threshold: usize) {
        self.hot_threshold = threshold;
    }

    /// Sets the loop threshold iteration count for on-stack replacement.
    pub fn set_loop_threshold(&mut self, threshold: usize) {
        self.loop_threshold = threshold;
    }

    /// Enables or disables JIT code caching.
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    /// Sets JIT code cache size limit in megabytes.
    pub fn set_code_cache_mb(&mut self, mb: usize) {
        self.code_cache_mb = mb;
    }

    /// Sets whether differential replay against the VM is enabled.
    pub fn set_verify_with_vm(&mut self, verify: bool) {
        self.verify_with_vm = verify;
    }

    /// Parse → HIR → MIR → compile → execute.
    /// Scripting language semantics: top-level code runs directly (wrapped
    /// in a synthetic _hudhud_init function). No main() requirement.
    pub fn run(&mut self, source: &str) -> Result<JitRunResult, String> {
        let ast = parse(source).map_err(|e| format!("parse: {e}"))?;
        // M4: yerel importlar AST-düzeyinde birleştirilir; çözümlenemeyen
        // import yerinde kalır → precheck reddi → dürüst VM-fallback.
        let ast = crate::module_linker::link_imports(&ast, self.base_dir.as_deref())?;
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
            let (opt, _n) = hudhudscript_mir_opt::optimize_with_rounds(f, self.mir_opt_rounds);
            *f = opt;
        }

        if std::env::var("HUDHUD_JIT_TRACE").is_ok() {
            eprintln!("[jit:trace] Successfully lowered {} functions to native MIR", mir_module.functions.len());
        }

        let target = TargetSpec::host();
        let ctx = CodegenContext {
            target: &target,
            opt: self.opt_level,
            opt_goal: self.opt_goal,
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

        // Kütüphane Modülü (M2, v0.9.35): init/main yoksa ölümcül hata yerine
        // derlenmiş fonksiyon kütüphanesi say — hiçbiri koşmaz, exit 0 ile
        // döner (hudunit benzeri test koşucuları sembolleri call_uniform ile
        // çağırır). Yalnız fonksiyonlar da boşsa "nothing to execute" kalır
        // (yukarıdaki boş-modül denetimi).
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
            (None, None) => {
                return Ok(JitRunResult {
                    exit_status: 0,
                    return_value: 0,
                    functions_compiled: n,
                })
            }
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


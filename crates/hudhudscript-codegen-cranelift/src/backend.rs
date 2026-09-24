//! `CraneliftBackend` — NativeBackend implementation over cranelift-jit.

use std::collections::HashMap;
use std::path::Path;

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

/// Host-ISA Cranelift backend (first-light capabilities: JIT only).
pub struct CraneliftBackend {
    symbols: HashMap<String, NativeAddress>,
}

impl CraneliftBackend {
    pub fn new() -> Self {
        CraneliftBackend { symbols: HashMap::new() }
    }

    fn make_jit_module(opt_level: &str) -> Result<JITModule, BackendError> {
        let mut flags = settings::builder();
        flags
            .set("opt_level", opt_level)
            .map_err(|e| BackendError::new("ISA_FAIL", format!("opt_level: {e}")))?;
        let isa_builder = cranelift_native::builder()
            .map_err(|e| BackendError::new("ISA_FAIL", format!("native isa: {e}")))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flags))
            .map_err(|e| BackendError::new("ISA_FAIL", format!("isa finish: {e}")))?;
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        // External semboller (native-abi crate'inden)
        let symbols: &[(&str, *const u8)] = &[
            ("hudhud_print_int", hudhudscript_native_abi::hudhud_print_int as *const u8),
            ("hudhud_print_float", hudhudscript_native_abi::hudhud_print_float as *const u8),
            ("hudhud_print_str", hudhudscript_native_abi::hudhud_print_str as *const u8),
            ("fmod", libm::fmod as *const u8),
            ("hudhud_date_millis", hudhudscript_native_abi::hudhud_date_millis as *const u8),
            ("hudhud_math_sin", hudhudscript_native_abi::hudhud_math_sin as *const u8),
            ("hudhud_math_sqrt", hudhudscript_native_abi::hudhud_math_sqrt as *const u8),
            ("hudhud_math_cos", hudhudscript_native_abi::hudhud_math_cos as *const u8),
            ("hudhud_math_floor", hudhudscript_native_abi::hudhud_math_floor as *const u8),
            ("hudhud_math_abs", hudhudscript_native_abi::hudhud_math_abs as *const u8),
            ("hudhud_math_pow", hudhudscript_native_abi::hudhud_math_pow as *const u8),
            ("hudhud_math_min", hudhudscript_native_abi::hudhud_math_min as *const u8),
            ("hudhud_math_max", hudhudscript_native_abi::hudhud_math_max as *const u8),
            ("hudhud_globals", hudhudscript_native_abi::hudhud_globals as *const u8),
            ("hudhud_global_get", hudhudscript_native_abi::hudhud_global_get as *const u8),
            ("hudhud_global_set", hudhudscript_native_abi::hudhud_global_set as *const u8),
            ("hudhud_string_concat", hudhudscript_native_abi::hudhud_string_concat as *const u8),
            ("hudhud_string_len", hudhudscript_native_abi::hudhud_string_len as *const u8),
            ("hudhud_string_eq", hudhudscript_native_abi::hudhud_string_eq as *const u8),
            ("hudhud_int_to_string", hudhudscript_native_abi::hudhud_int_to_string as *const u8),
            ("hudhud_float_to_string", hudhudscript_native_abi::hudhud_float_to_string as *const u8),
            ("hudhud_string_char_at", hudhudscript_native_abi::hudhud_string_char_at as *const u8),
            ("hudhud_string_substring", hudhudscript_native_abi::hudhud_string_substring as *const u8),
            ("hudhud_string_to_int", hudhudscript_native_abi::hudhud_string_to_int as *const u8),
            ("hudhud_string_split", hudhudscript_native_abi::hudhud_string_split as *const u8),
            ("hudhud_string_index_of", hudhudscript_native_abi::hudhud_string_index_of as *const u8),
            ("hudhud_array_new", hudhudscript_native_abi::hudhud_array_new as *const u8),
            ("hudhud_array_filled", hudhudscript_native_abi::hudhud_array_filled as *const u8),
            ("hudhud_array_fill", hudhudscript_native_abi::hudhud_array_fill as *const u8),
            ("hudhud_array_push", hudhudscript_native_abi::hudhud_array_push as *const u8),
            ("hudhud_array_get", hudhudscript_native_abi::hudhud_array_get as *const u8),
            ("hudhud_array_set", hudhudscript_native_abi::hudhud_array_set as *const u8),
            ("hudhud_array_len", hudhudscript_native_abi::hudhud_array_len as *const u8),
            ("hudhud_array_pop", hudhudscript_native_abi::hudhud_array_pop as *const u8),
            ("hudhud_array_join", hudhudscript_native_abi::hudhud_array_join as *const u8),
            ("hudhud_object_new", hudhudscript_native_abi::hudhud_object_new as *const u8),
            ("hudhud_object_set", hudhudscript_native_abi::hudhud_object_set as *const u8),
            ("hudhud_object_get", hudhudscript_native_abi::hudhud_object_get as *const u8),
            ("hudhud_object_has", hudhudscript_native_abi::hudhud_object_has as *const u8),
            ("hudhud_object_len", hudhudscript_native_abi::hudhud_object_len as *const u8),
            ("hudhud_typeof", hudhudscript_native_abi::hudhud_typeof as *const u8),
            ("hudhud_register_string", hudhudscript_native_abi::hudhud_register_string as *const u8),
            ("hudhud_bigint_add_trusted", hudhudscript_native_abi::hudhud_bigint_add_trusted as *const u8),
            ("hudhud_bigint_sub_trusted", hudhudscript_native_abi::hudhud_bigint_sub_trusted as *const u8),
            ("hudhud_bigint_mul_trusted", hudhudscript_native_abi::hudhud_bigint_mul_trusted as *const u8),
            ("hudhud_string_cmp", hudhudscript_native_abi::hudhud_string_cmp as *const u8),
            ("hudhud_string_append", hudhudscript_native_abi::hudhud_string_append as *const u8),
            ("hudhud_throw", hudhudscript_native_abi::hudhud_throw as *const u8),
            ("hudhud_has_exception", hudhudscript_native_abi::hudhud_has_exception as *const u8),
            ("hudhud_catch", hudhudscript_native_abi::hudhud_catch as *const u8),
            ("hudhud_num_add", hudhudscript_native_abi::hudhud_num_add as *const u8),
            ("hudhud_num_sub", hudhudscript_native_abi::hudhud_num_sub as *const u8),
            ("hudhud_num_mul", hudhudscript_native_abi::hudhud_num_mul as *const u8),
            ("hudhud_num_div", hudhudscript_native_abi::hudhud_num_div as *const u8),
            ("hudhud_num_rem", hudhudscript_native_abi::hudhud_num_rem as *const u8),
            ("hudhud_num_cmp", hudhudscript_native_abi::hudhud_num_cmp as *const u8),
            ("hudhud_string_trim", hudhudscript_native_abi::hudhud_string_trim as *const u8),
            ("hudhud_string_starts_with", hudhudscript_native_abi::hudhud_string_starts_with as *const u8),
            ("hudhud_string_ends_with", hudhudscript_native_abi::hudhud_string_ends_with as *const u8),
            ("hudhud_string_contains", hudhudscript_native_abi::hudhud_string_contains as *const u8),
            ("hudhud_string_replace", hudhudscript_native_abi::hudhud_string_replace as *const u8),
            ("hudhud_string_to_lower", hudhudscript_native_abi::hudhud_string_to_lower as *const u8),
            ("hudhud_string_to_upper", hudhudscript_native_abi::hudhud_string_to_upper as *const u8),
            ("hudhud_string_char_code_at", hudhudscript_native_abi::hudhud_string_char_code_at as *const u8),
            ("hudhud_array_slice", hudhudscript_native_abi::hudhud_array_slice as *const u8),
            ("hudhud_array_reverse", hudhudscript_native_abi::hudhud_array_reverse as *const u8),
            ("hudhud_array_concat", hudhudscript_native_abi::hudhud_array_concat as *const u8),
            ("hudhud_array_index_of", hudhudscript_native_abi::hudhud_array_index_of as *const u8),
            ("hudhud_array_includes", hudhudscript_native_abi::hudhud_array_includes as *const u8),
            ("hudhud_array_shift", hudhudscript_native_abi::hudhud_array_shift as *const u8),
            ("hudhud_array_unshift", hudhudscript_native_abi::hudhud_array_unshift as *const u8),
            ("hudhud_array_sort", hudhudscript_native_abi::hudhud_array_sort as *const u8),
            ("hudhud_date_now", hudhudscript_native_abi::hudhud_date_now as *const u8),
            ("hudhud_time_nanos", hudhudscript_native_abi::hudhud_time_nanos as *const u8),
            ("hudhud_time_micros", hudhudscript_native_abi::hudhud_time_micros as *const u8),
            ("hudhud_sleep_millis", hudhudscript_native_abi::hudhud_sleep_millis as *const u8),
            ("hudhud_sleep_micros", hudhudscript_native_abi::hudhud_sleep_micros as *const u8),
            ("hudhud_date_year", hudhudscript_native_abi::hudhud_date_year as *const u8),
            ("hudhud_date_month", hudhudscript_native_abi::hudhud_date_month as *const u8),
            ("hudhud_date_day", hudhudscript_native_abi::hudhud_date_day as *const u8),
            ("hudhud_date_hour", hudhudscript_native_abi::hudhud_date_hour as *const u8),
            ("hudhud_date_minute", hudhudscript_native_abi::hudhud_date_minute as *const u8),
            ("hudhud_date_second", hudhudscript_native_abi::hudhud_date_second as *const u8),
            ("hudhud_date_iso", hudhudscript_native_abi::hudhud_date_iso as *const u8),
            // data sembolü: satır-içi global erişimi (emit_global_load/store)
            ("GLOBAL_SLOTS", unsafe { hudhudscript_native_abi::object::GLOBAL_SLOTS.as_ptr() } as *const u8),
        ];
        for &(name, ptr) in symbols {
            builder.symbol(name, ptr);
        }
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
        _ctx: &CodegenContext<'_>,
    ) -> Result<CompiledFunction, BackendError> {
        let mut module = Self::make_jit_module("speed")?;
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
        let mut jit = Self::make_jit_module("speed")?;
        let ptr = jit.isa().pointer_type();

        // 1) TÜM fonksiyonları ÖNCE declare et (CallStatic ileri referans)
        let mut func_ids: Vec<cranelift_module::FuncId> = Vec::new();
        for f in &module.functions {
            let mut sig = jit.make_signature();
            sig.params.push(cranelift::prelude::AbiParam::new(
                cranelift::prelude::types::I32,
            ));
            sig.params.push(cranelift::prelude::AbiParam::new(ptr));
            sig.params.push(cranelift::prelude::AbiParam::new(ptr));
            let symbol = format!("hudhud_{}", f.name);
            let id = jit
                .declare_function(&symbol, cranelift_module::Linkage::Export, &sig)
                .map_err(|e| {
                    BackendError::new("DECLARE_FAIL", format!("declare {symbol}: {e}"))
                        .in_function(f.name.to_string())
                })?;
            func_ids.push(id);
        }

        // 2) Her fonksiyonu çevir (CallStatic → declare edilmiş FuncId'ye call)
        let mut results = Vec::new();
        for f in module.functions.iter() {
            let mut fc = cranelift::prelude::FunctionBuilderContext::new();
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

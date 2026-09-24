//! Trait-contract test: proves the `NativeBackend`/`JitEngine`
//! abstraction is implementable and drivable WITHOUT any real codegen
//! backend. A recording backend (test-only) implements the traits and
//! the test asserts dispatch through trait objects, capability gating
//! and error propagation — the contracts M2 (Cranelift) will plug into.

use std::path::Path;

use hudhudscript_codegen::backend::{
    BackendError, CodegenContext, CompiledFunction, CompiledModule, NativeBackend,
    OptGoal, OptLevel,
};
use hudhudscript_codegen::jit::JitConfig;
use hudhudscript_codegen::capabilities::{BackendCapabilities, CapabilityNeed};
use hudhudscript_codegen::jit::{JitEngine, NativeFunction};
use hudhudscript_mir::builder::MirFunctionBuilder;
use hudhudscript_mir::{MirFunction, MirModule, MirType};
use hudhudscript_target::TargetSpec;

/// Kayıt tutan test backend'i: gerçek kod üretmez, çağrıları sayar.
struct RecordingBackend {
    compile_calls: usize,
    jit_calls: usize,
    fail_compile: bool,
}

impl RecordingBackend {
    fn new() -> Self {
        RecordingBackend { compile_calls: 0, jit_calls: 0, fail_compile: false }
    }
}

impl NativeBackend for RecordingBackend {
    fn name(&self) -> &'static str {
        "recording"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::JIT_ONLY
    }

    fn supports_target(&self, target: &TargetSpec) -> bool {
        // Test sözleşmesi: yalnız 64-bit süreçler.
        target.pointer_width == 8
    }

    fn compile_function(
        &mut self,
        _func: &MirFunction,
        _ctx: &CodegenContext<'_>,
    ) -> Result<CompiledFunction, BackendError> {
        self.compile_calls += 1;
        if self.fail_compile {
            return Err(BackendError::new("TEST_FAIL", "injected failure")
                .in_function("victim"));
        }
        Ok(CompiledFunction {
            symbol: "f".into(),
            address: Some(0x1000),
            code_size: 16,
        })
    }

    fn compile_module(
        &mut self,
        _module: &MirModule,
        ctx: &CodegenContext<'_>,
    ) -> Result<CompiledModule, BackendError> {
        Ok(CompiledModule {
            functions: vec![],
            abi_version: ctx.abi_version,
            backend_name: self.name().into(),
            target_triple: ctx.target.triple.clone(),
        })
    }

    fn emit_object(
        &mut self,
        _module: &MirModule,
        _output: &Path,
        _ctx: &CodegenContext<'_>,
    ) -> Result<(), BackendError> {
        Err(BackendError::new("UNSUPPORTED", "recording backend has no AOT lane"))
    }

    fn create_jit(&self, _config: &JitConfig) -> Result<Box<dyn JitEngine>, BackendError> {
        Ok(Box::new(RecordingJit { compiles: 0 }))
    }
}

struct RecordingJit {
    compiles: usize,
}

impl JitEngine for RecordingJit {
    fn compile(&mut self, _function: &MirFunction) -> Result<NativeFunction, BackendError> {
        self.compiles += 1;
        Ok(NativeFunction {
            id: hudhudscript_mir::FunctionId(0),
            address: 0x2000,
            code_size: 32,
        })
    }

    fn lookup(&self, _symbol: &str) -> Option<usize> {
        Some(0x2000)
    }

    fn invalidate(&mut self, _symbol: &str) {}

    fn shutdown(&mut self) {}
}

#[test]
fn dispatch_through_trait_object() {
    let mut backend: Box<dyn NativeBackend> = Box::new(RecordingBackend::new());
    assert_eq!(backend.name(), "recording");
    assert!(backend.capabilities().supports(CapabilityNeed::Jit));
    assert!(!backend.capabilities().supports(CapabilityNeed::Aot));

    let target = TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let module = MirModule::default();
    let compiled = backend.compile_module(&module, &ctx).expect("module compiles");
    assert_eq!(compiled.backend_name, "recording");
    assert_eq!(compiled.abi_version, 1);
}

#[test]
fn target_gating_uses_pointer_width() {
    let backend = RecordingBackend::new();
    let t64 = hudhudscript_target::parse_triple("x86_64-unknown-linux-gnu").unwrap();
    let t32 = hudhudscript_target::parse_triple("armv7-unknown-linux-gnueabihf").unwrap();
    assert!(backend.supports_target(&t64));
    assert!(!backend.supports_target(&t32));
}

#[test]
fn errors_carry_function_context() {
    let mut backend = RecordingBackend::new();
    backend.fail_compile = true;
    let target = TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O0,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let err = backend.compile_function(&empty_mir_fn(), &ctx).unwrap_err();
    assert_eq!(err.function.as_deref(), Some("victim"));
    assert!(err.to_string().contains("TEST_FAIL"));
    assert!(err.to_string().contains("victim"));
}

#[test]
fn aot_lane_is_refused_when_capability_missing() {
    let mut backend = RecordingBackend::new();
    let target = TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O1,
        opt_goal: OptGoal::Size,
        debug_info: false,
        abi_version: 1,
    };
    let err = backend
        .emit_object(&MirModule::default(), Path::new("/tmp/x.o"), &ctx)
        .unwrap_err();
    assert_eq!(err.code, "UNSUPPORTED");
}

#[test]
fn jit_engine_object_round_trip() {
    let backend = RecordingBackend::new();
    let cfg = JitConfig {
        target: TargetSpec::host(),
        opt: OptLevel::O2,
        code_cache_mb: 64,
    };
    let mut jit = backend.create_jit(&cfg).expect("jit engine");
    let mir = example_add_mir();
    let nf = jit.compile(&mir).expect("compile");
    assert_eq!(nf.address, 0x2000);
    assert_eq!(jit.lookup("add"), Some(0x2000));
    jit.shutdown();
}

fn example_add_mir() -> MirFunction {
    let mut b = MirFunctionBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, 2);
    let y = b.const_i64(e, 3);
    let s = b.bin(e, hudhudscript_mir::builder::BinOp::Add, MirType::I64, x, y);
    b.ret(e, s);
    b.finish()
}

fn empty_mir_fn() -> MirFunction {
    let mut b = MirFunctionBuilder::new("victim", vec![], MirType::I64);
    let e = b.entry();
    let z = b.const_i64(e, 0);
    b.ret(e, z);
    b.finish()
}

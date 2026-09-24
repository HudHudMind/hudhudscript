//! `NativeBackend` trait and shared codegen types
//! (JIT_AOT_ARCHITECTURE.md §7).

use std::fmt;
use std::path::Path;

use hudhudscript_mir::{MirFunction, MirModule};
use hudhudscript_target::TargetSpec;

use crate::capabilities::BackendCapabilities;
use crate::jit::{JitConfig, JitEngine};

/// Optimization level — a backend-independent concept (§18 of the CLI
/// design: backend, opt level and mode are three separate axes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

/// What the optimization is trading for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptGoal {
    Speed,
    Size,
    SizeMin,
}

/// Shared per-compilation context handed to every backend call.
pub struct CodegenContext<'a> {
    pub target: &'a TargetSpec,
    pub opt: OptLevel,
    pub opt_goal: OptGoal,
    pub debug_info: bool,
    /// HUDHUD_RUNTIME_ABI_VERSION (§11.2) — must ride every artifact and
    /// cache key.
    pub abi_version: u32,
}

/// A single compiled function handle.
#[derive(Debug)]
pub struct CompiledFunction {
    pub symbol: String,
    /// Address once placed in executable memory (JIT) or `None` until
    /// the object is linked (AOT).
    pub address: Option<usize>,
    pub code_size: usize,
}

/// A compiled module: functions plus the metadata block embedded in
/// every artifact (ABI version, backend, target triple).
#[derive(Debug)]
pub struct CompiledModule {
    pub functions: Vec<CompiledFunction>,
    pub abi_version: u32,
    pub backend_name: String,
    pub target_triple: String,
}

/// Uniform backend error. Backends never panic across the FFI/test
/// boundary — they return this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    pub code: &'static str,
    pub message: String,
    pub function: Option<String>,
}

impl BackendError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        BackendError { code, message: message.into(), function: None }
    }

    pub fn in_function(mut self, function: impl Into<String>) -> Self {
        self.function = Some(function.into());
        self
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.function {
            Some(func) => write!(f, "[{}] {} (in `{func}`)", self.code, self.message),
            None => write!(f, "[{}] {}", self.code, self.message),
        }
    }
}

impl std::error::Error for BackendError {}

/// The single code-generation interface every backend implements
/// (§7). Object-first rule: AOT goes through `emit_object`; producing a
/// final executable from inside a backend is forbidden by design.
pub trait NativeBackend {
    fn name(&self) -> &'static str;

    fn capabilities(&self) -> BackendCapabilities;

    fn supports_target(&self, target: &TargetSpec) -> bool;

    fn compile_function(
        &mut self,
        func: &MirFunction,
        ctx: &CodegenContext<'_>,
    ) -> Result<CompiledFunction, BackendError>;

    fn compile_module(
        &mut self,
        module: &MirModule,
        ctx: &CodegenContext<'_>,
    ) -> Result<CompiledModule, BackendError>;

    fn emit_object(
        &mut self,
        module: &MirModule,
        output: &Path,
        ctx: &CodegenContext<'_>,
    ) -> Result<(), BackendError>;

    fn create_jit(&self, config: &JitConfig) -> Result<Box<dyn JitEngine>, BackendError>;
}

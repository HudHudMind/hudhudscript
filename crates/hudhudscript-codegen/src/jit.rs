//! `JitEngine` trait — executable-memory side of a backend
//! (JIT_AOT_ARCHITECTURE.md §7).

use hudhudscript_mir::{FunctionId, MirFunction};
use hudhudscript_target::TargetSpec;

use crate::backend::{BackendError, OptLevel};

/// JitEngine configuration carried from `[runtime.jit]` + CLI flags.
#[derive(Debug, Clone)]
pub struct JitConfig {
    pub target: TargetSpec,
    pub opt: OptLevel,
    pub code_cache_mb: u32,
}

/// Raw native code address.
pub type NativeAddress = usize;

/// A compiled callable: the address plus the identity it was built from.
#[derive(Debug)]
pub struct NativeFunction {
    pub id: FunctionId,
    pub address: NativeAddress,
    pub code_size: usize,
}

/// Runtime JIT engine handle created by `NativeBackend::create_jit`.
pub trait JitEngine {
    fn compile(&mut self, function: &MirFunction) -> Result<NativeFunction, BackendError>;
    fn lookup(&self, symbol: &str) -> Option<NativeAddress>;
    fn invalidate(&mut self, symbol: &str);
    fn shutdown(&mut self);
}

use hudhudscript_bytecode::{SymId, Value16};
use std::sync::Arc;

/// Decode a packed `u32` into `(opcode, arg1, arg2)`.
#[inline(always)]
pub(crate) const fn decode_packed(packed: u32) -> (u8, u8, u16) {
    let opcode = (packed & 0xFF) as u8;
    let arg1 = ((packed >> 8) & 0xFF) as u8;
    let arg2 = ((packed >> 16) & 0xFFFF) as u16;
    (opcode, arg1, arg2)
}

/// A3b: widen a popped stack `Value` to `f64` for the existing
/// `NumAdd` / `NumSub` / ... arithmetic arms.  Accepts `Number(f64)` or
/// `Int(i64)` (widens `i as f64`), returning `None` for any other
/// variant.  Fast path: `Number` is unwrapped without branching to
/// `Int` since `num_as_f64` inlines to an ordered match.  Kural 7c:
/// single helper — no per-arm duplication of the widening pattern.
#[inline(always)]
#[inline(always)]
pub(crate) fn num_as_f64(v: Option<Value16>) -> Option<f64> {
    v.and_then(|val| val.as_number_fast())
}

/// A3b: widen a borrowed slot `Value` (read from `registers`) to `f64`.
/// Used by the `NumSubISlot` / `NumLtISlot` / ... fused arms where the
/// slot may hold either `Number` or `Int` (integer-valued parameters
/// passed via `LoadIntConst` end up in registers as `Int`).
#[inline(always)]
pub(crate) fn num_ref_as_f64(v: Option<&Value16>) -> Option<f64> {
    v.and_then(|val| val.as_number_fast())
}

/// A3c: slot-classification for the `Int*ISlot` dual-type dispatch.
/// The fused integer-slot arms take different code paths depending on
/// whether the slot holds `Int` (stay in `i64`) or `Number` (widen imm
/// to `f64`, push Number).  Kural 7c: both paths are strictly-typed,
/// no fallback — the VM errors when the slot is neither Int nor Number.
#[derive(Debug, Clone, Copy)]
pub(crate) enum NumericSlot {
    Int(i64),
    Num(f64),
}

#[inline(always)]
pub(crate) fn numeric_slot(v: Option<&Value16>) -> Option<NumericSlot> {
    let val = v?;
    if let Some(b) = val.as_bigint() { return None; }
    if let Some(i) = val.as_int() { return Some(NumericSlot::Int(i)); }
    if let Some(n) = val.as_number() { return Some(NumericSlot::Num(n)); }
    None
}

/// Result of `exec_end_finally` — either jump to an IP or return from
/// the current function.  Isolated from the main `StepAction` because
/// `FinallyExit` never emits `Advance` / `Break`.
pub(crate) enum EndFinallyAction {
    Jump(usize),
    Return,
}

/// Result of the shared finally-instruction dispatcher (`exec_finally_instr`).
/// Distinct from `StepAction` so the outer instruction arm only needs a
/// compact match (Gap 2 — stack-frame minimisation).
pub(crate) enum FinallyStep {
    Advance,
    Jump(usize),
    Return,
}

/// Gap 2 — try/finally VM state snapshot used across function-call
/// boundaries.  Allocated on the heap via `Box<_>` so the caller's
/// stack frame stays small (3 Vecs + Option would otherwise add ~72
/// bytes per recursive frame and overflow the 2 MB test thread stack
/// on mutual-recursion tests).
pub(crate) struct SavedFinally {
    pub(crate) try_frames: Vec<(usize, usize, usize)>,
    pub(crate) finally_frames: Vec<(usize, usize, usize)>,
    pub(crate) pending_flow: Option<PendingFlow>,
}

/// Pending abrupt-completion carried across a `finally` body.
///
/// When `Return` or `Throw` fires inside a try/catch that has an active
/// finally frame, the VM stashes the abrupt flow here and jumps to the
/// finally body.  The `FinallyExit` instruction re-issues the stashed flow
/// after the finally body completes (unless the finally body itself
/// performed an abrupt completion, which overwrites the pending slot —
/// matching the interpreter's `finally_result.is_err() → return
/// finally_result` semantics).
///
/// Boxed payload keeps the enum small — `Value` is ~176 bytes, which
/// would otherwise inflate every `call_chunk_with_captures` stack frame
/// when paired with the `saved_pending_flow` Option.
#[derive(Debug, Clone)]
pub(crate) enum PendingFlow {
    Return(Box<Value16>),
    Throw(Box<Value16>),
}

/// Result of the packed fast-dispatch path.
pub(crate) enum PackedResult {
    /// Instruction handled; advance ip by 1.
    Advance,
    /// Instruction handled; set ip to the given target.
    Jump(usize),
    /// Hit a Return instruction. `src` is the register holding the value.
    Return { src: u8 },
    /// Opcode not handled by the fast path; fall through to the full match.
    Fallthrough,
}

/// Result of a single-instruction step in `execute_instructions`.
///
/// Mirrors the subset of outer-loop flow previously expressed via raw
/// `continue;` / `break;` statements inside the big instruction match.
/// Needed so the step body can live inside a closure (for unified runtime
/// error → try/catch routing) while still conveying jump / return intents
/// back to the loop driver.
pub(crate) enum StepAction {
    /// Normal fall-through: caller should advance `ip` by 1.
    Advance,
    /// The handler already set `ip` to a new target; caller must NOT advance.
    Jumped,
    /// A `Return` instruction fired — exit the loop with `hit_return = true`.
    /// `src` names the register holding the return value; the loop driver copies
    /// it to `last_return` and, when there is no pending finally block, to the
    /// caller's destination register.
    Return { src: u8 },
    /// K10: Function call — trampoline pushes frame and continues with callee.
    /// `ip` is the instruction pointer of the Call site; used for IC cache lookup.
    Call {
        func_sym: SymId,
        function_idx: u32,
        arg_count: u8,
        first_arg: u8,
        dst: u8,
        ip: usize,
    },
    /// Fallback `Break` (no loop header available) — exit the loop with
    /// `hit_return` unchanged (false).
    Break,
    /// CROSS-1 (TCO): self-recursive tail-call — the caller must
    /// (1) unwind any scope/try/loop/finally frames pushed since the
    /// function's entry,
    /// (2) overwrite `registers[0..argc]` with the drained args
    /// stashed in `self.tco_args` (and null out any remaining
    /// locals beyond them),
    /// (3) clear the operand stack, and
    /// (4) reset `ip = 0` and continue the execute_instructions loop.
    /// Zero stack-frame — no push/pop of the call stack.
    ///
    /// The Vec<Value> of drained args is stashed in `VM::tco_args`
    /// (not the enum) to keep `Result<StepAction, CompileError>`
    /// small on the hot dispatch path.
    TailCall,
}

/// Result of one `step_generator_iter` call — signals whether the caller
/// should continue the loop, exit it, or surface an error (#P2-7).
#[derive(Debug)]
pub(crate) enum GenStep {
    /// A value was produced and bound to the iterator variable.
    Advanced,
    /// Generator exhausted — caller should pop and jump to exit target.
    Exhausted,
}

/// Maximum number of values allowed on the VM stack before raising a stack
/// overflow error. This prevents runaway recursion or infinite loops from
/// consuming unbounded memory.
/// Configurable via HUDHUD_VM_STACK_LIMIT env var (#793).
/// Cached after first call — env::var is far too expensive to call on every
/// instruction (discovered via perf: 14% of fib(30) runtime was in getenv).
#[inline(always)]
pub(crate) fn max_stack_size() -> usize {
    use std::sync::OnceLock;
    static CACHED: OnceLock<usize> = OnceLock::new();
    *CACHED.get_or_init(|| {
        std::env::var("HUDHUD_VM_STACK_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(hudhudscript_errors::constants::MAX_STACK_SIZE)
    })
}

/// Sentinel value stored in the packed-instruction cache slots that
/// couldn't be packed into a `u32`. Chosen as `u32::MAX` because real
/// packed encodings always have opcode < 128 (currently 76 opcodes
/// defined), so the top byte of a real encoding is never `0xFF`.

pub(crate) const STACK_SAFETY_MARGIN: usize = 16 * 1024; // 16KB

#[cfg(target_os = "linux")]
pub(crate) fn remaining_stack_bytes() -> usize {
    let local: u8 = 0;
    let current_sp = &local as *const u8 as usize;
    thread_local! {
        static STACK_BOTTOM: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    STACK_BOTTOM.with(|bottom| {
        let b = bottom.get();
        if b != 0 {
            return if current_sp > b {
                current_sp - b
            } else {
                usize::MAX
            };
        }
        // First call: resolve stack bottom via pthread (cached thereafter).
        unsafe {
            let mut attr: libc::pthread_attr_t = std::mem::zeroed();
            if libc::pthread_getattr_np(libc::pthread_self(), &mut attr) != 0 {
                return usize::MAX;
            }
            let mut stack_addr: *mut libc::c_void = std::ptr::null_mut();
            let mut stack_size: libc::size_t = 0;
            if libc::pthread_attr_getstack(&mut attr, &mut stack_addr, &mut stack_size) != 0 {
                libc::pthread_attr_destroy(&mut attr);
                return usize::MAX;
            }
            libc::pthread_attr_destroy(&mut attr);
            let sb = stack_addr as usize;
            bottom.set(sb);
            if current_sp > sb {
                current_sp - sb
            } else {
                usize::MAX
            }
        }
    })
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn remaining_stack_bytes(default_bytes: usize) -> usize {
    let local: u8 = 0;
    let current_sp = &local as *const u8 as usize;
    thread_local! {
        static STACK_TOP: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    STACK_TOP.with(|top| {
        if top.get() == 0 {
            top.set(current_sp);
            return usize::MAX;
        }
        let stack_top = top.get();
        // Stack grows downward on x86/ARM
        if stack_top > current_sp {
            let used = stack_top - current_sp;
            let total: usize = default_bytes;
            total.saturating_sub(used)
        } else {
            usize::MAX
        }
    })
}

// Kural 7: the legacy `next_vm_id`/`VM_ID_COUNTER` pair previously minted
// promise/actor ids here.  Promise ids now come from the shared
// `hudhudscript_async::PromiseRegistry`; actor ids come from
// `hudhudscript_bytecode::actor_registry::SharedActorRegistry`.
// There is no second id generator in the VM.

// ---------------------------------------------------------------------------
// Module Registry (#928)
// ---------------------------------------------------------------------------

pub enum ExecutionState {
    /// Execution completed normally.
    Completed,
    /// Execution was suspended after the time limit.
    /// Call `execute_resume()` to continue.
    Suspended { ip: usize },
}

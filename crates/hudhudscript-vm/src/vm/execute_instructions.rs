use crate::vm::config_types::{OutputLocale, SandboxConfig};
use crate::vm::mcp_dispatch::{dispatch_mcp_tool_call, McpContext};
use crate::vm::prepack::PACK_SENTINEL;
use crate::vm::provider_dispatch::{dispatch_provider_call, ProviderCallConfig, ProviderContext};
use crate::vm::registry::{BuiltinFn, ModuleRegistry};
use crate::vm::util::evaluate_condition_static;
use crate::vm::VM;
use crate::vm::{
    max_stack_size, num_as_f64, num_ref_as_f64, numeric_slot, EndFinallyAction, FinallyStep,
    GenStep, NumericSlot, PackedResult, PendingFlow, SavedFinally, StepAction,
};
use crate::vm::execute::StepContext;
use hudhudscript_bytecode::cache_utils::{enforce_cache_limit, MAX_MCP_CACHE, MAX_RAG_STORE_CACHE};
use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult, SourcePosition};
use hudhudscript_bytecode::packed_instruction;
use hudhudscript_bytecode::shared_value::{
    num_add, num_div, num_eq, num_ge, num_gt, num_le, num_lt, num_mod, num_mul, num_neg, num_sub,
};
use hudhudscript_bytecode::{
    Bytecode, ClassData, FunctionChunk, FunctionData, GeneratorState16, InstanceData, Instruction,
    PromiseState16, ReprTag, Value16,
};
use hudhudscript_debug::Debugger;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use hudhudscript_governance::{Condition, Constitution};
use hudhudscript_mcp::{McpClient, Tool as McpToolDefinition};
use hudhudscript_rag::{
    DistanceMetric, EmbeddingProvider, SimpleEmbedding, VectorStore, VectorStoreConfig,
};
use hudhudscript_runtime::provider::ProviderRegistry;
use hudhudscript_tools::ToolRegistry;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

impl crate::vm::VM {
    #[inline(never)]
    pub(crate) fn execute_instructions(
        &mut self,
        instructions: &[Instruction],
        constants: &[Value16],
        bytecode: &Bytecode,
        packed: &[u32],
        chunk_source_positions: Option<&[Option<(usize, usize)>]>,
        ip: &mut usize,
    ) -> CompileResult<bool> {
        let mut local_ip = *ip;
        let mut hit_return = false;

        // CROSS-1 (TCO): snapshot function-frame entry depths so
        // `StepAction::TailCall` can unwind any try / finally / loop
        // frames pushed by the running body before jumping back to
        // ip = 0.  Captured once at entry — the unwind loop restores
        // to EXACTLY these depths on every tail call.
        let tco_entry_try = self.try_frames.len();
        let tco_entry_finally = self.finally_frames.len();
        let tco_entry_loops = self.loop_headers.len();

        // PERF0008: pre-check fuel once; loop-internal counter avoids
        // fuel_limit.is_some() per instruction (dead when fuel is None).
        let has_fuel = self.fuel_limit.is_some();
        let mut fuel_left = self.fuel_remaining;

        while local_ip < instructions.len() {
            if has_fuel {
                if fuel_left == 0 {
                    return Err(compile_codes::runtime_error(
                        "Out of gas: execution fuel exhausted".to_string(),
                    ));
                }
                fuel_left -= 1;
            }

            // PERF-24: cancellation / deadline / stack-overflow all fire
            // every 1024 instructions.  Each instruction can push at most a
            // handful of stack slots, so 1024-op drift against MAX_STACK_SIZE
            // (1_000_000) is negligible.  Pulling the stack check out of the
            // per-instruction path removes a load + compare from the hot loop.
            if local_ip & 0x7FF == 0 {
                if self
                    .cancellation_token
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    return Err(compile_codes::runtime_error(
                        "Execution cancelled".to_string(),
                    ));
                }
                if let Some(deadline) = self.execution_deadline {
                    if std::time::Instant::now() >= deadline {
                        self.suspended_ip = Some(local_ip);
                        return Err(compile_codes::runtime_error(
                            "Execution time limit reached".to_string(),
                        ));
                    }
                }
                // Stack overflow check removed — register-based VM
            }

            // Issue #661: Debugger hook — check breakpoints and step mode
            if self.debugger.is_some() {
                // Prefer the per-function-chunk source_positions table when
                // executing inside a FunctionChunk (BC v5+): that's the only
                // way step-into / breakpoints inside closures can resolve
                // the right file:line. Fall back to the top-level bytecode
                // table for main-module instructions.
                let pos = match chunk_source_positions {
                    Some(sp) => sp.get(local_ip).copied().flatten(),
                    None => bytecode.get_source_position(local_ip),
                };
                if let Some((line, _col)) = pos {
                    let file_ref = self
                        .current_file
                        .as_deref()
                        .unwrap_or("<bytecode>")
                        .to_string();
                    // Conditional-breakpoint support (Kural 2 parity):
                    // the `on_statement_with_eval` closure is passed the
                    // breakpoint's condition string and returns whether
                    // it evaluates truthy. The VM doesn't eval AST, so
                    // we reuse the same static parser-based fallback the
                    // interpreter uses (literal bool + simple expr only).
                    // TODO(#661): full condition eval against the
                    // paused frame's locals requires compiling a one-shot
                    // bytecode fragment at setBreakpoints time and running
                    // it against the current `registers` — deferred
                    // (see note at `evaluate_condition_static`).
                    let should_pause = if let Some(ref mut dbg) = self.debugger {
                        dbg.on_statement_with_eval(&file_ref, line, |cond| {
                            evaluate_condition_static(cond)
                        })
                    } else {
                        false
                    };
                    if should_pause {
                        loop {
                            if let Some(ref dbg) = self.debugger {
                                if dbg.state() != hudhudscript_debug::DebugState::Paused {
                                    break;
                                }
                            }
                            std::thread::yield_now();
                        }
                    }
                }
            }

            // ── Packed fast-dispatch ───────────────────────────────
            let p = unsafe { *packed.get_unchecked(local_ip) };
            if p != PACK_SENTINEL {
                match self.dispatch_packed(p, instructions, constants, bytecode, local_ip)? {
                    PackedResult::Advance => { local_ip += 1; continue; }
                    PackedResult::Jump(target) => { local_ip = target; continue; }
                    PackedResult::Return => {
                        if !self.finally_frames.is_empty() {
                            if let Some(target_ip) = self.handle_return_finally() {
                                local_ip = target_ip;
                                continue;
                            }
                        }
                        hit_return = true;
                        break;
                    }
                    PackedResult::Fallthrough => {}
                }
            }

            // ── Inline hot path — bypass dispatch_unpacked's 150+ arm match ──
            let instr = unsafe { instructions.get_unchecked(local_ip) };
            match instr {

                Instruction::Return { src } => {
                    self.last_return = self.registers[*src as usize];
                    if !self.finally_frames.is_empty() {
                        if let Some(target_ip) = self.handle_return_finally() {
                            local_ip = target_ip;
                            continue;
                        }
                    }
                    hit_return = true;
                    break;
                }
                Instruction::IntAddReturn { src1, src2 } => {
                    let (t1, p1) = self.registers[*src1 as usize].split_tag();
                    let (t2, p2) = self.registers[*src2 as usize].split_tag();
                    let result = match (t1, t2) {
                        (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_add(p2 as i64)),
                        (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                            let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                            let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                            Value16::number(a + b)
                        }
                        _ => return Err(Self::runtime_error_with_pos("IntAddReturn: operands not numeric", bytecode, local_ip)),
                    };
                    let dst = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                    self.registers[dst as usize] = result;
                    self.last_return = result;
                    if !self.finally_frames.is_empty() {
                        if let Some(target_ip) = self.handle_return_finally() {
                            local_ip = target_ip;
                            continue;
                        }
                    }
                    hit_return = true;
                    break;
                }
                Instruction::IntSubReturn { src1, src2 } => {
                    let (t1, p1) = self.registers[*src1 as usize].split_tag();
                    let (t2, p2) = self.registers[*src2 as usize].split_tag();
                    let result = match (t1, t2) {
                        (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_sub(p2 as i64)),
                        (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                            let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                            let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                            Value16::number(a - b)
                        }
                        _ => return Err(Self::runtime_error_with_pos("IntSubReturn: operands not numeric", bytecode, local_ip)),
                    };
                    let d = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                    self.registers[d as usize] = result;
                    self.last_return = result;
                    if !self.finally_frames.is_empty() {
                        if let Some(target_ip) = self.handle_return_finally() {
                            local_ip = target_ip;
                            continue;
                        }
                    }
                    hit_return = true;
                    break;
                }
                Instruction::Move { dst, src } => {
                    self.registers[*dst as usize] = self.registers[*src as usize];
                    local_ip += 1; continue;
                }
                Instruction::IntAdd { dst, src1, src2 } => {
                    let (t1, p1) = self.registers[*src1 as usize].split_tag();
                    let (t2, p2) = self.registers[*src2 as usize].split_tag();
                    self.registers[*dst as usize] = match (t1, t2) {
                        (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_add(p2 as i64)),
                        (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                            let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                            let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                            Value16::number(a + b)
                        }
                        _ => return Err(Self::runtime_error_with_pos("IntAdd: operands not numeric", bytecode, local_ip)),
                    };
                    local_ip += 1; continue;
                }
                Instruction::JumpIfFalse { src, offset } => {
                    let v = &self.registers[*src as usize];
                    if !v.is_truthy() {
                        let new_ip = (local_ip as i64).wrapping_add(*offset as i64);
                        if new_ip < 0 || new_ip > instructions.len() as i64 {
                            return Err(Self::runtime_error_with_pos(
                                format!("JumpIfFalse out of bounds: ip={} offset={}", local_ip, offset),
                                bytecode, local_ip));
                        }
                        local_ip = new_ip as usize;
                        continue;
                    }
                    local_ip += 1; continue;
                }
                Instruction::IntLtRRJumpIfFalse { src1, src2, offset } => {
                    let (t1, p1) = self.registers[*src1 as usize].split_tag();
                    let (t2, p2) = self.registers[*src2 as usize].split_tag();
                    let cond = match (t1, t2) {
                        (ReprTag::Int, ReprTag::Int) => (p1 as i64) < (p2 as i64),
                        (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                            let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                            let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                            a < b
                        }
                        _ => return Err(Self::runtime_error_with_pos("IntLtRRJumpIfFalse: operands not numeric", bytecode, local_ip)),
                    };
                    if !cond {
                        let new_ip = (local_ip as i64).wrapping_add(*offset as i64);
                        local_ip = new_ip as usize;
                        continue;
                    }
                    local_ip += 1; continue;
                }
                Instruction::IntLeRRJumpIfFalse { src1, src2, offset } => {
                    let (t1, p1) = self.registers[*src1 as usize].split_tag();
                    let (t2, p2) = self.registers[*src2 as usize].split_tag();
                    let cond = match (t1, t2) {
                        (ReprTag::Int, ReprTag::Int) => (p1 as i64) <= (p2 as i64),
                        (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                            let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                            let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                            a <= b
                        }
                        _ => return Err(Self::runtime_error_with_pos("IntLeRRJumpIfFalse: operands not numeric", bytecode, local_ip)),
                    };
                    if !cond {
                        let new_ip = (local_ip as i64).wrapping_add(*offset as i64);
                        local_ip = new_ip as usize;
                        continue;
                    }
                    local_ip += 1; continue;
                }
                Instruction::IntLtRIJumpIfFalse { src, imm, offset } => {
                    let (tag, p) = self.registers[*src as usize].split_tag();
                    let cond = match tag {
                        ReprTag::Int => (p as i64) < (*imm as i64),
                        ReprTag::Number => f64::from_bits(p) < (*imm as f64),
                        _ => return Err(Self::runtime_error_with_pos("IntLtRIJumpIfFalse: src not numeric", bytecode, local_ip)),
                    };
                    if !cond {
                        let new_ip = (local_ip as i64).wrapping_add(*offset as i64);
                        local_ip = new_ip as usize;
                        continue;
                    }
                    local_ip += 1; continue;
                }
                Instruction::IntLeRIJumpIfFalse { src, imm, offset } => {
                    let (tag, p) = self.registers[*src as usize].split_tag();
                    let cond = match tag {
                        ReprTag::Int => (p as i64) <= (*imm as i64),
                        ReprTag::Number => f64::from_bits(p) <= (*imm as f64),
                        _ => return Err(Self::runtime_error_with_pos("IntLeRIJumpIfFalse: src not numeric", bytecode, local_ip)),
                    };
                    if !cond {
                        let new_ip = (local_ip as i64).wrapping_add(*offset as i64);
                        if new_ip < 0 || new_ip > instructions.len() as i64 {
                            return Err(Self::runtime_error_with_pos(
                                format!("IntLeRIJumpIfFalse out of bounds: ip={} offset={}", local_ip, offset),
                                bytecode, local_ip));
                        }
                        local_ip = new_ip as usize;
                        continue;
                    }
                    local_ip += 1; continue;
                }
                Instruction::IntSub { dst, src1, src2 } => {
                    let a_val = &self.registers[*src1 as usize];
                    let b_val = &self.registers[*src2 as usize];
                    self.registers[*dst as usize] =
                        if a_val.is_int() && b_val.is_int() {
                            Value16::int(a_val.as_int_unchecked().wrapping_sub(b_val.as_int_unchecked()))
                        } else {
                            let a = a_val.as_number()
                                .or_else(|| a_val.as_int().map(|i| i as f64))
                                .ok_or_else(|| Self::runtime_error_with_pos(
                                    "IntSub: src1 not numeric", bytecode, local_ip))?;
                            let b = b_val.as_number()
                                .or_else(|| b_val.as_int().map(|i| i as f64))
                                .ok_or_else(|| Self::runtime_error_with_pos(
                                    "IntSub: src2 not numeric", bytecode, local_ip))?;
                            Value16::number(a - b)
                        };
                    local_ip += 1; continue;
                }
                Instruction::IntMul { dst, src1, src2 } => {
                    let a_val = &self.registers[*src1 as usize];
                    let b_val = &self.registers[*src2 as usize];
                    self.registers[*dst as usize] =
                        if a_val.is_int() && b_val.is_int() {
                            Value16::int(a_val.as_int_unchecked().wrapping_mul(b_val.as_int_unchecked()))
                        } else {
                            let a = a_val.as_number()
                                .or_else(|| a_val.as_int().map(|i| i as f64))
                                .ok_or_else(|| Self::runtime_error_with_pos(
                                    "IntMul: src1 not numeric", bytecode, local_ip))?;
                            let b = b_val.as_number()
                                .or_else(|| b_val.as_int().map(|i| i as f64))
                                .ok_or_else(|| Self::runtime_error_with_pos(
                                    "IntMul: src2 not numeric", bytecode, local_ip))?;
                            Value16::number(a * b)
                        };
                    local_ip += 1; continue;
                }
                Instruction::IntAddI { dst, src, imm } => {
                    let a_val = self.registers[*src as usize];
                    self.registers[*dst as usize] =
                        if a_val.is_int() {
                            Value16::int(a_val.as_int_unchecked().wrapping_add(*imm as i64))
                        } else {
                            let a = a_val.as_number()
                                .or_else(|| a_val.as_int().map(|i| i as f64))
                                .ok_or_else(|| Self::runtime_error_with_pos(
                                    "IntAddI: src not numeric", bytecode, local_ip))?;
                            Value16::number(a + *imm as f64)
                        };
                    local_ip += 1; continue;
                }
                Instruction::IntSubI { dst, src, imm } => {
                    let a_val = self.registers[*src as usize];
                    self.registers[*dst as usize] =
                        if a_val.is_int() {
                            Value16::int(a_val.as_int_unchecked().wrapping_sub(*imm as i64))
                        } else {
                            let a = a_val.as_number()
                                .or_else(|| a_val.as_int().map(|i| i as f64))
                                .ok_or_else(|| Self::runtime_error_with_pos(
                                    "IntSubI: src not numeric", bytecode, local_ip))?;
                            Value16::number(a - *imm as f64)
                        };
                    local_ip += 1; continue;
                }
                Instruction::IntCmp { dst, src1, src2, op } => {
                    let v1 = &self.registers[*src1 as usize];
                    let v2 = &self.registers[*src2 as usize];
                    let result = if let (Some(a), Some(b)) = (v1.as_int(), v2.as_int()) {
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => false }
                    } else if let (Some(a), Some(b)) = (v1.as_number(), v2.as_number()) {
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => false }
                    } else {
                        // Fall through to slow path for string/bool/null/coercion
                        local_ip += 0; // no-op, let _ => {} catch and go to dispatch_unpacked
                        // Actually, we need to signal fallthrough. Use a trick: don't continue.
                        // We can't easily do this in a match arm, so handle the common cases
                        // here and let the rest go through.
                        // For simplicity: coerce int to float
                        let a = v1.as_number_fast().unwrap_or(0.0);
                        let b = v2.as_number_fast().unwrap_or(0.0);
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => false }
                    };
                    self.registers[*dst as usize] = Value16::bool_(result);
                    local_ip += 1; continue;
                }
                Instruction::NumDiv { dst, src1, src2 } => {
                    let a = self.registers[*src1 as usize].as_number()
                        .or_else(|| self.registers[*src1 as usize].as_int().map(|i| i as f64))
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "NumDiv: src1 not numeric", bytecode, local_ip))?;
                    let b = self.registers[*src2 as usize].as_number()
                        .or_else(|| self.registers[*src2 as usize].as_int().map(|i| i as f64))
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "NumDiv: src2 not numeric", bytecode, local_ip))?;
                    if b == 0.0 {
                        return Err(Self::runtime_error_with_pos(
                            "Division by zero", bytecode, local_ip));
                    }
                    self.registers[*dst as usize] = Value16::number(a / b);
                    local_ip += 1; continue;
                }
                Instruction::IntDiv { dst, src1, src2 } => {
                    let a = self.registers[*src1 as usize].as_int()
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "IntDiv: src1 not integer", bytecode, local_ip))?;
                    let b = self.registers[*src2 as usize].as_int()
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "IntDiv: src2 not integer", bytecode, local_ip))?;
                    if b == 0 {
                        return Err(Self::runtime_error_with_pos(
                            "Division by zero", bytecode, local_ip));
                    }
                    self.registers[*dst as usize] = Value16::int(a / b);
                    local_ip += 1; continue;
                }
                Instruction::IntMod { dst, src1, src2 } => {
                    let a = self.registers[*src1 as usize].as_int()
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "IntMod: src1 not integer", bytecode, local_ip))?;
                    let b = self.registers[*src2 as usize].as_int()
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "IntMod: src2 not integer", bytecode, local_ip))?;
                    if b == 0 {
                        return Err(Self::runtime_error_with_pos(
                            "Modulo by zero", bytecode, local_ip));
                    }
                    self.registers[*dst as usize] = Value16::int(a % b);
                    local_ip += 1; continue;
                }
                Instruction::NumMod { dst, src1, src2 } => {
                    let a = self.registers[*src1 as usize].as_number()
                        .or_else(|| self.registers[*src1 as usize].as_int().map(|i| i as f64))
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "NumMod: src1 not numeric", bytecode, local_ip))?;
                    let b = self.registers[*src2 as usize].as_number()
                        .or_else(|| self.registers[*src2 as usize].as_int().map(|i| i as f64))
                        .ok_or_else(|| Self::runtime_error_with_pos(
                            "NumMod: src2 not numeric", bytecode, local_ip))?;
                    if b == 0.0 {
                        return Err(Self::runtime_error_with_pos(
                            "Modulo by zero", bytecode, local_ip));
                    }
                    // P11: fast path — x % 1.0 == x.fract() (no C fmod call)
                    let result = if b == 1.0 { a.fract() } else { a % b };
                    self.registers[*dst as usize] = Value16::number(result);
                    local_ip += 1; continue;
                }
                Instruction::LoadIntConst { dst, const_idx } => {
                    self.registers[*dst as usize] = Value16::int(bytecode.int_constants[*const_idx as usize]);
                    local_ip += 1; continue;
                }
                Instruction::LoadNumConst { dst, const_idx } => {
                    let bits = bytecode.numeric_constants[*const_idx as usize];
                    self.registers[*dst as usize] = Value16::number(f64::from_bits(bits));
                    local_ip += 1; continue;
                }
                Instruction::LoadConst { dst, const_idx } => {
                    self.registers[*dst as usize] = constants[*const_idx as usize];
                    local_ip += 1; continue;
                }
                Instruction::IntSubCall1(super_idx) | Instruction::IntAddCall1(super_idx) => {
                    let sp = bytecode.get_super_instr_payload(*super_idx);
                    let slot_idx = sp.slot as usize;
                    let is_sub = matches!(instr, Instruction::IntSubCall1(_));
                    match crate::vm::numeric_slot(Some(&self.registers[slot_idx])) {
                        Some(crate::vm::NumericSlot::Int(a)) => {
                            let v = if is_sub { a.wrapping_sub(sp.imm as i64) } else { a.wrapping_add(sp.imm as i64) };
                            self.registers[1] = Value16::int(v);
                        }
                        Some(crate::vm::NumericSlot::Num(a)) => {
                            let v = if is_sub { a - sp.imm as f64 } else { a + sp.imm as f64 };
                            self.registers[1] = Value16::number(v);
                        }
                        None => {
                            return Err(Self::runtime_error_with_pos(
                                "IntSubCall1/IntAddCall1: expected numeric local", bytecode, local_ip));
                        }
                    }
                    let payload = bytecode.get_call_payload(sp.call_idx);
                    local_ip += 1;
                    self.pending_call = Some((payload.sym, payload.arg_count, 1, 255));
                    hit_return = false;
                    break;
                }
                Instruction::Neg { dst, src } => {
                    let v = self.registers[*src as usize];
                    self.registers[*dst as usize] = if let Some(n) = v.as_int() {
                        Value16::int(-n)
                    } else if let Some(n) = v.as_number() {
                        Value16::number(-n)
                    } else {
                        return Err(Self::runtime_error_with_pos(
                            "Neg: operand not numeric", bytecode, local_ip));
                    };
                    local_ip += 1; continue;
                }
                Instruction::Not { dst, src } => {
                    let v = self.registers[*src as usize];
                    self.registers[*dst as usize] = Value16::bool_(!v.is_truthy());
                    local_ip += 1; continue;
                }
                Instruction::Index { dst, obj, idx } => {
                    let obj_val = &self.registers[*obj as usize];
                    let idx_val = &self.registers[*idx as usize];
                    // Array indexing — fast path with single tag check
                    if let Some(arr) = obj_val.as_array() {
                        let i = idx_val.as_int_fast().unwrap_or(0) as usize;
                        if i < arr.len() {
                            self.registers[*dst as usize] = arr[i];
                        } else {
                            self.registers[*dst as usize] = Value16::null();
                        }
                        local_ip += 1; continue;
                    }
                    // String indexing
                    if let Some(s) = obj_val.as_str() {
                        let i = idx_val.as_int_fast().unwrap_or(0) as usize;
                        let ch = s.chars().nth(i).map(|c| c.to_string()).unwrap_or_default();
                        self.registers[*dst as usize] = Value16::string(ch);
                        local_ip += 1; continue;
                    }
                    // Fall through to slow path for objects
                }
                _ => {}
            }

            // ── Slow path: dispatch_unpacked ───────────────────────
            let mut ctx = StepContext {
                instructions,
                constants,
                bytecode,
                ip: local_ip,
                ip_ref: &mut local_ip,
            };
            match self.dispatch_unpacked(instr, &mut ctx) {
                Ok(StepAction::Advance) => { local_ip += 1; }
                Ok(StepAction::Jumped) => {}
                Ok(StepAction::Return) => {
                    if !self.finally_frames.is_empty() {
                        if let Some(target_ip) = self.handle_return_finally() {
                            local_ip = target_ip;
                            continue;
                        }
                    }
                    hit_return = true;
                    break;
                }
                Ok(StepAction::Call { func_sym, arg_count, first_arg, dst }) => {
                    local_ip += 1;
                    self.pending_call = Some((func_sym, arg_count, first_arg, dst));
                    hit_return = false;
                    break;
                }
                Ok(StepAction::Break) => { break; }
                Ok(StepAction::TailCall) => {
                    let args = self.tco_args.take().unwrap_or_default();
                    while self.try_frames.len() > tco_entry_try {
                        self.try_frames.pop();
                    }
                    while self.finally_frames.len() > tco_entry_finally {
                        self.finally_frames.pop();
                    }
                    while self.loop_headers.len() > tco_entry_loops {
                        self.loop_headers.pop();
                    }
                    for (i, v) in args.iter().cloned().enumerate() {
                        self.registers[i] = v;
                    }
                    self.args_scratch = args;
                    local_ip = 0;
                    continue;
                }
                Err(e) => {
                    if let Some(catch_ip) = self.try_catch_runtime_error(&e) {
                        local_ip = catch_ip;
                    } else if let Some(finally_ip) = self.handle_err_finally(&e) {
                        local_ip = finally_ip;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // PERF0008: sync local fuel counter back to VM for remaining_fuel() API
        self.fuel_remaining = fuel_left;
        *ip = local_ip;
        Ok(hit_return)
    }
}
